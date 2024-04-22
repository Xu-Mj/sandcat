pub(crate) mod home;
use std::rc::Rc;

use gloo::timers::callback::Interval;
use gloo::utils::window;
use web_sys::HtmlDivElement;
use yew::platform::spawn_local;
use yew::prelude::*;
use yewdux::Dispatch;

use super::{
    AddFriendState, AddFriendStateItem, ConvState, CreateConvState, FriendListState,
    FriendShipState, SendResultState,
};
use crate::db::current_item;
use crate::db::repository::Repository;
use crate::icons::CloseIcon;
use crate::model::friend::{Friend, FriendShipWithUser};
use crate::model::message::{Msg, ServerResponse};
use crate::model::notification::{Notification, NotificationState, NotificationType};
use crate::model::user::User;
use crate::model::{ComponentType, CurrentItem, FriendShipStateType, RightContentType};

use crate::state::{AppState, SendMessageState};
use crate::{
    components::{left::Left, right::Right},
    db::QueryStatus,
};

pub struct Home {
    notification_node: NodeRef,
    notification_interval: Option<Interval>,
    // call_state: Rc<RecSendCallState>,
    conv_state: Rc<ConvState>,
    friend_state: Rc<FriendListState>,
    friend_ship_state: Rc<FriendShipState>,
    add_friend_state: Rc<AddFriendState>,
    notifications: Vec<Notification>,
    notification: Rc<NotificationState>,
    create_conv: Rc<CreateConvState>,
    send_result: Rc<SendResultState>,
}

pub enum HomeMsg {
    // 联系人列表选中元素改变
    SwitchFriend(CurrentItem),
    // 会话列表选中元素改变
    SwitchConv(CurrentItem),
    // 查询数据库
    Query(QueryStatus<QueryResult>),
    // 收到消息
    // RecMsgStateChange(Msg),
    // 收到消息
    // ReceiveMessage(Msg),
    // 收到好友请求
    ReceiveFriendShipReq(FriendShipWithUser),
    // 回复好友请求
    FriendShipResponse((AttrValue, Friend)),
    // received friend application response
    RecFsResp(Friend),
    // 显示顶部消息通知
    // 发送视频电话请求
    // 发送消息
    SendMessage(Msg),
    // 发送消息收到
    // SendBackMsg(Msg),
    Notification(Notification),
    CleanNotification,
    CloseNotificationByIndex(usize),
    // 创建会话状态改变回调
    CreateFriendConv((RightContentType, Friend)),
    CreateGroupConv((RightContentType, Vec<String>)),
    AddFriendStateChange(AddFriendStateItem),
    SendResultState(ServerResponse),
}

#[derive(Properties, Clone, PartialEq)]
pub struct HomeProps {
    pub id: AttrValue,
}

type QueryResult = (User, CurrentItem, CurrentItem, ComponentType);

impl Component for Home {
    type Message = HomeMsg;
    type Properties = HomeProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self::new(ctx)
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            HomeMsg::SwitchFriend(conv) => {
                let friend_state = Rc::make_mut(&mut self.friend_state);
                if friend_state.friend.item_id == conv.item_id && !conv.item_id.is_empty() {
                    return false;
                }
                friend_state.friend = conv.clone();
                // 数据库更新config表，记录当前current_friend_id
                current_item::save_friend(&conv).unwrap();
                true
            }
            HomeMsg::SwitchConv(conv) => {
                let conv_state = Rc::make_mut(&mut self.conv_state);
                // 如果id没有变化，那么不更新数据库
                if conv_state.conv.item_id == conv.item_id
                    && conv_state.conv.content_type == conv.content_type
                {
                    return false;
                }
                current_item::save_conv(&conv).unwrap();
                conv_state.conv = conv;
                true
            }
            HomeMsg::Query(status) => {
                match status {
                    QueryStatus::QuerySuccess(u) => {
                        Dispatch::<AppState>::global().reduce_mut(|s| {
                            s.login_user = u.0;
                            s.component_type = u.3;
                        });
                        let conv_state = Rc::make_mut(&mut self.conv_state);
                        conv_state.conv = u.1;
                        let friend_state = Rc::make_mut(&mut self.friend_state);
                        friend_state.friend = u.2;
                    }
                    QueryStatus::QueryFail(_) => {
                        gloo::console::log!("query fail")
                    }
                    _ => {}
                }
                true
            }
            HomeMsg::SendMessage(msg) => {
                // change the send message state to send hello
                Dispatch::<SendMessageState>::global().reduce_mut(|s| s.msg = msg);
                false
            }
            // todo don't need anymore
            // HomeMsg::SendBackMsg(_msg) => {
            // 发送已收到消息给服务器
            // self.send_msg(msg);
            // false
            // }
            // don't need anymore, at the conversation component
            HomeMsg::ReceiveFriendShipReq(friendship) => {
                // self.handle_friendship_req(ctx, friendship)
                let state = Rc::make_mut(&mut self.friend_ship_state);
                state.ship = Some(friendship.clone());
                state.state_type = FriendShipStateType::Req;
                true
            }
            HomeMsg::FriendShipResponse((friendship_id, friend)) => {
                self.agree_friendship(ctx, friendship_id, friend)
            }
            HomeMsg::Notification(noti) => {
                log::debug!("notification:{:?}", &noti);
                self.notify(noti);
                let ctx = ctx.link().clone();
                if self.notification_interval.is_none() {
                    self.notification_interval = Some(Interval::new(3 * 1000, move || {
                        ctx.send_message(HomeMsg::CleanNotification)
                    }));
                }
                true
            }
            HomeMsg::CleanNotification => {
                if !self.notifications.is_empty() {
                    self.notifications.remove(0);
                } else {
                    self.notification_interval = None;
                }
                true
            }
            HomeMsg::CloseNotificationByIndex(index) => {
                if index < self.notifications.len() {
                    self.notifications.remove(index);
                    return true;
                }
                false
            }
            HomeMsg::CreateFriendConv((t, info)) => {
                let state = Rc::make_mut(&mut self.create_conv);
                state.type_ = t;
                state.friend = Some(info);
                true
            }
            HomeMsg::CreateGroupConv((t, list)) => {
                if list.is_empty() {
                    return false;
                }
                let state = Rc::make_mut(&mut self.create_conv);
                state.type_ = t;
                state.group = Some(list);
                true
            }
            HomeMsg::AddFriendStateChange(item) => {
                let state = Rc::make_mut(&mut self.add_friend_state);
                state.item = item;
                true
            }
            HomeMsg::RecFsResp(friend) => {
                let state = Rc::make_mut(&mut self.friend_ship_state);
                state.friend = Some(friend);
                state.state_type = FriendShipStateType::RecResp;
                true
            }
            HomeMsg::SendResultState(resp) => {
                let state = Rc::make_mut(&mut self.send_result);
                state.msg = resp;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let notify = self
            .notifications
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let mut class = classes!("notification-item") ;
                match item.type_ {
                    NotificationType::Info => class.push("info") ,
                    // NotificationType::Success => class.push("success"),
                    NotificationType::Warn => class.push("warn"),
                    NotificationType::Error => class.push("error"),
                }
                html! {
                    <div {class} key={index}>
                        <div class="notification-close" onclick={ctx.link().callback(move |_| HomeMsg::CloseNotificationByIndex(index))}>
                            <CloseIcon />
                        </div>
                        <div class="notification-title">
                            {item.title.clone()}
                        </div>
                        <div class="notification-content">
                            {item.content.clone()}
                        </div>
                    </div>
                }
            })
            .collect::<Html>();

        html! {
            <ContextProvider<Rc<FriendShipState>> context={self.friend_ship_state.clone()}>
            <ContextProvider<Rc<FriendListState>> context={self.friend_state.clone()}>
            <ContextProvider<Rc<ConvState>> context={self.conv_state.clone()}>
            <ContextProvider<Rc<NotificationState>> context={self.notification.clone()}>
            <ContextProvider<Rc<CreateConvState>> context={self.create_conv.clone()}>
            <ContextProvider<Rc<AddFriendState>> context={self.add_friend_state.clone()}>
            <ContextProvider<Rc<SendResultState>> context={self.send_result.clone()}>
                <div class="home" id="app">
                    <Left user_id={ctx.props().id.clone()}/>
                    <Right />
                    // 通知组件

                    <div class="notify" ref={self.notification_node.clone()}>
                        {notify}
                    </div>
                </div>
            </ContextProvider<Rc<SendResultState>>>
            </ContextProvider<Rc<AddFriendState>>>
            </ContextProvider<Rc<CreateConvState>>>
            </ContextProvider<Rc<NotificationState>>>
            </ContextProvider<Rc<ConvState>>>
            </ContextProvider<Rc<FriendListState>>>
            </ContextProvider<Rc<FriendShipState>>>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        // 将通知区域向上滚动
        if self.notifications.len() > 2 {
            if let Some(div) = self.notification_node.cast::<HtmlDivElement>() {
                div.set_scroll_top(div.scroll_height());
            }
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        // self.ws.borrow_mut().cleanup();
        log::debug!("home destroy==> delete database");
        // 测试阶段，销毁时删除数据库
        spawn_local(async {
            let _ = Repository::new().await.delete_db().await;
        });
        window().local_storage().unwrap().unwrap().clear().unwrap();
    }
}
