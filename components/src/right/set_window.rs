use fluent::{FluentBundle, FluentResource};
use gloo::utils::document;
use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlDivElement;
use yew::prelude::*;
use yewdux::Dispatch;

use i18n::{en_us, zh_cn, LanguageType};
use icons::PlusRectIcon;
use sandcat_sdk::api;
use sandcat_sdk::{
    db,
    model::{
        conversation::Conversation,
        friend::Friend,
        group::{Group, GroupMember},
        ItemInfo, RightContentType,
    },
    pb::message::GroupUpdate,
    state::{MuteState, RefreshMsgListState, UpdateConvState},
};
use utils::tr;

pub struct SetWindow {
    members: Vec<GroupMember>,
    info: Option<Box<dyn ItemInfo>>,
    group: Option<Group>,
    friend: Option<Friend>,
    conv: Conversation,
    node: NodeRef,
    i18n: FluentBundle<FluentResource>,
    click_closure: Option<Closure<dyn FnMut(web_sys::MouseEvent)>>,
}

pub enum SetWindowMsg {
    QueryInfo(
        Box<Option<Group>>,
        Box<Option<Friend>>,
        Vec<GroupMember>,
        Conversation,
    ),
    MuteClicked,
    OnFriendNameChange(Event),
    OnGroupNameChange(Event),
    OnGroupAnnoChange(Event),
    OnGroupDescChange(Event),
    DeleteClicked,
    None,
}

#[derive(Properties, PartialEq)]
pub struct SetWindowProps {
    pub conv_type: RightContentType,
    pub user_id: AttrValue,
    pub id: AttrValue,
    pub close: Callback<()>,
    pub plus_click: Callback<()>,
    pub lang: LanguageType,
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
            let mut members: Vec<GroupMember> = vec![];
            let mut friend: Option<Friend> = None;
            let mut group: Option<Group> = None;
            match conv_type {
                RightContentType::Friend => {
                    let f = db::db_ins().friends.get(id.as_str()).await;
                    friend = Some(f);
                }
                RightContentType::Group => {
                    // query group information
                    let g = db::db_ins().groups.get(id.as_str()).await.unwrap().unwrap();
                    group = Some(g);
                    // query members by group id
                    if let Ok(m) = db::db_ins()
                        .group_members
                        .get_list_by_group_id(id.as_str())
                        .await
                    {
                        for v in m.into_iter() {
                            members.push(v);
                        }
                    }
                }
                _ => {}
            }
            // qeury conversation is mute
            let conv = db::db_ins().convs.get_by_frined_id(id.as_str()).await;
            SetWindowMsg::QueryInfo(Box::new(group), Box::new(friend), members, conv)
        });
        let res = match ctx.props().lang {
            LanguageType::ZhCN => zh_cn::SET_WINDOW,
            LanguageType::EnUS => en_us::SET_WINDOW,
        };
        let i18n = utils::create_bundle(res);
        Self {
            i18n,
            members: Vec::new(),
            info: None,
            conv: Conversation::default(),
            node: NodeRef::default(),
            click_closure: None,
            group: None,
            friend: None,
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            if let Some(node) = self.node.cast::<HtmlDivElement>() {
                let _ = node.focus();
                let onclose = ctx.props().close.clone();
                // register click event to document
                let func = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                    if let Some(target) = event.target() {
                        let target_node = target.dyn_into::<web_sys::Node>().unwrap();
                        let node = document().get_element_by_id("setting-window").unwrap();
                        if !node.contains(Some(&target_node)) {
                            onclose.emit(());
                            // 卸载这个onclick 事件
                            document().set_onclick(None);
                        }
                    }
                }) as Box<dyn FnMut(web_sys::MouseEvent)>);
                document().set_onclick(Some(func.as_ref().unchecked_ref()));
                self.click_closure = Some(func);
            }
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SetWindowMsg::QueryInfo(group, friend, members, mute) => {
                self.group = *group;
                self.friend = *friend;
                self.members = members;
                self.conv = mute;
                true
            }
            SetWindowMsg::MuteClicked => {
                self.conv.mute = !self.conv.mute;
                let conv = self.conv.clone();
                // update conversation
                spawn_local(async move {
                    db::db_ins().convs.mute(&conv).await.unwrap();
                });
                // todo send mute message to conversation component
                if let Some(info) = self.info.as_ref() {
                    Dispatch::<MuteState>::global().reduce_mut(|s| s.conv_id = info.id());
                }
                true
            }
            SetWindowMsg::None => false,
            SetWindowMsg::OnGroupNameChange(event) => {
                if let Some(info) = self.group.as_mut() {
                    let name = event
                        .target_unchecked_into::<web_sys::HtmlInputElement>()
                        .value();
                    if info.name != name {
                        info.name = name.into();
                        let name = info.name.clone();
                        let id = info.id.clone();
                        // update group name
                        self.update_group(ctx.props().user_id.clone());
                        Dispatch::<UpdateConvState>::global().reduce_mut(|s| {
                            s.id = id;
                            s.name = Some(name)
                        });
                        return true;
                    }
                }
                false
            }
            SetWindowMsg::OnGroupAnnoChange(event) => {
                if let Some(info) = self.group.as_mut() {
                    let anno = event
                        .target_unchecked_into::<web_sys::HtmlInputElement>()
                        .value();
                    if info.announcement != anno {
                        info.announcement = anno.into();
                        self.update_group(ctx.props().user_id.clone());
                        return true;
                    }
                }
                false
            }
            SetWindowMsg::OnGroupDescChange(event) => {
                if let Some(info) = self.group.as_mut() {
                    let desc = event
                        .target_unchecked_into::<web_sys::HtmlInputElement>()
                        .value();
                    if info.description != desc {
                        info.description = desc.into();
                        self.update_group(ctx.props().user_id.clone());
                        return true;
                    }
                }
                false
            }
            SetWindowMsg::DeleteClicked => {
                match ctx.props().conv_type {
                    RightContentType::Friend => {
                        if let Some(friend) = self.friend.as_ref() {
                            let id = friend.friend_id.clone();
                            spawn_local(async move {
                                // clean group messages
                                if let Err(err) =
                                    db::db_ins().messages.batch_delete(id.as_str()).await
                                {
                                    log::error!("clean group messages error: {:?}", err);
                                }
                                Dispatch::<RefreshMsgListState>::global()
                                    .reduce_mut(|s| s.refresh = !s.refresh);
                            })
                        }
                    }
                    RightContentType::Group => {
                        if let Some(group) = self.group.as_ref() {
                            let id = group.id.clone();
                            spawn_local(async move {
                                // clean group messages
                                if let Err(err) =
                                    db::db_ins().group_msgs.batch_delete(id.as_str()).await
                                {
                                    log::error!("clean group messages error: {:?}", err);
                                }
                                Dispatch::<RefreshMsgListState>::global()
                                    .reduce_mut(|s| s.refresh = !s.refresh);
                            })
                        }
                    }
                    _ => {}
                }
                false
            }
            SetWindowMsg::OnFriendNameChange(event) => {
                if let Some(friend) = self.friend.as_mut() {
                    let r = event
                        .target_unchecked_into::<web_sys::HtmlInputElement>()
                        .value();
                    if let Some(remark) = friend.remark.as_ref() {
                        if *remark != r {
                            friend.remark = Some(r.into());
                            // update friend remark
                            let user_id = ctx.props().user_id.clone().to_string();
                            let friend = friend.clone();
                            self.update_friend_remark(user_id, friend);
                        }
                    } else {
                        friend.remark = Some(r.clone().into());

                        let friend = friend.clone();
                        let user_id = ctx.props().user_id.clone().to_string();
                        let friend = friend.clone();
                        self.update_friend_remark(user_id, friend);
                    }
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut avatars = html!();
        let mut info = html!();
        match ctx.props().conv_type {
            RightContentType::Friend => {
                if let Some(friend) = self.friend.as_ref() {
                    avatars = html! {
                        <div class="avatar-name">
                            <img src={&friend.avatar} />
                            <span>{&friend.name}</span>
                        </div>
                    };
                    let on_name_change = ctx.link().callback(SetWindowMsg::OnFriendNameChange);
                    info = html! {
                        <div class="group-name">
                            <div>
                                {tr!(self.i18n, "remark")}
                            </div>
                            <input type="text" value={friend.remark.clone()} onchange={on_name_change} />
                        </div>
                    };
                }
            }
            RightContentType::Group => {
                avatars = self
                    .members
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
                if let Some(v) = self.group.as_ref() {
                    let on_group_name_change = ctx.link().callback(SetWindowMsg::OnGroupNameChange);
                    let on_group_anno_change = ctx.link().callback(SetWindowMsg::OnGroupAnnoChange);
                    let on_group_desc_change = ctx.link().callback(SetWindowMsg::OnGroupDescChange);
                    info = html! {
                        <>
                            <div class="group-name">
                                <div>
                                    {tr!(self.i18n, "group_name")}
                                </div>
                                <input type="text" value={&v.name} onchange={on_group_name_change} />
                            </div>
                            <div class="group-announcement">
                                <div>
                                    {tr!(self.i18n, "group_announcement")}
                                </div>
                                <input type="text" value={&v.announcement} onchange={on_group_anno_change} />
                            </div>
                            <div class="group-desc">
                                <div>
                                    {tr!(self.i18n, "group_desc")}
                                </div>
                                <input type="text" value={&v.description} onchange={on_group_desc_change} />
                            </div>
                        </>
                    }
                }
            }
            _ => {}
        }

        let add_click = ctx.props().plus_click.reform(|_| ());
        let add_friend = html! {
            <div class="avatar-name pointer" onclick={add_click}>
                <PlusRectIcon/>
                <span>{tr!(self.i18n, "add")}</span>
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

        let setting = html! {
            <div class="setting-item">
            {tr!(self.i18n, "mute")}
            <span class={switch} onclick={mute_click}>
                <span class={slider}></span>
            </span>
            </div>
        };
        html! {
            <div ref={self.node.clone()} id="setting-window" class="set-window box-shadow">
                <div class="people">
                    {avatars}
                    {add_friend}
                </div>
                <div class="info">
                {info}
                </div>
                <div class="setting">
                    {setting}
                </div>
                <div class="bottom pointer" onclick={ctx.link().callback(|_| SetWindowMsg::DeleteClicked)} >
                    {tr!(self.i18n, "delete")}
                </div>
            </div>
        }
    }
}

impl SetWindow {
    fn update_group(&self, user_id: AttrValue) {
        let group = self.group.as_ref().unwrap().clone();
        spawn_local(async move {
            let group = GroupUpdate {
                id: group.id.to_string(),
                name: group.name.to_string(),
                announcement: group.announcement.to_string(),
                description: group.description.to_string(),
                avatar: String::new(),
                update_time: 0,
            };
            match api::groups().update(user_id.as_str(), group).await {
                Ok(group) => {
                    if let Err(err) = db::db_ins().groups.put(&group).await {
                        log::error!("update group name error: {:?}", err);
                    };
                }
                Err(e) => {
                    log::error!("update group name error: {:?}", e)
                }
            };
        });
    }

    fn update_friend_remark(&self, user_id: String, friend: Friend) {
        spawn_local(async move {
            let remark = friend.remark.as_ref().unwrap();
            if api::friends()
                .update_remark(
                    user_id,
                    friend.friend_id.clone().to_string(),
                    remark.to_string(),
                )
                .await
                .is_ok()
            {
                db::db_ins().friends.put_friend(&friend).await;
                Dispatch::<UpdateConvState>::global().reduce_mut(|s| {
                    s.id = friend.friend_id;
                    s.name = Some(friend.remark.unwrap())
                });
            }
        });
    }
}
