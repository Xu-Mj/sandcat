use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{
    components::left::conv_com::conversations::ChatsMsg,
    db,
    model::{
        conversation::Conversation,
        message::{GroupMsg, Message, Msg, SingleCall, DEFAULT_HELLO_MESSAGE},
        ContentType, RightContentType,
    },
};

use super::Chats;

impl Chats {
    pub fn handle_sent_msg(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        let conv_type = match msg {
            Msg::Group(_) => RightContentType::Group,
            Msg::Single(_) | Msg::SingleCall(_) => RightContentType::Friend,
            _ => RightContentType::Default,
        };
        match msg {
            Msg::Single(mut msg)
            | Msg::Group(GroupMsg::Message(mut msg))
            | Msg::OfflineSync(mut msg) => {
                let conv = Conversation {
                    last_msg: msg.content.clone(),
                    last_msg_time: msg.create_time,
                    last_msg_type: msg.content_type,
                    conv_type,
                    friend_id: msg.friend_id.clone(),
                    unread_count: 1,
                    ..Default::default()
                };
                let is_self = msg.is_self;
                spawn_local(async move {
                    if let Err(err) = db::messages().await.add_message(&mut msg).await {
                        log::error!("{:?}", err);
                    }
                });
                self.operate_msg(ctx, conv, is_self)
            }
            Msg::Group(group_msg) => {
                match group_msg {
                    GroupMsg::Invitation(msg) => {
                        self.handle_group_invitation(ctx, msg);
                    }
                    GroupMsg::Dismiss(group_id) => {
                        self.handle_group_dismiss(ctx, group_id);
                    }
                    // don't handle it now
                    _ => {}
                }
                false
            }
            Msg::SingleCall(msg) => self.handle_single_call_conv(ctx, msg, conv_type),
            _ => false,
        }
    }

    /// handle the message of single call for the conversation list
    pub fn handle_single_call_conv(
        &mut self,
        ctx: &Context<Self>,
        msg: SingleCall,
        conv_type: RightContentType,
    ) -> bool {
        match msg {
            SingleCall::Invite(msg) => {
                let mut conv = Conversation::from(msg);
                conv.unread_count = 1;
                conv.conv_type = conv_type;
                self.operate_msg(ctx, conv, false)
            }
            SingleCall::InviteCancel(msg) => {
                let is_self = msg.is_self;
                let mut conv = Conversation::from(msg);
                conv.unread_count = 1;
                conv.conv_type = conv_type;
                self.operate_msg(ctx, conv, is_self)
            }
            SingleCall::NotAnswer(msg) => {
                let is_self = msg.is_self;
                let mut conv = Conversation::from(msg);
                conv.unread_count = 1;
                conv.conv_type = conv_type;
                self.operate_msg(ctx, conv, is_self)
            }
            SingleCall::HangUp(msg) => {
                let is_self = msg.is_self;
                let mut conv = Conversation::from(msg);
                conv.unread_count = 1;
                conv.conv_type = conv_type;
                self.operate_msg(ctx, conv, is_self)
            }
            SingleCall::InviteAnswer(msg) => {
                let is_self = msg.is_self;
                let mut conv = Conversation::from(msg);
                conv.unread_count = 1;
                conv.conv_type = conv_type;
                self.operate_msg(ctx, conv, is_self)
            }
            _ => false,
        }
    }

    pub fn operate_msg(
        &mut self,
        ctx: &Context<Self>,
        mut conv: Conversation,
        is_self: bool,
    ) -> bool {
        let friend_id = conv.friend_id.clone();
        let mut clean = false;
        let unread_count = conv.unread_count;
        /* // judge the conversation is already the first one
        let is_first = self
            .list
            .first()
            .map_or(false, |first| *first.0 == friend_id);
        if is_first {
            let mut old = self.list.get_mut(&friend_id).unwrap();
            if !old.mute && !is_self && self.conv_state.conv.item_id != friend_id {
                self.unread_state.add_msg_count.emit(unread_count);
            }
            // 这里是因为要直接更新面板上的数据，所以需要处理未读数量
            if friend_id != self.conv_state.conv.item_id {
                old.unread_count += unread_count;
            } else {
                old.unread_count = 0;
                clean = true;
            }
            conv.name = old.name.clone();
            conv.avatar = old.avatar.clone();
            conv.id = old.id;
            conv.unread_count = old.unread_count;
            conv.mute = old.mute;
            // self.list.shift_insert(0, friend_id, conv.clone());
            spawn_local(async move {
                db::convs().await.put_conv(&conv, clean).await.unwrap();
            });
            return true;
        } */

        // not the first one
        let dest = self.list.shift_remove(&friend_id);
        if dest.is_some() {
            let mut old = dest.unwrap();
            // deal with unread message count
            if !old.mute && !is_self && self.conv_state.conv.item_id != friend_id {
                self.unread_state.add_msg_count.emit(unread_count);
            }
            // 这里是因为要直接更新面板上的数据，所以需要处理未读数量
            if friend_id != self.conv_state.conv.item_id {
                old.unread_count += unread_count;
            } else {
                old.unread_count = 0;
                clean = true;
            }
            conv.name = old.name;
            conv.avatar = old.avatar;
            conv.id = old.id;
            conv.unread_count = old.unread_count;
            conv.mute = old.mute;
            self.list.shift_insert(0, friend_id, conv.clone());
            log::debug!("dest: {:?}", self.list);
            spawn_local(async move {
                db::convs().await.put_conv(&conv, clean).await.unwrap();
            });
            true
        } else {
            if !is_self && self.conv_state.conv.item_id != friend_id {
                self.unread_state.add_msg_count.emit(unread_count);
            }
            // 如果会话列表中不存在那么需要新建
            ctx.link().send_future(async move {
                let friend = db::friends().await.get_friend(friend_id.as_str()).await;
                conv.avatar = friend.avatar;
                if let Some(name) = friend.remark {
                    conv.name = name;
                } else {
                    conv.name = friend.name;
                }
                db::convs().await.put_conv(&conv, false).await.unwrap();
                conv.unread_count = unread_count;
                log::debug!("创建会话: {:?}", &conv);
                ChatsMsg::InsertConv(conv)
            });
            false
        }
    }

    pub fn handle_receive_single_call(
        &mut self,
        ctx: &Context<Self>,
        mut message: Msg,
        conv_type: RightContentType,
    ) {
        if let Msg::SingleCall(ref mut msg) = message {
            match msg {
                SingleCall::InviteCancel(m) => {
                    let friend_id = m.send_id.clone();
                    m.send_id = m.friend_id.clone();
                    m.friend_id = friend_id;
                    m.is_self = false;
                    self.handle_single_call_conv(ctx, msg.clone(), conv_type);
                    self.rec_msg_state.notify.emit(message);
                }
                SingleCall::NotAnswer(m) => {
                    let friend_id = m.send_id.clone();
                    m.send_id = m.friend_id.clone();
                    m.friend_id = friend_id;
                    m.is_self = false;
                    self.handle_single_call_conv(ctx, msg.clone(), conv_type);
                    self.rec_msg_state.notify.emit(message);
                }
                SingleCall::InviteAnswer(m) => {
                    let friend_id = m.send_id.clone();
                    m.send_id = m.friend_id.clone();
                    m.friend_id = friend_id;
                    m.is_self = false;
                    self.handle_single_call_conv(ctx, msg.clone(), conv_type);
                    self.rec_msg_state.notify.emit(message);
                }
                SingleCall::HangUp(m) => {
                    let friend_id = m.send_id.clone();
                    m.send_id = m.friend_id.clone();
                    m.friend_id = friend_id;
                    m.is_self = false;
                    self.handle_single_call_conv(ctx, msg.clone(), conv_type);
                    self.rec_msg_state.notify.emit(message);
                }
                _ => {}
            }
        }
    }

    pub fn handle_receive_message(&mut self, ctx: &Context<Self>, mut message: Msg) -> bool {
        let conv_type = match message {
            Msg::Group(_) => RightContentType::Group,
            Msg::Single(_) | Msg::SingleCall(_) => RightContentType::Friend,
            _ => RightContentType::Default,
        };
        match message {
            Msg::Single(ref mut msg) => {
                let friend_id = msg.send_id.clone();
                msg.send_id = msg.friend_id.clone();
                msg.friend_id = friend_id;
                msg.is_read = false;
                msg.is_self = false;

                let conv = Conversation {
                    last_msg: msg.content.clone(),
                    last_msg_time: msg.create_time,
                    last_msg_type: msg.content_type,
                    conv_type,
                    friend_id: msg.friend_id.clone(),
                    unread_count: 1,
                    ..Default::default()
                };
                let is_self = msg.is_self;

                let mut msg = msg.clone();
                // let msg_id = msg.server_id.to_string();

                let is_send = (self.conv_state.conv.content_type == RightContentType::Friend
                    || self.conv_state.conv.content_type == RightContentType::Group)
                    && self.conv_state.conv.item_id == msg.friend_id;
                spawn_local(async move {
                    // ctx.link().send_future(async move {
                    // save to db
                    db::messages().await.add_message(&mut msg).await.unwrap();
                    // ChatsMsg::None
                    // if let Err(err) = db::messages().await.add_message(&mut msg).await {
                    //     HomeMsg::Notification(Notification::error_from_content(
                    //         format!("Internal Error:{:?}", err).into(),
                    //     ))
                    // } else {
                    //     HomeMsg::SendBackMsg(Msg::SingleDeliveredNotice(msg_id))
                    // }
                });

                // notify other components we have received new message
                if is_send {
                    ctx.link().send_message(ChatsMsg::RecMsgNotify(message));
                }
                return self.operate_msg(ctx, conv, is_self);
            }
            Msg::Group(ref group_msg) => {
                match group_msg {
                    GroupMsg::Invitation(msg) => {
                        // receive create group message
                        self.handle_group_invitation(ctx, msg.clone());
                    }
                    GroupMsg::Message(msg) => {
                        let msg = msg.clone();
                        let _msg_id = msg.server_id.to_string();
                        let conv = Conversation {
                            last_msg: msg.content.clone(),
                            last_msg_time: msg.create_time,
                            last_msg_type: msg.content_type,
                            conv_type,
                            friend_id: msg.friend_id.clone(),
                            unread_count: 1,
                            ..Default::default()
                        };
                        let is_self = msg.is_self;
                        // if self.conv_state.conv.item_id != msg.friend_id {
                        //     let conv_state = Rc::make_mut(&mut self.conv_state);
                        //     let _ = current_item::save_conv(&conv_state.conv)
                        //         .map_err(|err| log::error!("save conv fail{:?}", err));
                        // }
                        let is_send = (self.conv_state.conv.content_type
                            == RightContentType::Friend
                            || self.conv_state.conv.content_type == RightContentType::Group)
                            && self.conv_state.conv.item_id == msg.friend_id;
                        ctx.link().send_future(async move {
                            // 数据入库
                            db::group_msgs().await.put(&msg).await.unwrap();
                            ChatsMsg::None
                            // if let Err(err) = db::group_msgs().await.put(&msg).await {
                            //     HomeMsg::Notification(Notification::error_from_content(
                            //         format!("内部错误:{:?}", err).into(),
                            //     ))
                            // } else {
                            //     HomeMsg::SendBackMsg(Msg::SingleDeliveredNotice(msg_id))
                            // }
                        });

                        if is_send {
                            ctx.link().send_message(ChatsMsg::RecMsgNotify(message));
                        }
                        return self.operate_msg(ctx, conv, is_self);
                    }
                    GroupMsg::MemberExit((mem_id, group_id)) => {
                        // todo modify conversation list
                        // delete member information from da
                        // let user_id = ctx.props().user_id.clone();
                        let mem_id = mem_id.clone();
                        let group_id = group_id.clone();
                        // let ctx = ctx.link().clone();
                        spawn_local(async move {
                            db::group_members()
                                .await
                                .delete(&mem_id, &group_id)
                                .await
                                .unwrap();
                            // if let Err(err) =
                            //     db::group_members().await.delete(&mem_id, &group_id).await
                            // {
                            //     log::error!("remove group member fail:{:?}", err);
                            // } else {
                            //     // send message received
                            //     ctx.send_message(HomeMsg::SendBackMsg(Msg::Group(
                            //         GroupMsg::DismissOrExitReceived((
                            //             user_id.to_string(),
                            //             group_id,
                            //         )),
                            //     )));
                            // }
                        });
                    }
                    GroupMsg::Dismiss(group_id) => {
                        // delete group from db
                        // let user_id = ctx.props().user_id.clone();
                        // we can consume the group_msg here because it is behind in the reference
                        let cloned_group_id = group_id.clone();
                        log::debug!("received dismiss message, group id : {}", group_id);
                        let is_send = (self.conv_state.conv.content_type
                            == RightContentType::Friend
                            || self.conv_state.conv.content_type == RightContentType::Group)
                            && self.conv_state.conv.item_id == group_id;
                        spawn_local(async move {
                            if let Err(err) = db::groups().await.dismiss(&cloned_group_id).await {
                                log::error!("remove group fail:{:?}", err);
                            } else {
                                //     // send message to other component
                                //     ctx.send_message(HomeMsg::RecSendMsgStateChange(message));
                                //     // send message received
                                //     ctx.send_message(HomeMsg::SendBackMsg(Msg::Group(
                                //         GroupMsg::DismissOrExitReceived((
                                //             user_id.to_string(),
                                //             group_id,
                                //         )),
                                //     )));
                            }
                        });

                        self.handle_group_dismiss(ctx, group_id.to_string());
                        if is_send {
                            ctx.link().send_message(ChatsMsg::RecMsgNotify(message));
                        }
                    }
                    GroupMsg::DismissOrExitReceived(_) | GroupMsg::InvitationReceived(_) => {}
                }
            }
            Msg::SendRelationshipReq(_msg) => {}
            Msg::RecRelationship(friendship) => {
                // 收到好友请求
                self.handle_friendship_req(ctx, friendship);
            }
            Msg::ReadNotice(_) | Msg::SingleDeliveredNotice(_) => {}
            Msg::OfflineSync(_) => {}
            Msg::SingleCall(ref m) => {
                // call message is handled by PhoneCall component
                // 保存电话信息，通知phone call组件
                log::debug!("receive message from websocket: {:?}", m);
                self.call_msg = m.clone();
                self.handle_receive_single_call(ctx, message, conv_type);
                return true;
            }
            Msg::FriendshipDeliveredNotice(_) => {}
            Msg::RelationshipRes(friend) => {
                // 收到好友同意消息
                // self.info(AttrValue::from("好友同意"));
                let send_id = ctx.props().user_id.clone();
                // 需要通知联系人列表更新
                // 数据入库
                // let cloned_ctx = ctx.link().clone();
                ctx.link().send_future(async move {
                    db::friendships()
                        .await
                        .agree_by_friend_id(friend.friend_id.as_str())
                        .await;
                    db::friends().await.put_friend(&friend).await;
                    // send received message
                    // cloned_ctx.send_message(HomeMsg::SendBackMsg(Msg::FriendshipDeliveredNotice(
                    // friend.fs_id.to_string(),
                    // )));
                    // send hello message
                    let mut msg = Message {
                        seq: 0,
                        local_id: nanoid::nanoid!().into(),
                        server_id: AttrValue::default(),
                        send_id,
                        friend_id: friend.friend_id.clone(),
                        content_type: ContentType::Text,
                        content: friend
                            .hello
                            .unwrap_or_else(|| AttrValue::from(DEFAULT_HELLO_MESSAGE)),
                        create_time: chrono::Local::now().timestamp_millis(),
                        is_read: true,
                        is_self: true,
                        file_content: AttrValue::default(),
                        id: 0,
                        send_time: 0,
                        is_success: false,
                    };
                    let _ = db::messages()
                        .await
                        .add_message(&mut msg)
                        .await
                        .map_err(|err| log::error!("save message fail:{:?}", err));
                    ChatsMsg::SendMessage(Msg::Single(msg))
                });
            }
            Msg::ServerRecResp(_msg) => {
                // need to use the local to mark the message as send-success
                // log::debug!("receive server response: {:?}", msg);
            }
        }
        false
    }
}
