use std::rc::Rc;

use yew::{AttrValue, Context, NodeRef};
use yewdux::Dispatch;

use crate::db;
use crate::model::message::SendStatus;
use crate::pages::SendResultState;
use crate::state::I18nState;
use crate::{
    db::{current_item, QueryError, QueryStatus, DB_NAME},
    model::{
        friend::Friend,
        message::{InviteMsg, Message, Msg, DEFAULT_HELLO_MESSAGE},
        notification::{Notification, NotificationState, NotificationType},
        ContentType, CurrentItem, FriendShipStateType,
    },
    pages::{
        home_page::HomeMsg, AddFriendState, ConvState, CreateConvState, FriendListState,
        FriendShipState, RecSendCallState, RemoveFriendState, SendMessageState,
    },
};

use super::{Home, QueryResult};

async fn query(id: &str) -> Result<QueryResult, QueryError> {
    let user_repo = db::users().await;
    let user = user_repo.get(id).await.unwrap();

    Ok((
        user,
        current_item::get_conv(),
        current_item::get_friend(),
        current_item::get_com_type(),
    ))
}

impl Home {
    pub fn new(ctx: &Context<Self>) -> Self {
        // 测试数据库
        // 查询当前登录用户放到登录中
        let id = ctx.props().id.clone();
        // 每次创建Home组件时，检查一下数据库名是否存在，不存在则创建
        // 这样就能保证每次创建Home组件时，数据库名都是当前登录用户的id
        DB_NAME.get_or_init(|| format!("im-{}", id));
        let clone_id = id.clone();
        ctx.link().send_future(async move {
            match query(clone_id.as_str()).await {
                Ok(data) => HomeMsg::Query(QueryStatus::QuerySuccess(data)),
                Err(err) => HomeMsg::Query(QueryStatus::QueryFail(err)),
            }
        });

        // 使用ctx发送一个正在查询的状态
        ctx.link()
            .send_message(HomeMsg::Query(QueryStatus::Querying));
        let switch_friend_callback = ctx.link().callback(HomeMsg::SwitchFriend);
        let switch_conv_callback = ctx.link().callback(HomeMsg::SwitchConv);
        let remove_event = ctx.link().callback(HomeMsg::RemoveFriend);
        let rec_msg_event = ctx.link().callback(HomeMsg::SendMsgStateChange);
        // let rec_listener = ctx.link().callback(HomeMsg::ReceiveMessage);
        let send_msg_event = ctx.link().callback(HomeMsg::SendMessage);
        // let send_back_event = ctx.link().callback(HomeMsg::SendBackMsg);
        let call_event = ctx.link().callback(HomeMsg::SendCallInvite);
        let rec_friend_req_event = ctx.link().callback(HomeMsg::ReceiveFriendShipReq);
        let rec_friend_res_event = ctx.link().callback(HomeMsg::FriendShipResponse);
        let rec_resp = ctx.link().callback(HomeMsg::RecFsResp);
        let error_event = ctx.link().callback(HomeMsg::Notification);
        let create_friend_conv = ctx.link().callback(HomeMsg::CreateFriendConv);
        let create_group_conv = ctx.link().callback(HomeMsg::CreateGroupConv);
        let add = ctx.link().callback(HomeMsg::AddFriendStateChange);
        let send_result = ctx.link().callback(HomeMsg::SendResultState);
        // change lang state
        Dispatch::<I18nState>::global()
            .reduce_mut(|state| state.lang = current_item::get_language());
        Self {
            send_msg_state: Rc::new(SendMessageState {
                msg: Msg::Single(Message::default()),
                // send_back_event,
                send_msg_event: send_msg_event.clone(),
                call_event: call_event.clone(),
            }),
            conv_state: Rc::new(ConvState {
                conv: CurrentItem::default(),
                state_change_event: switch_conv_callback,
            }),
            // ws,
            friend_ship_state: Rc::new(FriendShipState {
                ship: None,
                friend: None,
                state_type: FriendShipStateType::Req,
                req_change_event: rec_friend_req_event,
                res_change_event: rec_friend_res_event,
                rec_resp,
            }),
            friend_state: Rc::new(FriendListState {
                friend: Default::default(),
                state_change_event: switch_friend_callback,
            }),
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
            remove_friend_state: Rc::new(RemoveFriendState::with_event(remove_event)),
            create_conv: Rc::new(CreateConvState {
                friend: None,
                group: None,
                type_: crate::model::RightContentType::Default,
                create_friend: create_friend_conv,
                create_group: create_group_conv,
            }),

            add_friend_state: Rc::new(AddFriendState {
                add,
                ..Default::default()
            }),

            send_result: Rc::new(SendResultState {
                notify: send_result,
                ..Default::default()
            }),
        }
    }

    pub fn info(&mut self, value: AttrValue) {
        self.notifications.push(Notification {
            type_: NotificationType::Info,
            title: AttrValue::from("INFO"),
            content: value,
        });
    }

    pub fn warn(&mut self, value: AttrValue) {
        self.notifications.push(Notification {
            type_: NotificationType::Info,
            title: AttrValue::from("WARN"),
            content: value,
        });
    }

    pub fn error(&mut self, value: AttrValue) {
        self.notifications.push(Notification {
            type_: NotificationType::Error,
            title: AttrValue::from("ERROR"),
            content: value,
        });
    }

    pub fn notify(&mut self, notify: Notification) {
        match notify.type_ {
            NotificationType::Info => self.info(notify.content),
            // NotificationType::Success => {}
            NotificationType::Warn => self.warn(notify.content),
            NotificationType::Error => self.error(notify.content),
        }
    }

    /// agree friend request from frienship list component
    pub fn agree_friendship(
        &mut self,
        ctx: &Context<Self>,
        friendship_id: AttrValue,
        friend: Friend,
    ) -> bool {
        log::debug!("同意好友添加请求消息:{:?}", &friend);
        let state = Rc::make_mut(&mut self.friend_ship_state);
        state.friend = Some(friend.clone());
        state.state_type = FriendShipStateType::Res;

        let send_id = ctx.props().id.clone();
        // 入库
        ctx.link().send_future(async move {
            db::friendships().await.agree(friendship_id.as_str()).await;
            db::friends().await.put_friend(&friend).await;
            let mut msg = Message {
                seq: 0,
                local_id: nanoid::nanoid!().into(),
                server_id: AttrValue::default(),
                send_id,
                friend_id: friend.friend_id.clone(),
                content_type: ContentType::Text,
                content: friend
                    .hello
                    .clone()
                    .unwrap_or_else(|| AttrValue::from(DEFAULT_HELLO_MESSAGE)),
                create_time: chrono::Local::now().timestamp_millis(),
                is_read: true,
                is_self: true,
                file_content: AttrValue::default(),
                id: 0,
                send_time: 0,
                send_status: SendStatus::Sending,
            };
            let _ = db::messages()
                .await
                .add_message(&mut msg)
                .await
                .map_err(|err| log::error!("添加好友打招呼消息入库失败:{:?}", err));
            log::debug!("发送打招呼:{:?}", &msg);
            HomeMsg::SendMessage(Msg::Single(msg))
        });
        true
    }
}
