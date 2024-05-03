pub mod emoji;
pub mod friend_card;
pub mod friendship_list;
pub mod msg_item;
pub mod msg_list;
pub mod postcard;
pub mod sender;
pub mod set_drawer;
pub mod set_window;
pub mod setting;

use std::rc::Rc;

use fluent::{FluentBundle, FluentResource};
use yew::platform::spawn_local;
use yew::prelude::*;
use yewdux::Dispatch;

use abi::model::RightContentType;
use abi::model::{ComponentType, ItemInfo};
use abi::state::AppState;
use abi::state::{ComponentTypeState, ConvState, CreateConvState, FriendListState, I18nState};
use i18n::{en_us, zh_cn, LanguageType};
use icons::{CatHeadIcon, CloseIcon, MaxIcon};
use utils::tr;

use crate::right::friendship_list::FriendShipList;
use crate::right::set_window::SetWindow;
use crate::right::setting::Setting;
use crate::right::{msg_list::MessageList, postcard::PostCard};
use crate::select_friends::SelectFriendList;

pub struct Right {
    show_setting: bool,
    show_friend_list: bool,
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
}

impl Right {
    fn match_content(&mut self, ctx: &Context<Self>) {
        let id = self.conv_state.conv.item_id.clone();
        if id.is_empty() {
            self.cur_conv_info = None;
            return;
        }
        match self.com_state.component_type {
            ComponentType::Messages => {
                log::debug!(
                    "right conv content type:{:?}",
                    self.conv_state.conv.content_type
                );
                match self.conv_state.conv.content_type {
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
                }
            }

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
        log::debug!("Right update msg: {:?}", msg);
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
                if nodes.is_empty() {
                    return true;
                }
                // create group conversation and send 'create group' message
                Dispatch::<CreateConvState>::global().reduce_mut(|s| s.create_group(nodes));
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
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut top_bar_info = html!();
        let mut setting = html!();
        let mut friend_list = html!();
        if let Some(info) = &self.cur_conv_info {
            let onclick = ctx.link().callback(|event: MouseEvent| {
                event.stop_propagation();
                RightMsg::ShowSetting
            });
            let close = ctx.link().callback(|_| RightMsg::ShowSelectFriendList);
            let submit_back = ctx.link().callback(RightMsg::CreateGroup);

            if self.show_setting {
                setting = html! (
                    <SetWindow
                        id={info.id()}
                        conv_type={info.get_type()}
                        close={ctx.link().callback(|_| RightMsg::ShowSetting)}
                        plus_click={close.clone()}
                        lang={self.lang_state.lang} />);
            }
            if self.show_friend_list {
                friend_list = html!(<SelectFriendList
                            except={info.id()}
                            close_back={close}
                            {submit_back}
                            lang={self.lang_state.lang} />);
            }
            top_bar_info = html! {
                <div class="right-top-bar-friend">
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
                        <h2 class="choose-conv">{tr!(self.i18n, "hello")}</h2>
                    }
                } else {
                    html! {
                    <MessageList
                        friend_id={&self.conv_state.conv.item_id.clone()}
                        cur_user_avatar={self.state.login_user.avatar.clone()}
                        conv_type={self.conv_state.conv.content_type.clone()}
                        cur_user_id={self.state.login_user.id.clone()}
                        lang={self.lang_state.lang}/>
                    }
                }
            }
            ComponentType::Contacts => {
                // 要根据右部内容类型绘制页面
                match self.friend_list_state.friend.content_type {
                    RightContentType::Friend
                    | RightContentType::Group
                    | RightContentType::UserInfo => {
                        html! {
                            <PostCard user_id={&self.state.login_user.id}
                            id={&self.friend_list_state.friend.item_id}
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
            <div class="right-container">
                <div class="right-top-bar">
                    <div class="close-bar">
                        <span></span>
                        <MaxIcon/>
                        <CloseIcon/>
                    </div>
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
