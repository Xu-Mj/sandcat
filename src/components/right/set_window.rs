use web_sys::HtmlDivElement;
use yew::prelude::*;

use crate::{
    db::{conv::ConvRepo, friend::FriendRepo, group_members::GroupMembersRepo},
    icons::PlusRectIcon,
    model::{ItemInfo, RightContentType},
};
#[derive(Default)]
pub struct SetWindow {
    list: Vec<Box<dyn ItemInfo>>,
    is_mute: bool,
    node: NodeRef,
}

pub enum SetWindowMsg {
    QueryInfo(Vec<Box<dyn ItemInfo>>, bool),
}

#[derive(Properties, PartialEq)]
pub struct SetWindowProps {
    pub conv_type: RightContentType,
    pub id: AttrValue,
    pub close: Callback<()>,
    pub plus_click: Callback<()>,
}

/// query friend/group information by props id
/// layout:
/// top: list people's avatar
/// middle: some settings
/// bottom: delete
impl Component for SetWindow {
    type Message = SetWindowMsg;

    type Properties = SetWindowProps;

    fn create(ctx: &Context<Self>) -> Self {
        let id = ctx.props().id.clone();
        let conv_type = ctx.props().conv_type.clone();
        ctx.link().send_future(async move {
            let mut info: Vec<Box<dyn ItemInfo>> = vec![];
            match conv_type {
                RightContentType::Friend => {
                    let friend = FriendRepo::new().await.get_friend(id.clone()).await;
                    info.push(Box::new(friend));
                }
                RightContentType::Group => {
                    // query members by group id
                    if let Ok(list) = GroupMembersRepo::new()
                        .await
                        .get_list_by_group_id(id.as_str())
                        .await
                    {
                        for v in list.into_iter() {
                            info.push(Box::new(v));
                        }
                    }
                }
                _ => {}
            }
            // qeury conversation is mute
            let conv = ConvRepo::new().await.get_by_frined_id(id).await;
            SetWindowMsg::QueryInfo(info, conv.mute)
        });
        Self {
            ..Default::default()
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if let Some(node) = self.node.cast::<HtmlDivElement>() {
            node.focus().unwrap();
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SetWindowMsg::QueryInfo(info, mute) => {
                self.list = info;
                self.is_mute = mute;
                true
            }
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let avatars = self
            .list
            .iter()
            .map(|item| {
                html! {
                    <div class="avatar-name">
                        <img src={item.avatar()} />
                        <span>{item.name()}</span>
                    </div>
                }
            })
            .collect::<Html>();
        let add_click = ctx.props().plus_click.reform(|_| ());
        let add_friend = html! {
            <div class="avatar-name pointer" onclick={add_click}>
                <PlusRectIcon/>
                <span>{"添加"}</span>
            </div>
        };
        let setting = html! {
            <>
            <label for="msg-mute">{"消息免打扰"}</label>
            <input id="msg-mute" type="switch" /* value={self.is_mute} *//>
            </>
        };
        let onblur = ctx.props().close.reform(|_| ());
        html! {
            <div ref={self.node.clone()} tabindex="0"  {onblur} class="set-window box-shadow">
                <div class="people">
                    {avatars}
                    {add_friend}
                </div>
                <div class="setting">
                    {setting}
                </div>
                <div class="bottom">
                    {"删除"}
                </div>
            </div>
        }
    }
}
