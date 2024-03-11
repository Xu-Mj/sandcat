pub mod emoji;
pub mod friend_card;
pub mod friendship_list;
pub mod msg_item;
pub mod msg_list;
pub mod postcard;
pub mod sender;
pub mod set_drawer;

use std::rc::Rc;

use yew::platform::spawn_local;
use yew::prelude::*;

use crate::components::right::friendship_list::FriendShipList;
use crate::db::friend::FriendRepo;
use crate::db::group::GroupRepo;
use crate::icons::{CloseIcon, MaxIcon};
use crate::model::ItemInfo;
use crate::model::RightContentType;
use crate::pages::{ConvState, FriendListState};
use crate::{
    components::right::{msg_list::MessageList, postcard::PostCard},
    pages::{AppState, ComponentType},
};

pub struct Right {
    pub state: Rc<AppState>,
    pub _ctx_listener: ContextHandle<Rc<AppState>>,
    pub conv_state: Rc<ConvState>,
    pub _conv_listener: ContextHandle<Rc<ConvState>>,
    pub cur_conv_info: Option<Box<dyn ItemInfo>>,
    pub friend_list_state: Rc<FriendListState>,
    pub _friend_list_listener: ContextHandle<Rc<FriendListState>>,
}

pub enum RightMsg {
    StateChanged(Rc<AppState>),
    ConvStateChanged(Rc<ConvState>),
    ContentChange(Option<Box<dyn ItemInfo>>),
    FriendListStateChanged(Rc<FriendListState>),
    ShowSetting,
}

impl Right {
    fn match_content(&mut self, ctx: &Context<Self>) {
        let id = self.conv_state.conv.item_id.clone();
        if id.is_empty() {
            self.cur_conv_info = None;
            return;
        }
        match self.state.component_type {
            ComponentType::Messages => {
                log::debug!(
                    "right conv content type:{:?}",
                    self.conv_state.conv.content_type
                );
                match self.conv_state.conv.content_type {
                    RightContentType::Default => {}
                    RightContentType::Friend => {
                        ctx.link().send_future(async {
                            let friend = FriendRepo::new().await.get_friend(id).await;
                            RightMsg::ContentChange(Some(Box::new(friend)))
                        });
                    }
                    RightContentType::Group => {
                        let ctx = ctx.link().clone();
                        spawn_local(async move {
                            if let Ok(Some(group)) = GroupRepo::new().await.get(id).await {
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
        let (state, _ctx_listener) = ctx
            .link()
            .context(ctx.link().callback(RightMsg::StateChanged))
            .expect("expect state");
        let (conv_state, _conv_listener) = ctx
            .link()
            .context(ctx.link().callback(RightMsg::ConvStateChanged))
            .expect("expect state");
        let (friend_list_state, _friend_list_listener) = ctx
            .link()
            .context(ctx.link().callback(RightMsg::FriendListStateChanged))
            .expect("expect state");
        let cur_conv_info = None;
        Self {
            state,
            _ctx_listener,
            conv_state,
            _conv_listener,
            cur_conv_info,
            friend_list_state,
            _friend_list_listener,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            RightMsg::StateChanged(state) => {
                // 根据state中的不同数据变化，渲染右侧页面
                self.state = state;
                // 为了标题栏展示好友名字以及当前会话设置
                self.match_content(ctx);
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
            RightMsg::ShowSetting => true,
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let mut top_bar_info = html!();
        if let Some(info) = &self.cur_conv_info {
            let onclick = _ctx.link().callback(|_| RightMsg::ShowSetting);
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
        let content = match self.state.component_type {
            ComponentType::Messages => {
                // 处理没有选中会话的情况
                if self.conv_state.conv.item_id.is_empty() {
                    html! {
                        <h2 class="choose-conv">{"与挚友开始聊天吧！"}</h2>
                    }
                } else {
                    html! {
                    <MessageList
                        friend_id={&self.conv_state.conv.item_id.clone()}
                        cur_user_avatar={self.state.login_user.avatar.clone()}
                        conv_type={self.conv_state.conv.content_type.clone()}
                        cur_user_id={self.state.login_user.id.clone()}/>
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
                            <PostCard user_id={&self.state.login_user.id.clone()}
                            friend_id={&self.friend_list_state.friend.item_id.clone()}
                            conv_type={self.friend_list_state.friend.content_type.clone()}/>
                        }
                    }
                    RightContentType::FriendShipList => {
                        log::debug!("right msg container");
                        html! {
                            // self.friendships.iter().map(|item|
                            //
                            // )
                            <FriendShipList/>
                        }
                    }
                    _ => {
                        html!()
                    }
                }
            }
            ComponentType::Setting => html! {},
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
                <div class="msg-container">
                    {content}
                </div>
            </div>
        }
    }
}
