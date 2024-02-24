#![allow(dead_code)]
#![allow(unused_variables)]

use crate::components::phone_call::PhoneCall;
use crate::db::friend::FriendRepo;
use crate::db::friend_ship::FriendShipRepo;
use crate::db::{current_item, TOKEN, WS_ADDR};
use crate::icons::CloseIcon;
use crate::model::friend::{Friend, FriendShipWithUser, ItemInfo};
use crate::model::message::{DeliveredNotice, InviteMsg, Message, Msg, DEFAULT_HELLO_MESSAGE};
use crate::model::notification::{Notification, NotificationState, NotificationType};
use crate::model::user::User;
use crate::model::ContentType;
use crate::ws::ws::WebSocketManager;
use crate::{
    components::{left::Left, right::Right},
    db::{message::MessageRepo, user::UserRepo, QueryError, QueryStatus, DB_NAME},
};
use gloo::timers::callback::{Interval, Timeout};
use gloo::utils::window;
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::HtmlDivElement;
use yew::prelude::*;

use super::{
    AppState, ComponentType, ConvState, CurrentItem, FriendListState, FriendShipState,
    FriendShipStateType, RecSendCallState, RecSendMessageState, WaitState,
};

pub struct Home {
    state: Rc<AppState>,
    msg_state: Rc<RecSendMessageState>,
    call_state: Rc<RecSendCallState>,
    // 音视频电话相关的message，通过这个状态给phone call 组件发送消息
    call_msg: Msg,
    conv_state: Rc<ConvState>,
    friend_state: Rc<FriendListState>,
    // user_state: QueryStatus<QueryResult>,
    user: User,
    ws: Rc<RefCell<WebSocketManager>>,
    friend_ship_state: Rc<FriendShipState>,
    show_notify: bool,
    call_friend_info: Option<Box<dyn ItemInfo>>,
    call_timer: Option<Timeout>,
    notifications: Vec<Notification>,
    notification: Rc<NotificationState>,
    notification_node: NodeRef,
    notification_interval: Option<Interval>,
    // need_wait_count: usize,
    wait_state: Rc<WaitState>,
}
const WAIT_COUNT: usize = 1;
pub enum HomeMsg {
    // 全局组件切换
    SwitchComponent(ComponentType),
    // 联系人列表选中元素改变
    SwitchFriend(CurrentItem),
    // 会话列表选中元素改变
    SwitchConv(CurrentItem),
    // 需要等待子组件完成必须操作
    WaitStateChanged,
    // 查询数据库
    Query(QueryStatus<QueryResult>),
    // 接收/发送消息
    RecSendMsgStateChange(Msg),
    // 收到消息
    ReceiveMessage(Msg),
    // 收到好友请求
    ReceiveFriendShipReq(FriendShipWithUser),
    // 回复好友请求
    FriendShipResponse((AttrValue, Friend)),
    ReceiveFriendShipRes(Friend),
    // 显示顶部消息通知
    ShowNotify(Box<dyn ItemInfo>),
    // 发送视频电话请求
    SendCallInvite(InviteMsg),
    // 发送消息
    SendMessage(Msg),
    // 发送消息收到
    SendBackMsg(Msg),
    Notification(Notification),
    CleanNotification,
    CloseNotificationByIndex(usize),
    RecSendCallStateChange(Msg),
}

#[derive(Properties, Clone, PartialEq)]
pub struct HomeProps {
    pub id: AttrValue,
}

type QueryResult = (User, CurrentItem, CurrentItem, ComponentType);

async fn query(id: AttrValue) -> Result<QueryResult, QueryError> {
    let user_repo = UserRepo::new().await;
    let user = user_repo.get(id).await.unwrap();
    Ok((
        user,
        current_item::get_conv(),
        current_item::get_friend(),
        current_item::get_com_type(),
    ))
}

impl Home {
    fn send_msg(&self, msg: &Msg) {
        // 发送已收到消息给服务器
        match self
            .ws
            .borrow()
            .send_message(&serde_json::to_string(&msg).unwrap())
        {
            Ok(_) => {
                log::info!("发送成功")
            }
            Err(e) => {
                log::error!("发送失败: {:?}", e)
            }
        };
    }

    fn info(&mut self, value: AttrValue) {
        self.notifications.push(Notification {
            type_: NotificationType::Info,
            title: AttrValue::from("INFO"),
            content: value,
        });
    }

    fn warn(&mut self, value: AttrValue) {
        self.notifications.push(Notification {
            type_: NotificationType::Info,
            title: AttrValue::from("WARN"),
            content: value,
        });
    }

    fn error(&mut self, value: AttrValue) {
        self.notifications.push(Notification {
            type_: NotificationType::Error,
            title: AttrValue::from("ERROR"),
            content: value,
        });
    }

    fn notify(&mut self, notify: Notification) {
        match notify.type_ {
            NotificationType::Info => self.info(notify.content),
            NotificationType::Success => {}
            NotificationType::Warn => self.warn(notify.content),
            NotificationType::Error => self.error(notify.content),
        }
    }
}

impl Component for Home {
    type Message = HomeMsg;
    type Properties = HomeProps;

    fn create(ctx: &Context<Self>) -> Self {
        // 测试数据库
        // 查询当前登录用户放到登录中
        let id = ctx.props().id.clone();
        // 每次创建Home组件时，检查一下数据库名是否存在，不存在则创建
        // 这样就能保证每次创建Home组件时，数据库名都是当前登录用户的id
        DB_NAME.get_or_init(|| format!("im-{}", id.clone()));
        let cloned_id = id.clone();
        ctx.link().send_future(async move {
            match query(cloned_id).await {
                Ok(data) => HomeMsg::Query(QueryStatus::QuerySuccess(data)),
                Err(err) => HomeMsg::Query(QueryStatus::QueryFail(err)),
            }
        });

        // 使用ctx发送一个正在查询的状态
        ctx.link()
            .send_message(HomeMsg::Query(QueryStatus::Querying));
        let callback = ctx.link().callback(HomeMsg::SwitchComponent);
        let switch_friend_callback = ctx.link().callback(HomeMsg::SwitchFriend);
        let switch_conv_callback = ctx.link().callback(HomeMsg::SwitchConv);
        let ready = ctx.link().callback(|_| HomeMsg::WaitStateChanged);
        let rec_msg_event = ctx.link().callback(HomeMsg::RecSendMsgStateChange);
        let rec_listener = ctx.link().callback(HomeMsg::ReceiveMessage);
        let send_msg_event = ctx.link().callback(HomeMsg::SendMessage);
        let call_event = ctx.link().callback(HomeMsg::SendCallInvite);
        let rec_friend_req_event = ctx.link().callback(HomeMsg::ReceiveFriendShipReq);
        let rec_friend_res_event = ctx.link().callback(HomeMsg::FriendShipResponse);
        let error_event = ctx.link().callback(HomeMsg::Notification);
        // 不能用这么多unwrap()
        let token = window()
            .local_storage()
            .unwrap()
            .unwrap()
            .get(TOKEN)
            .unwrap()
            .unwrap();
        let addr = window()
            .local_storage()
            .unwrap()
            .unwrap()
            .get(WS_ADDR)
            .unwrap()
            .unwrap();
        let url = format!("{}/{}/conn/{}/{}", addr, id.clone(), token, id);
        let ws = Rc::new(RefCell::new(WebSocketManager::new(url, rec_listener)));
        Self {
            state: Rc::new(AppState {
                component_type: ComponentType::Messages,
                switch_com_event: callback,
                ..Default::default()
            }),
            msg_state: Rc::new(RecSendMessageState {
                msg: Msg::Single(Message::default()),
                send_msg_event: send_msg_event.clone(),
                call_event: call_event.clone(),
            }),
            user: User::default(),
            conv_state: Rc::new(ConvState {
                conv: CurrentItem::default(),
                state_change_event: switch_conv_callback,
            }),
            ws,
            friend_ship_state: Rc::new(FriendShipState {
                ship: None,
                friend: None,
                state_type: FriendShipStateType::Req,
                req_change_event: rec_friend_req_event,
                res_change_event: rec_friend_res_event,
            }),
            friend_state: Rc::new(FriendListState {
                friend: Default::default(),
                state_change_event: switch_friend_callback,
            }),
            show_notify: false,
            call_friend_info: None,
            call_timer: None,
            notifications: vec![],
            notification: Rc::new(NotificationState {
                notify: error_event,
            }),
            notification_node: NodeRef::default(),
            notification_interval: None,
            call_state: Rc::new(RecSendCallState {
                msg: InviteMsg::default(),
                send_msg_event,
                rec_msg_event,
                call_event,
            }),
            call_msg: Msg::default(),
            wait_state: Rc::new(WaitState {
                wait_count: WAIT_COUNT,
                ready,
            }),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            HomeMsg::SwitchComponent(component_type) => {
                let shared_state = Rc::make_mut(&mut self.state);
                if shared_state.component_type == component_type {
                    return false;
                }
                current_item::save_com_type(&component_type).unwrap();
                shared_state.component_type = component_type;
                gloo::console::log!("home state listener");
                // 是否会重新渲染所有子元素？
                true
            }
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
                log::debug!("home switch conv listener:{:?}", conv);
                let conv_state = Rc::make_mut(&mut self.conv_state);
                log::debug!("home switch conv listener:{:?}", conv_state);

                // 如果id没有变化，那么不更新数据库
                if conv_state.conv.item_id == conv.item_id
                    && conv_state.conv.content_type == conv.content_type
                {
                    return false;
                }
                conv_state.conv = conv;
                true
            }
            HomeMsg::Query(status) => {
                let shared_state = Rc::make_mut(&mut self.state);
                match status {
                    QueryStatus::QuerySuccess(ref u) => {
                        shared_state.login_user = u.0.clone();
                        let conv_state = Rc::make_mut(&mut self.conv_state);
                        conv_state.conv = u.1.clone();
                        let friend_state = Rc::make_mut(&mut self.friend_state);
                        friend_state.friend = u.2.clone();
                        self.user = u.0.clone();
                        shared_state.component_type = u.3.clone();
                    }
                    QueryStatus::QueryFail(_) => {
                        gloo::console::log!("query fail")
                    }
                    _ => {}
                }
                // self.user_state = statue;
                true
            }
            HomeMsg::ReceiveMessage(mut message) => {
                match message {
                    Msg::Single(ref mut msg) => {
                        let friend_id = msg.send_id.clone();
                        msg.send_id = msg.friend_id.clone();
                        msg.friend_id = friend_id;
                        msg.is_read = false;

                        let mut msg = msg.clone();
                        let msg_id = msg.msg_id.to_string();
                        if self.conv_state.conv.item_id != msg.friend_id {
                            let conv_state = Rc::make_mut(&mut self.conv_state);
                            conv_state.conv.unread_count =
                                conv_state.conv.unread_count.saturating_add(1);
                            let _ = current_item::save_conv(&conv_state.conv)
                                .map_err(|err| log::error!("save conv fail"));
                        }
                        ctx.link().send_future(async move {
                            // 数据入库
                            if let Err(err) = MessageRepo::new().await.add_message(&mut msg).await {
                                HomeMsg::Notification(Notification::error_from_content(
                                    format!("内部错误:{:?}", err).into(),
                                ))
                            } else {
                                HomeMsg::SendBackMsg(Msg::SingleDeliveredNotice(DeliveredNotice {
                                    msg_id,
                                    create_time: chrono::Local::now().timestamp_millis(),
                                }))
                            }
                        });

                        ctx.link()
                            .send_message(HomeMsg::RecSendMsgStateChange(message));
                    }
                    Msg::Group(_) => {
                        // if self.conv_state.conv.item_id != msg.friend_id {
                        //     let conv_state = Rc::make_mut(&mut self.conv_state);
                        //     conv_state.conv.unread_count =
                        //         conv_state.conv.unread_count.saturating_add(1);
                        //     current_item::save_conv(&conv_state.conv);
                        // }
                    }
                    Msg::SendRelationshipReq(_msg) => {}
                    Msg::RecRelationship(msg) => {
                        // 收到好友请求
                        ctx.link().send_message(HomeMsg::ReceiveFriendShipReq(msg));
                    }
                    Msg::ReadNotice(_) | Msg::SingleDeliveredNotice(_) => {}
                    Msg::OfflineSync(_) => {}
                    Msg::SingleCallOffer(_)
                    | Msg::SingleCallInvite(_)
                    | Msg::SingleCallInviteCancel(_)
                    | Msg::SingleCallNotAnswer(_)
                    | Msg::SingleCallInviteAnswer(_)
                    | Msg::SingleCallAgree(_)
                    | Msg::SingleCallHangUp(_)
                    | Msg::NewIceCandidate(_) => {
                        // 保存电话信息，通知phone call组件
                        self.call_msg = message;
                        return true;
                    }
                    Msg::FriendshipDeliveredNotice(_) => {}
                    Msg::RelationshipRes(friend) => {
                        // 收到好友同意消息
                        self.info(AttrValue::from("好友同意"));
                        let send_id = self.state.login_user.id.clone();
                        // 需要通知联系人列表更新
                        // 数据入库
                        ctx.link().send_future(async move {
                            FriendShipRepo::new()
                                .await
                                .agree_by_friend_id(friend.friend_id.clone())
                                .await;
                            FriendRepo::new().await.put_friend(&friend).await;
                            // 创建一个会话，TODO这里需要一个friendship数据，用来创建打招呼的信息
                            let mut msg = Message {
                                msg_id: nanoid::nanoid!().into(),
                                send_id,
                                friend_id: friend.friend_id.clone(),
                                content_type: ContentType::Text,
                                content: friend.hello.unwrap_or_else(|| AttrValue::from(DEFAULT_HELLO_MESSAGE)),
                                create_time: chrono::Local::now().timestamp_millis(),
                                is_read: true,
                                is_self: true,
                                file_content: AttrValue::default(),
                                id: 0,
                            };
                            let _ = MessageRepo::new()
                                .await
                                .add_message(&mut msg)
                                .await
                                .map_err(|err| log::error!("save message fail:{:?}", err));
                            HomeMsg::SendMessage(Msg::Single(msg))
                        });
                    }
                }
                false
            }
            HomeMsg::SendMessage(msg) => {
                ctx.link()
                    .send_message(HomeMsg::RecSendMsgStateChange(msg.clone()));
                // 发送消息
                self.send_msg(&msg);
                false
            }
            HomeMsg::RecSendMsgStateChange(msg) => {
                log::debug!("RecSendMsgStateChange:{:?}", &msg);
                let conv_state = Rc::make_mut(&mut self.msg_state);
                conv_state.msg = msg;
                true
            }
            HomeMsg::RecSendCallStateChange(msg) => {
                let conv_state = Rc::make_mut(&mut self.msg_state);
                conv_state.msg = msg;
                true
            }
            HomeMsg::SendBackMsg(msg) => {
                // 发送已收到消息给服务器
                self.send_msg(&msg);
                false
            }
            HomeMsg::ReceiveFriendShipReq(friendship) => {
                log::debug!("ReceiveFriendShipReq:{:?}", &friendship);
                let id = friendship.friendship_id.clone().to_string();
                let state = Rc::make_mut(&mut self.friend_ship_state);
                state.ship = Some(friendship.clone());
                state.state_type = FriendShipStateType::Req;
                // 入库
                ctx.link().send_future(async move {
                    FriendShipRepo::new().await.put_friendship(&friendship).await;
                    // 发送收到通知
                    HomeMsg::SendBackMsg(Msg::FriendshipDeliveredNotice(DeliveredNotice {
                        msg_id: id,
                        create_time: chrono::Local::now().timestamp_millis(),
                    }))
                });
                // 显示通知
                // self.info(AttrValue::from("收到好友请求"));
                ctx.link().send_message(HomeMsg::Notification(Notification {
                    type_: NotificationType::Info,
                    title: AttrValue::default(),
                    content: AttrValue::from("收到好友请求"),
                }));
                true
            }
            HomeMsg::FriendShipResponse((friendship_id, friend)) => {
                log::debug!("同意好友添加请求消息:{:?}", &friend);
                let state = Rc::make_mut(&mut self.friend_ship_state);
                state.friend = Some(friend.clone());
                state.state_type = FriendShipStateType::Res;

                let send_id = self.state.login_user.id.clone();
                // 入库
                ctx.link().send_future(async move {
                    FriendShipRepo::new().await.agree(friendship_id).await;
                    FriendRepo::new().await.put_friend(&friend).await;
                    let mut msg = Message {
                        msg_id: nanoid::nanoid!().into(),
                        send_id,
                        friend_id: friend.friend_id.clone(),
                        content_type: ContentType::Text,
                        content: friend.hello.clone().unwrap_or_else(|| AttrValue::from(DEFAULT_HELLO_MESSAGE)),
                        create_time: chrono::Local::now().timestamp_millis(),
                        is_read: true,
                        is_self: true,
                        file_content: AttrValue::default(),
                        id: 0,
                    };
                    let _ = MessageRepo::new()
                        .await
                        .add_message(&mut msg)
                        .await
                        .map_err(|err| log::error!("添加好友打招呼消息入库失败:{:?}", err));
                    log::debug!("发送打招呼:{:?}", &msg);
                    HomeMsg::SendMessage(Msg::Single(msg))
                });
                true
            }
            HomeMsg::ReceiveFriendShipRes(_) => todo!(),

            HomeMsg::ShowNotify(item) => {
                self.call_friend_info = Some(item);
                true
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
                if self.notifications.len() > 0 {
                    self.notifications.remove(0);
                } else {
                    self.notification_interval = None;
                }
                true
            }
            HomeMsg::SendCallInvite(msg) => {
                log::debug!("home rec video call event");
                let conv_state = Rc::make_mut(&mut self.call_state);
                conv_state.msg = msg;
                true
            }
            HomeMsg::WaitStateChanged => {
                log::debug!("wait state changed");
                let state = Rc::make_mut(&mut self.wait_state);
                state.wait_count -= 1;
                if state.wait_count == 0 {
                    // 所有需要等待的组件都完成
                    WebSocketManager::connect(self.ws.clone());
                }
                false
            }
            HomeMsg::CloseNotificationByIndex(index) => {
                if index < self.notifications.len() {
                    self.notifications.remove(index);
                    return true;
                }
                false
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
                    NotificationType::Success => class.push("success"),
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
            <ContextProvider<Rc<AppState>> context={self.state.clone()}>
                <ContextProvider<Rc<RecSendMessageState>> context={self.msg_state.clone()}>
                    <ContextProvider<Rc<FriendShipState>> context={self.friend_ship_state.clone()}>
                        <ContextProvider<Rc<FriendListState>> context={self.friend_state.clone()}>
                            <ContextProvider<Rc<ConvState>> context={self.conv_state.clone()}>
                                <ContextProvider<Rc<NotificationState>> context={self.notification.clone()}>
                                    <ContextProvider<Rc<RecSendCallState>> context={self.call_state.clone()}>
                                        <ContextProvider<Msg> context={self.call_msg.clone()}>
                                            <ContextProvider<Rc<WaitState>> context={self.wait_state.clone()}>
                                                <div class="home" id="app">
                                                    <Left />
                                                    <Right />
                                                    // 通知组件
                                                    <PhoneCall ws={self.ws.clone()} user_id={self.user.id.clone()}/>
                                                    <div class="notify" ref={self.notification_node.clone()}>
                                                        {notify}
                                                    </div>
                                                </div>
                                            </ContextProvider<Rc<WaitState>>>
                                        </ContextProvider<Msg>>
                                    </ContextProvider<Rc<RecSendCallState>>>
                                </ContextProvider<Rc<NotificationState>>>
                            </ContextProvider<Rc<ConvState>>>
                        </ContextProvider<Rc<FriendListState>>>
                    </ContextProvider<Rc<FriendShipState>>>
                </ContextProvider<Rc<RecSendMessageState>>>
            </ContextProvider<Rc<AppState>>>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        // 将通知区域向上滚动
        if self.notifications.len() > 2 {
            if let Some(div) = self.notification_node.cast::<HtmlDivElement>() {
                div.set_scroll_top(div.scroll_height());
            }
        }
    }

    fn destroy(&mut self, ctx: &Context<Self>) {
        self.ws.borrow_mut().cleanup();
        log::debug!("home destroy==> delete database");
        // 测试阶段，销毁时删除数据库
        let _ = window()
            .indexed_db()
            .unwrap()
            .as_ref()
            .unwrap()
            .delete_database(DB_NAME.get().unwrap().as_str());
        window().local_storage().unwrap().unwrap().clear().unwrap();
    }
}
