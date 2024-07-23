use log::error;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yewdux::Dispatch;

use sandcat_sdk::{
    api, db,
    error::Error,
    model::{
        conversation::Conversation,
        friend::FriendStatus,
        group::GroupMember,
        message::{GroupMsg, Msg, RespMsgType, SingleCall},
        notification::Notification,
        voice::Voice,
        ContentType, FriendShipStateType, RightContentType,
    },
    pb::message::Msg as PbMsg,
    state::{
        AudioDownloadedState, CreateConvState, FriendShipState, SendMessageState, SendResultState,
        UnreadState,
    },
};

use super::Chats;

use crate::left::conv_com::conversations::ChatsMsg;

/// handle send and receive message
impl Chats {
    /// used to update the conversation list when a message is sent
    pub fn handle_sent_msg(&mut self, ctx: &Context<Self>, msg: &Msg) -> bool {
        log::debug!("handle_sent_msg:{:?}", msg);
        let conv_type = match msg {
            Msg::Group(_) => RightContentType::Group,
            Msg::Single(_) | Msg::SingleCall(_) => RightContentType::Friend,
            _ => RightContentType::Default,
        };
        match msg {
            Msg::Single(msg) | Msg::Group(GroupMsg::Message(msg)) | Msg::OfflineSync(msg) => {
                let conv = Conversation {
                    last_msg: msg.content.clone(),
                    last_msg_time: msg.create_time,
                    last_msg_type: msg.content_type,
                    last_msg_is_self: msg.is_self,
                    conv_type,
                    friend_id: msg.friend_id.clone(),
                    unread_count: 0,
                    ..Default::default()
                };
                if !msg.is_resend {
                    return self.handle_conv(ctx, conv);
                }
                false
            }
            Msg::Group(group_msg) => {
                match group_msg {
                    GroupMsg::Invitation((msg, _)) => {
                        let msg = msg.clone();
                        let ctx = ctx.link().clone();
                        spawn_local(async move {
                            Self::handle_group_invitation(ctx, msg).await;
                        });
                    }
                    GroupMsg::Dismiss((group_id, _)) => {
                        let group_id = group_id.clone();
                        spawn_local(async move {
                            // todo update conversation
                            Self::dismiss_group(group_id).await.unwrap();
                        });
                    }
                    // don't handle it now
                    _ => {}
                }
                false
            }
            Msg::SingleCall(msg) => {
                Dispatch::<SendMessageState>::global().set(SendMessageState {
                    msg: Msg::SingleCall(msg.clone()),
                });
                self.handle_single_call_conv(ctx, msg.clone(), conv_type)
            }
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
        let conv = match msg {
            SingleCall::Invite(msg) => {
                let mut conv = Conversation::from(msg);
                conv.conv_type = conv_type;
                conv.last_msg_is_self = true;
                conv
            }
            SingleCall::InviteCancel(msg) => {
                let mut conv = Conversation::from(msg);
                conv.conv_type = conv_type;
                conv
                // self.handle_conv(ctx, conv, is_self)
            }
            SingleCall::NotAnswer(msg) => {
                let mut conv = Conversation::from(msg);
                conv.conv_type = conv_type;
                conv
            }
            SingleCall::HangUp(msg) => {
                let mut conv = Conversation::from(msg);
                conv.conv_type = conv_type;
                conv
            }
            SingleCall::InviteAnswer(msg) => {
                let mut conv = Conversation::from(msg);
                conv.conv_type = conv_type;
                conv
            }
            _ => return false,
        };
        self.handle_conv(ctx, conv)
    }

    fn incr_unread_count(conv: &Conversation, current_id: &AttrValue) {
        if !conv.mute && !conv.last_msg_is_self && *current_id != conv.friend_id {
            Dispatch::<UnreadState>::global()
                .reduce_mut(|s| s.msg_count = s.msg_count.saturating_add(conv.unread_count));
        }
    }

    fn save_conversation_and_update_friend_info(
        mut conv: Conversation,
        new_name: AttrValue,
        need_update_friend_info: bool,
        clean: bool,
    ) {
        spawn_local(async move {
            if clean {
                conv.unread_count = 0;
            }
            if let Err(e) = db::db_ins().convs.put_conv(&conv).await {
                error!("put conv error:{:?}", e);
                Notification::error("put conv error").notify();
            }

            if need_update_friend_info && conv.conv_type == RightContentType::Friend {
                if let Err(e) = db::db_ins()
                    .friends
                    .update_friend_avatar_nickname(&conv.friend_id, conv.avatar.clone(), new_name)
                    .await
                {
                    error!("update friend info error:{:?}", e);
                }
            }
        });
    }

    fn create_new_conversation(
        ctx: &Context<Self>,
        mut conv: Conversation,
        friend_id: AttrValue,
        current_id: AttrValue,
    ) {
        let ctx = ctx.link().clone();
        spawn_local(async move {
            let friend = db::db_ins().friends.get(&friend_id).await;
            if friend.friend_id.is_empty() {
                return;
            }

            conv.avatar = friend.avatar;
            conv.name = friend.name;
            conv.remark = friend.remark;

            if let Err(e) = db::db_ins().convs.put_conv(&conv).await {
                error!("failed to update conv: {:?}", e);
                Notification::error("update conv error").notify();
                return;
            }
            Self::incr_unread_count(&conv, &current_id);

            log::debug!("create conversation: {:?}", &conv);
            ctx.send_message(ChatsMsg::InsertConv(conv));
        });
    }

    fn update_old_conv(&mut self, old: Conversation, mut conv: Conversation) -> bool {
        let mut clean = false;
        let friend_id = conv.friend_id.clone();
        let current_id = self.conv_state.conv.item_id.clone();

        // handle unread message count
        if friend_id != *current_id {
            conv.unread_count += old.unread_count;
        } else {
            conv.unread_count = 0;
            clean = true;
        }

        let new_name = conv.name.clone();
        let mut need_update_friend_info = false;

        if conv.avatar.is_empty() || conv.conv_type != RightContentType::Friend {
            conv.avatar = old.avatar;
        } else if conv.avatar != old.avatar || conv.name != old.name {
            need_update_friend_info = true;
        }

        conv.name = old.name;
        conv.mute = old.mute;
        conv.is_pined = old.is_pined;

        if conv.is_pinned() {
            self.pinned_list
                .shift_insert(0, friend_id.clone(), conv.clone());
        } else {
            self.list.shift_insert(0, friend_id.clone(), conv.clone());
        }

        Self::incr_unread_count(&conv, &current_id);
        Self::save_conversation_and_update_friend_info(
            conv,
            new_name,
            need_update_friend_info,
            clean,
        );
        true
    }

    pub fn handle_conv(&mut self, ctx: &Context<Self>, conv: Conversation) -> bool {
        let friend_id = conv.friend_id.clone();
        let current_id = self.conv_state.conv.item_id.clone();
        // query pinned list first
        if let Some(old) = self.pinned_list.shift_remove(&friend_id) {
            return self.update_old_conv(old, conv);
        }

        if let Some(old) = self.list.shift_remove(&friend_id) {
            self.update_old_conv(old, conv)
        } else {
            Self::create_new_conversation(ctx, conv, friend_id, current_id);
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
                    self.handle_rec_lack_msg(ctx, m.seq);
                    self.handle_single_call_conv(ctx, msg.clone(), conv_type);
                    self.rec_msg_dis.reduce_mut(|s| s.msg = message);
                }
                SingleCall::NotAnswer(m) => {
                    let friend_id = m.send_id.clone();
                    m.send_id = m.friend_id.clone();
                    m.friend_id = friend_id;
                    m.is_self = false;
                    self.handle_rec_lack_msg(ctx, m.seq);
                    self.handle_single_call_conv(ctx, msg.clone(), conv_type);
                    self.rec_msg_dis.reduce_mut(|s| s.msg = message);
                }
                SingleCall::InviteAnswer(m) => {
                    let friend_id = m.send_id.clone();
                    m.send_id = m.friend_id.clone();
                    m.friend_id = friend_id;
                    m.is_self = false;
                    self.handle_rec_lack_msg(ctx, m.seq);
                    self.handle_single_call_conv(ctx, msg.clone(), conv_type);
                    self.rec_msg_dis.reduce_mut(|s| s.msg = message);
                }
                SingleCall::HangUp(m) => {
                    let friend_id = m.send_id.clone();
                    m.send_id = m.friend_id.clone();
                    m.friend_id = friend_id;
                    m.is_self = false;
                    self.handle_rec_lack_msg(ctx, m.seq);
                    self.handle_single_call_conv(ctx, msg.clone(), conv_type);
                    self.rec_msg_dis.reduce_mut(|s| s.msg = message);
                }
                _ => {}
            }
        }
    }

    pub fn handle_rec_lack_msg(&mut self, ctx: &Context<Self>, end: i64) {
        self.handle_lack_msg(ctx, end, false);
    }

    pub fn handle_send_lack_msg(&mut self, ctx: &Context<Self>, end: i64) {
        self.handle_lack_msg(ctx, end, true);
    }

    pub fn handle_lack_msg(&mut self, ctx: &Context<Self>, end: i64, is_send: bool) {
        if (!is_send && self.seq.local_seq > end - 1) || (is_send && self.seq.send_seq > end - 1) {
            return;
        }

        let (need_repull, start, other_seq) = if is_send {
            (
                self.seq.send_seq < end - 1,
                self.seq.local_seq,
                self.seq.send_seq,
            )
        } else {
            (
                self.seq.local_seq < end - 1,
                self.seq.send_seq,
                self.seq.local_seq,
            )
        };

        let user_id = ctx.props().user_id.clone();

        if is_send {
            self.seq.send_seq = end;
        } else {
            self.seq.local_seq = end;
        }
        let seq = self.seq.clone();

        let ctx = ctx.link().clone();
        spawn_local(async move {
            if let Err(e) = db::db_ins().seq.put(&seq).await {
                error!("save seq error: {:?}", e);
                Notification::error("save seq error").notify();
                return;
            }

            if need_repull {
                match Self::pull_offline_msgs(&user_id, is_send, other_seq, start, end).await {
                    Ok(messages) => ctx.send_message(ChatsMsg::HandleLackMessages(
                        Self::handle_offline_messages(ctx.clone(), user_id, messages).await,
                    )),
                    Err(e) => {
                        error!("pull offline msg error: {:?}", e);
                        Notification::error("pull offline msg error").notify();
                    }
                }
            }
        });
    }

    async fn pull_offline_msgs(
        user_id: &str,
        is_send: bool,
        send_seq: i64,
        start: i64,
        end: i64,
    ) -> Result<Vec<PbMsg>, Error> {
        let (send_start, send_end, rec_start, rec_end) = if is_send {
            (start, end, send_seq, send_seq)
        } else {
            (send_seq, send_seq, start, end)
        };
        api::messages()
            .pull_offline_msg(user_id, send_start, send_end, rec_start, rec_end)
            .await
    }

    pub async fn download_voice_and_save(
        url: &str,
        local_id: &str,
        duration: u8,
    ) -> Result<(), Error> {
        // request from file server
        let data = api::file().download_voice(url).await?;

        let voice = Voice::new(local_id.to_string(), data, duration);
        db::db_ins().voices.save(&voice).await?;

        Dispatch::<AudioDownloadedState>::global()
            .reduce_mut(|s| s.local_id = local_id.to_string().into());
        Ok(())
    }

    pub fn handle_receive_message(&mut self, ctx: &Context<Self>, message: Msg) -> bool {
        let conv_type = match message {
            Msg::Group(_) => RightContentType::Group,
            Msg::Single(_) | Msg::SingleCall(_) => RightContentType::Friend,
            _ => RightContentType::Default,
        };

        match message {
            Msg::Single(mut msg) => {
                // if the message is self message, then we don't need to swap the send_id and friend_id
                if msg.send_id == ctx.props().user_id {
                    msg.is_self = true;
                } else {
                    let friend_id = msg.send_id.clone();
                    msg.send_id = msg.friend_id.clone();
                    msg.friend_id = friend_id;
                    msg.is_read = 0;
                    msg.is_self = false;
                }

                let mut conv = Conversation::from(msg.clone());
                conv.conv_type = conv_type;

                let is_send = (self.conv_state.conv.content_type == RightContentType::Friend
                    || self.conv_state.conv.content_type == RightContentType::Group)
                    && self.conv_state.conv.item_id == msg.friend_id;

                self.handle_rec_lack_msg(ctx, msg.seq);
                let scope = ctx.link().clone();
                spawn_local(async move {
                    // split audio data
                    if msg.content_type == ContentType::Audio {
                        // request from file server
                        if let Err(e) = Self::download_voice_and_save(
                            &msg.content,
                            &msg.local_id,
                            msg.audio_duration,
                        )
                        .await
                        {
                            Notification::error(e).notify();
                        }
                        msg.audio_downloaded = true;
                    }
                    // save to db
                    if let Err(e) = db::db_ins().messages.add_message(&mut msg).await {
                        error!("save message to db error: {:?}", e);
                        Notification::error("save message to db error").notify();
                    }

                    // notify other components we have received new message
                    if is_send {
                        scope.send_message(ChatsMsg::RecMsgNotify(Msg::Single(msg)));
                    }
                });

                return self.handle_conv(ctx, conv);
            }
            Msg::Group(group_msg) => {
                match group_msg {
                    GroupMsg::Invitation((msg, seq)) => {
                        // receive create group message
                        self.handle_rec_lack_msg(ctx, seq);
                        let ctx = ctx.link().clone();
                        spawn_local(async move { Self::handle_group_invitation(ctx, msg).await });
                    }
                    GroupMsg::InviteNew((_user_id, members, seq)) => {
                        self.handle_rec_lack_msg(ctx, seq);
                        spawn_local(async move {
                            if let Err(e) = db::db_ins()
                                .group_members
                                .put_list(
                                    members.members.into_iter().map(GroupMember::from).collect(),
                                )
                                .await
                            {
                                error!("save group member error: {:?}", e);
                                Notification::error("Failed to store group member").notify();
                            }
                        });
                    }
                    GroupMsg::Message(mut msg) => {
                        let mut conv = Conversation::from(msg.clone());
                        conv.conv_type = conv_type;

                        let is_send = (self.conv_state.conv.content_type
                            == RightContentType::Friend
                            || self.conv_state.conv.content_type == RightContentType::Group)
                            && self.conv_state.conv.item_id == msg.friend_id;

                        self.handle_rec_lack_msg(ctx, msg.seq);
                        let scope = ctx.link().clone();
                        spawn_local(async move {
                            if msg.content_type == ContentType::Audio {
                                // request from file server
                                if let Err(e) = Self::download_voice_and_save(
                                    &msg.content,
                                    &msg.local_id,
                                    msg.audio_duration,
                                )
                                .await
                                {
                                    Notification::error(e).notify();
                                }
                                msg.audio_downloaded = true;
                            }
                            if let Err(e) = db::db_ins().group_msgs.put(&msg).await {
                                error!("save message to db error: {:?}", e);
                                Notification::error("save message to db error").notify();
                            }
                            if is_send {
                                scope.send_message(ChatsMsg::RecMsgNotify(Msg::Group(
                                    GroupMsg::Message(msg),
                                )));
                            }
                        });

                        return self.handle_conv(ctx, conv);
                    }
                    GroupMsg::MemberExit((mem_id, group_id, seq)) => {
                        self.handle_rec_lack_msg(ctx, seq);
                        // todo modify conversation list
                        // delete member information from da
                        let mem_id = mem_id.clone();
                        let group_id = group_id.clone();
                        spawn_local(async move {
                            log::debug!(
                                "received group member exits message {group_id} --> {mem_id}, delete member from group"
                            );
                            if let Err(e) =
                                db::db_ins().group_members.delete(&group_id, &mem_id).await
                            {
                                error!("delete members error: {:?}", e);
                                Notification::error("delete members error").notify();
                            }
                        });
                    }
                    GroupMsg::Dismiss((group_id, seq)) => {
                        self.handle_rec_lack_msg(ctx, seq);
                        // delete group from db
                        // let user_id = ctx.props().user_id.clone();
                        // we can consume the group_msg here because it is behind in the reference
                        log::debug!("received dismiss message, group id : {}", group_id);
                        let is_send = (self.conv_state.conv.content_type
                            == RightContentType::Friend
                            || self.conv_state.conv.content_type == RightContentType::Group)
                            && self.conv_state.conv.item_id == group_id;

                        let scope = ctx.link().clone();
                        spawn_local(async move {
                            if let Err(err) = db::db_ins().groups.dismiss(&group_id).await {
                                log::error!("remove group fail:{:?}", err);
                                Notification::error("remove group error").notify();
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
                            // self.handle_group_dismiss(ctx, group_id.to_string());
                            Self::dismiss_group(group_id.clone()).await.unwrap();
                            if is_send {
                                scope.send_message(ChatsMsg::RecMsgNotify(Msg::Group(
                                    GroupMsg::Dismiss((group_id, seq)),
                                )));
                            }
                        });
                    }
                    GroupMsg::Update((group, seq)) => {
                        self.handle_rec_lack_msg(ctx, seq);

                        spawn_local(async move { Self::handle_group_update(group).await });

                        // todo send message received
                    }
                    GroupMsg::DismissOrExitReceived(_) | GroupMsg::InvitationReceived(_) => {}
                }
            }
            Msg::SendRelationshipReq(_msg) => {}
            Msg::RecRelationship((friendship, seq)) => {
                // 收到好友请求
                log::debug!("ReceiveFriendShipReq:{:?}", &friendship);
                // handle sequence
                self.handle_rec_lack_msg(ctx, seq);

                // save friendship
                spawn_local(async move {
                    if let Err(err) = db::db_ins().friendships.put_friendship(&friendship).await {
                        error!("save friendship error:{:?}", err);
                        return;
                    };
                    // notify
                    Dispatch::<FriendShipState>::global().reduce_mut(|s| {
                        s.ship = Some(friendship);
                        s.friend = None;
                        s.state_type = FriendShipStateType::Req;
                    });
                });
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
            Msg::RelationshipRes((friend, seq)) => {
                self.handle_rec_lack_msg(ctx, seq);
                // 收到好友同意消息
                // let send_id = ctx.props().user_id.clone();
                // 需要通知联系人列表更新
                // 数据入库
                spawn_local(async move {
                    if let Err(err) = db::db_ins()
                        .friendships
                        .agree_by_friend_id(friend.friend_id.as_str())
                        .await
                    {
                        error!("agree friendship error:{:?}", err);
                        return;
                    }
                    if let Err(err) = db::db_ins().friends.put_friend(&friend).await {
                        error!("save friend error:{:?}", err);
                        return;
                    }
                    let mut conv = Conversation::from(friend.clone());
                    conv.last_msg = AttrValue::from("new friend");
                    conv.last_msg_type = ContentType::Text;
                    conv.last_msg_time = chrono::Utc::now().timestamp_millis();
                    if let Err(e) = db::db_ins().convs.put_conv(&conv).await {
                        error!("save new conversation error: {:?}", e);
                        return;
                    }
                    CreateConvState::update(conv);

                    // send message to contact component to update the friend list
                    Dispatch::<FriendShipState>::global().reduce_mut(|s| {
                        s.friend = Some(friend);
                        s.ship = None;
                        s.state_type = FriendShipStateType::RecResp;
                    });
                });
            }
            Msg::ServerRecResp(msg) => {
                // need to use the local to mark the message as send-success
                // log::debug!("receive server response: {:?}", msg);
                // handle send sequence
                let send_seq = msg.send_seq;
                // update database
                spawn_local(async move {
                    match msg.resp_msg_type {
                        RespMsgType::Single => {
                            if let Err(err) = db::db_ins().messages.update_msg_status(&msg).await {
                                log::error!("update message fail:{:?}", err);
                                Notification::error("update message fail").notify();
                            }
                        }
                        RespMsgType::Group => {
                            if let Err(err) = db::db_ins().group_msgs.update_msg_status(&msg).await
                            {
                                log::error!("update message fail:{:?}", err);
                                Notification::error("update message fail").notify();
                            }
                        }
                    }
                    Dispatch::<SendResultState>::global().reduce_mut(|s| s.msg = msg);
                });
                self.handle_send_lack_msg(ctx, send_seq);
            }
            Msg::RecRelationshipDel((friend_id, seq)) => {
                // update database
                spawn_local(async move {
                    let mut friend = db::db_ins().friends.get(&friend_id).await;
                    if !friend.friend_id.is_empty() {
                        friend.status = FriendStatus::Deleted as i32;
                        if let Err(err) = db::db_ins().friends.put_friend(&friend).await {
                            error!("save friend error:{:?}", err);
                        }
                    }
                });
                self.handle_rec_lack_msg(ctx, seq);
            }
        }
        false
    }
}
