use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlDivElement;
use yew::prelude::*;

use crate::{
    db::{self, conv::ConvRepo, conversations::Conversations},
    icons::PlusRectIcon,
    model::{conversation::Conversation, ItemInfo, RightContentType},
};
#[derive(Default)]
pub struct SetWindow {
    list: Vec<Box<dyn ItemInfo>>,
    info: Option<Box<dyn ItemInfo>>,
    conv: Conversation,
    node: NodeRef,
}

pub enum SetWindowMsg {
    QueryInfo(
        Option<Box<dyn ItemInfo>>,
        Vec<Box<dyn ItemInfo>>,
        Conversation,
    ),
    MuteClicked,
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
            // init interfaces
            let group_db = db::groups().await;
            let friend_db = db::friends().await;
            let mut list: Vec<Box<dyn ItemInfo>> = vec![];
            let mut info: Option<Box<dyn ItemInfo>> = None;
            match conv_type {
                RightContentType::Friend => {
                    let friend = friend_db.get_friend(id.as_str()).await;
                    info = Some(Box::new(friend.clone()));
                    list.push(Box::new(friend));
                }
                RightContentType::Group => {
                    // query group information
                    let group = group_db.get(id.as_str()).await.unwrap().unwrap();
                    info = Some(Box::new(group.clone()));
                    // query members by group id
                    if let Ok(members) = db::group_members()
                        .await
                        .get_list_by_group_id(id.as_str())
                        .await
                    {
                        for v in members.into_iter() {
                            list.push(Box::new(v));
                        }
                    }
                }
                _ => {}
            }
            // qeury conversation is mute
            let conv = ConvRepo::new().await.get_by_frined_id(id.as_str()).await;
            SetWindowMsg::QueryInfo(info, list, conv)
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
            SetWindowMsg::QueryInfo(info, list, mute) => {
                self.list = list;
                self.info = info;
                self.conv = mute;
                true
            }
            SetWindowMsg::MuteClicked => {
                self.conv.mute = !self.conv.mute;
                let conv = self.conv.clone();
                // update conversation
                spawn_local(async move {
                    ConvRepo::new().await.mute(&conv).await.unwrap();
                });
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
        let mute_click = ctx.link().callback(|_| SetWindowMsg::MuteClicked);
        let mut switch = classes!("switch", "pointer");
        let mut slider = classes!("slider");
        if self.conv.mute {
            switch.push("background-change");
            slider.push("right");
        } else {
            slider.push("left");
        }
        let mut info = html!();
        if ctx.props().conv_type == RightContentType::Group {
            if let Some(v) = self.info.as_ref() {
                info = html! {
                    <div class="info">
                        <div class="group-name">
                            <div>
                                {"群聊名称"}
                            </div>
                            <input type="text" value={v.name()} />
                        </div>
                        <div class="group-announcement">
                            <div>
                                {"群公告"}
                            </div>
                            <input type="text" value={v.remark()} />
                        </div>
                        <div class="group-desc">
                            <div>
                                {"群描述"}
                            </div>
                            <input type="text" value={v.signature()} />
                        </div>
                    </div>
                }
            }
        }
        let setting = html! {
            <div class="setting-item">
            {"消息免打扰"}
            <span class={switch} onclick={mute_click}>
                <span class={slider}></span>
            </span>
            </div>
        };
        let onblur = ctx.props().close.reform(|_| ());
        html! {
            <div ref={self.node.clone()} tabindex="0"  {onblur} class="set-window box-shadow">
                <div class="people">
                    {avatars}
                    {add_friend}
                </div>
                {info}
                <div class="setting">
                    {setting}
                </div>
                <div class="bottom" >
                    {"删除"}
                </div>
            </div>
        }
    }
}
