pub mod emoji;
pub mod friend_card;
pub mod friendship_list;
pub mod msg_item;
pub mod msg_list;
mod msg_right_click;
pub mod postcard;
mod recorder;
pub mod sender;
pub mod set_drawer;
pub mod set_window;
pub mod setting;

use std::rc::Rc;

use fluent::{FluentBundle, FluentResource};
use gloo::timers::callback::Timeout;
use log::error;
use sandcat_sdk::model::group::GroupMemberFromServer;
use sandcat_sdk::model::notification::Notification;
use sandcat_sdk::pb::message::GroupInviteNew;
use wasm_bindgen::JsCast;
use web_sys::{CssAnimation, HtmlDivElement};
use yew::platform::spawn_local;
use yew::prelude::*;
use yewdux::Dispatch;

use i18n::{en_us, zh_cn, LanguageType};
use icons::{BackIcon, CatHeadIcon, CloseIcon, MaxIcon};
use sandcat_sdk::model::RightContentType;
use sandcat_sdk::model::{ComponentType, ItemInfo};
use sandcat_sdk::state::{AppState, MobileState, Notify, ShowRight};
use sandcat_sdk::state::{
    ComponentTypeState, ConvState, CreateGroupConvState, FriendListState, I18nState,
};
use sandcat_sdk::{api, db};
use utils::tr;

use crate::constant::HELLO;
use crate::right::friendship_list::FriendShipList;
use crate::right::set_window::SetWindow;
use crate::right::setting::Setting;
use crate::right::{msg_list::MessageList, postcard::PostCard};
use crate::select_friends::SelectFriendList;

pub struct Right {
    show_setting: bool,
    show_friend_list: bool,
    node_ref: NodeRef,
    touch_start: Option<TouchInfo>,
    timeout: Option<Timeout>,
    i18n: FluentBundle<FluentResource>,
    state: Rc<AppState>,
    _app_dis: Dispatch<AppState>,
    com_state: Rc<ComponentTypeState>,
    _com_dis: Dispatch<ComponentTypeState>,
    conv_state: Rc<ConvState>,
    _conv_dis: Dispatch<ConvState>,
    cur_conv_info: Option<Box<dyn ItemInfo>>,
    friend_list_state: Rc<FriendListState>,
    _friend_list_dis: Dispatch<FriendListState>,
    lang_state: Rc<I18nState>,
    _lang_dispatch: Dispatch<I18nState>,
}

#[derive(Debug, Default)]
struct TouchInfo {
    x: i32,
    y: i32,
    start_time: i64,
}

#[derive(Debug)]
pub enum RightMsg {
    StateChanged(Rc<AppState>),
    ComStateChanged(Rc<ComponentTypeState>),
    ConvStateChanged(Rc<ConvState>),
    ContentChange(Option<Box<dyn ItemInfo>>),
    FriendListStateChanged(Rc<FriendListState>),
    ShowSetting,
    ShowSelectFriendList,
    CreateGroup(Vec<String>),
    SwitchLang(Rc<I18nState>),
    Close,
    CleanTimer,
    TouchStart(TouchEvent),
    TouchEnd(TouchEvent),
}

impl Right {
    fn match_content(&mut self, ctx: &Context<Self>) {
        let id = self.conv_state.conv.item_id.clone();
        if id.is_empty() {
            self.cur_conv_info = None;
            return;
        }

        match self.com_state.component_type {
            ComponentType::Messages => match self.conv_state.conv.content_type {
                RightContentType::Default => {}
                RightContentType::Friend => {
                    ctx.link().send_future(async move {
                        let friend = db::db_ins().friends.get(id.as_str()).await;
                        RightMsg::ContentChange(Some(Box::new(friend)))
                    });
                }
                RightContentType::Group => {
                    let ctx = ctx.link().clone();
                    spawn_local(async move {
                        if let Ok(Some(group)) = db::db_ins().groups.get(id.as_str()).await {
                            ctx.send_message(RightMsg::ContentChange(Some(Box::new(group))));
                        }
                    });
                }

                _ => {}
            },

            _ => self.cur_conv_info = None,
        }
    }
}

impl Component for Right {
    type Message = RightMsg;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let conv_dis =
            Dispatch::global().subscribe(ctx.link().callback(RightMsg::ConvStateChanged));
        let _friend_list_dis =
            Dispatch::global().subscribe(ctx.link().callback(RightMsg::FriendListStateChanged));
        let lang_dispatch = Dispatch::global().subscribe(ctx.link().callback(RightMsg::SwitchLang));
        let app_dis = Dispatch::global().subscribe(ctx.link().callback(RightMsg::StateChanged));
        let com_dis = Dispatch::global().subscribe(ctx.link().callback(RightMsg::ComStateChanged));
        let lang_state = lang_dispatch.get();
        let cur_conv_info = None;
        let res = match lang_state.lang {
            LanguageType::ZhCN => zh_cn::RIGHT_PANEL,
            LanguageType::EnUS => en_us::RIGHT_PANEL,
        };
        let i18n = utils::create_bundle(res);
        Self {
            show_setting: false,
            show_friend_list: false,
            i18n,
            node_ref: NodeRef::default(),
            touch_start: None,
            timeout: None,
            state: app_dis.get(),
            _app_dis: app_dis,
            conv_state: conv_dis.get(),
            _conv_dis: conv_dis,
            cur_conv_info,
            friend_list_state: _friend_list_dis.get(),
            _friend_list_dis,
            lang_state,
            _lang_dispatch: lang_dispatch,
            com_state: com_dis.get(),
            _com_dis: com_dis,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            RightMsg::StateChanged(state) => {
                // 根据state中的不同数据变化，渲染右侧页面
                self.state = state;
                // 为了标题栏展示好友名字以及当前会话设置
                true
            }
            RightMsg::ContentChange(item) => {
                self.cur_conv_info = item;
                true
            }
            RightMsg::FriendListStateChanged(state) => {
                self.friend_list_state = state;
                true
            }
            RightMsg::ConvStateChanged(state) => {
                self.conv_state = state;
                self.match_content(ctx);
                true
            }
            RightMsg::ShowSetting => {
                if self.show_friend_list {
                    return false;
                }
                self.show_setting = !self.show_setting;
                true
            }
            RightMsg::ShowSelectFriendList => {
                self.show_friend_list = !self.show_friend_list;
                true
            }
            RightMsg::CreateGroup(nodes) => {
                self.show_friend_list = false;
                // todo need to handle the group invitation or create group
                // create group conversation and send 'create group' message
                if self.conv_state.conv.content_type == RightContentType::Friend {
                    Dispatch::<CreateGroupConvState>::global()
                        .reduce_mut(|s| s.create_group(nodes));
                } else if self.conv_state.conv.content_type == RightContentType::Group {
                    // invite the member
                    let user_id = self.state.login_user.id.to_string();
                    let group_id = self.conv_state.conv.item_id.to_string();
                    spawn_local(async move {
                        let group = match db::db_ins().groups.get(&group_id).await {
                            Ok(Some(group)) => group,
                            Ok(None) => {
                                Notification::error("get group info error").notify();
                                return;
                            }
                            Err(e) => {
                                error!("get group info error:{:?}", e);
                                Notification::error("get group info error").notify();
                                return;
                            }
                        };

                        if let Err(e) = api::groups()
                            .invite(GroupInviteNew {
                                user_id,
                                group_id,
                                members: nodes.clone(),
                            })
                            .await
                        {
                            error!("invite member error:{:?}", e);
                            Notification::error("invite member error").notify();
                            // return;
                        }
                        let time = chrono::Utc::now().timestamp_millis();
                        // update local group member list
                        let friends = db::db_ins().friends.get_list_by_ids(nodes).await.unwrap();
                        let members = friends
                            .into_iter()
                            .map(|friend| GroupMemberFromServer::from_friend(&group, friend, time))
                            .collect();
                        db::db_ins().group_members.put_list(members).await.unwrap();
                    });
                }
                self.show_friend_list = false;
                self.show_setting = false;
                true
            }
            RightMsg::SwitchLang(state) => {
                self.lang_state = state;
                let res = match self.lang_state.lang {
                    LanguageType::ZhCN => zh_cn::RIGHT_PANEL,
                    LanguageType::EnUS => en_us::RIGHT_PANEL,
                };
                let i18n = utils::create_bundle(res);
                self.i18n = i18n;
                true
            }
            RightMsg::ComStateChanged(state) => {
                self.com_state = state;
                self.match_content(ctx);
                true
            }
            RightMsg::Close => {
                if let Some(node) = self.node_ref.cast::<HtmlDivElement>() {
                    let animations = node.get_animations();
                    for i in 0..animations.length() {
                        let animation = animations.get(i);
                        if let Ok(animation) = animation.dyn_into::<CssAnimation>() {
                            if animation.animation_name() != "right-in" {
                                continue;
                            }
                            let _ = animation.reverse();
                            let ctx = ctx.link().clone();
                            self.timeout = Some(Timeout::new(200, move || {
                                ShowRight::None.notify();
                                ctx.send_message(RightMsg::CleanTimer);
                            }));
                        }
                    }
                }
                false
            }
            RightMsg::CleanTimer => {
                self.timeout = None;
                false
            }
            RightMsg::TouchStart(event) => {
                if let Some(touch) = event.touches().get(0) {
                    self.touch_start = Some(TouchInfo {
                        x: touch.client_x(),
                        y: touch.client_y(),
                        start_time: chrono::Utc::now().timestamp_millis(),
                    });
                };
                false
            }
            RightMsg::TouchEnd(event) => {
                // we can't use the .touches() to get the touch end
                // should use the changed_touches()
                if let Some(touch) = event.changed_touches().get(0) {
                    if let Some(ref info) = self.touch_start {
                        if touch.client_x() - info.x > 50
                            && (info.y - touch.client_y() < 20 || touch.client_y() - info.y < 20)
                            && (chrono::Utc::now().timestamp_millis() - info.start_time < 200)
                        {
                            ctx.link().send_message(RightMsg::Close);
                        }
                    }
                }
                self.touch_start = None;
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut setting = html!();
        let mut friend_list = html!();
        let (class, right_top_bar_class, back, operation_bar, ontouchstart, ontouchend) =
            match *MobileState::get() {
                MobileState::Desktop => (
                    "right-container",
                    "right-top-bar-friend",
                    html!(),
                    html! {
                        <div class="close-bar">
                            <span></span>
                            <MaxIcon/>
                            <CloseIcon/>
                        </div>
                    },
                    None,
                    None,
                ),
                MobileState::Mobile => (
                    "right-container-mobile",
                    "right-top-bar-friend-mobile",
                    html!(<span onclick={ctx.link().callback(|_| RightMsg::Close)}><BackIcon/></span>),
                    html!(),
                    Some(ctx.link().callback(RightMsg::TouchStart)),
                    Some(ctx.link().callback(RightMsg::TouchEnd)),
                ),
            };
        let mut top_bar_info = html!(
                <div class={right_top_bar_class}>
                    {back.clone()}<span></span><span></span>
                </div>);
        if let Some(info) = &self.cur_conv_info {
            let onclick = ctx.link().callback(|event: MouseEvent| {
                event.stop_propagation();
                RightMsg::ShowSetting
            });
            let close = ctx.link().callback(|_| RightMsg::ShowSelectFriendList);

            if self.show_setting {
                setting = html! (
                    <SetWindow
                        id={info.id()}
                        user_id={&self.state.login_user.id}
                        conv_type={info.get_type()}
                        close={ctx.link().callback(|_| RightMsg::ShowSetting)}
                        plus_click={close.clone()}
                        lang={self.lang_state.lang} />);
            }
            if self.show_friend_list {
                let submit_back = ctx.link().callback(RightMsg::CreateGroup);
                friend_list = html!(
                    <SelectFriendList
                        except={info.id()}
                        close_back={close}
                        {submit_back}
                        lang={self.lang_state.lang} />);
            }
            top_bar_info = html! {
                <div class={right_top_bar_class}>
                    {back}
                    <span>
                        {info.name()}
                    </span>
                    <span class="pointer" {onclick}>
                        {"···"}
                    </span>
                </div>
            }
        }
        let content = match self.com_state.component_type {
            ComponentType::Messages => {
                // 处理没有选中会话的情况
                if self.conv_state.conv.item_id.is_empty() {
                    html! {
                        <div class="choose-conv">
                            <CatHeadIcon/>
                            <h2 >{tr!(self.i18n, HELLO)}</h2>
                        </div>
                    }
                } else {
                    html! {
                    <MessageList
                        friend_id={&self.conv_state.conv.item_id}
                        cur_user_avatar={&self.state.login_user.avatar}
                        nickname={&self.state.login_user.name}
                        conv_type={self.conv_state.conv.content_type.clone()}
                        cur_user_id={&self.state.login_user.id}
                        lang={self.lang_state.lang}/>
                    }
                }
            }
            ComponentType::Contacts => {
                // 要根据右部内容类型绘制页面
                match self.friend_list_state.friend.content_type {
                    RightContentType::Friend | RightContentType::Group => {
                        html! {
                            <PostCard
                                user_id={&self.state.login_user.id}
                                id={&self.friend_list_state.friend.item_id}
                                avatar={&self.state.login_user.avatar}
                                nickname={&self.state.login_user.name}
                                conv_type={self.friend_list_state.friend.content_type.clone()}
                                lang={self.lang_state.lang}/>
                        }
                    }
                    RightContentType::FriendShipList => {
                        html! {
                            <FriendShipList user_id={&self.state.login_user.id} lang={self.lang_state.lang}/>
                        }
                    }
                    _ => {
                        html!(<div class="cat-head-icon"><CatHeadIcon/></div>)
                    }
                }
            }
            ComponentType::Setting => html! {<Setting lang={self.lang_state.lang} />},
            ComponentType::Default => html!(),
        };

        html! {
            <div ref={self.node_ref.clone()}
                {class}
                {ontouchstart}
                {ontouchend}>
                <div class="right-top-bar">
                    {operation_bar}
                    {top_bar_info}
                </div>
                    {setting}
                    {friend_list}
                <div class="msg-container">
                    {content}
                </div>
            </div>
        }
    }
}
