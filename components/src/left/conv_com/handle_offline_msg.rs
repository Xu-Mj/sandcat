use std::collections::HashMap;

use log::{error, warn};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yewdux::Dispatch;

use sandcat_sdk::{
    api, db,
    model::{
        conversation::Conversation,
        friend::FriendStatus,
        message::{convert_server_msg, GroupMsg, InviteType, Message, Msg, SingleCall},
        notification::Notification,
        ContentType, RightContentType, OFFLINE_TIME,
    },
    pb::message::Msg as PbMsg,
    state::{CreateConvState, RefreshMsgListState},
};

use super::Chats;

impl Chats {
    fn handle_offline_msg_map(
        &self,
        map: &mut HashMap<AttrValue, Conversation>,
        last_msg: AttrValue,
        mut msg: Message,
        conv_type: RightContentType,
    ) {
        let friend_id = msg.send_id.clone();
        msg.send_id = msg.friend_id.clone();
        msg.friend_id = friend_id;

        let conv = Conversation {
            friend_id: msg.friend_id.clone(),
            last_msg,
            last_msg_time: msg.send_time,
            last_msg_type: msg.content_type,
            unread_count: 1,
            conv_type,
            ..Default::default()
        };

        spawn_local(async move {
            if msg.content_type == ContentType::Audio {
                // request from file server
                if let Err(e) =
                    Self::download_voice_and_save(&msg.content, &msg.local_id, msg.audio_duration)
                        .await
                {
                    Notification::error(e).notify();
                }
                msg.audio_downloaded = true;
            }
            if let Err(e) = db::db_ins().messages.add_message(&mut msg).await {
                error!("save message to db error: {:?}", e);
                Notification::error("save message to db  error").notify();
            }
        });

        if let Some(v) = map.get_mut(&conv.friend_id) {
            v.last_msg = conv.last_msg;
            v.last_msg_time = conv.last_msg_time;
            v.last_msg_type = conv.last_msg_type;
            v.unread_count += 1;
        } else {
            map.insert(conv.friend_id.clone(), conv);
        }
    }

    pub fn get_call_content(&self, invite_type: &InviteType) -> AttrValue {
        match invite_type {
            InviteType::Video => AttrValue::from("[视频通话]"),
            InviteType::Audio => AttrValue::from("[语音通话]"),
        }
    }

    // tod handle the friend request and send the group create message to contact
    pub fn handle_offline_messages(&mut self, ctx: &Context<Self>, messages: Vec<PbMsg>) {
        if messages.is_empty() {
            return;
        }
        let mut map: HashMap<AttrValue, Conversation> = HashMap::with_capacity(messages.len());

        for item in messages.into_iter() {
            // let friend_id = item.send_id.clone();
            let msg = match convert_server_msg(item) {
                Ok(msg) => msg,
                Err(e) => {
                    error!("convert_server_msg error: {:?}", e);
                    Notification::error("sconvert_server_msg error").notify();
                    return;
                }
            };
            let conv_type = self.get_msg_type(&msg);
            match msg {
                Msg::Single(msg) => {
                    self.handle_offline_msg_map(&mut map, msg.content.clone(), msg, conv_type);
                }
                Msg::Group(group_msg) => match group_msg {
                    GroupMsg::Invitation((msg, _)) => {
                        self.handle_group_invitation(ctx, msg);
                    }
                    GroupMsg::Dismiss((group_id, _)) => {
                        self.handle_group_dismiss(ctx, group_id);
                    }
                    GroupMsg::Message(mut msg) => {
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
                        });
                    }
                    GroupMsg::MemberExit((mem_id, group_id, _)) => {
                        // todo send a exit message to the group
                        spawn_local(async move {
                            if let Err(e) =
                                db::db_ins().group_members.delete(&mem_id, &group_id).await
                            {
                                error!("remove members error: {:?}", e);
                                Notification::error("remove members error").notify();
                            }
                        });
                    }
                    GroupMsg::Update((group, _)) => {
                        self.handle_group_update(group);

                        // todo send message received
                    }
                    GroupMsg::DismissOrExitReceived(_) | GroupMsg::InvitationReceived(_) => {}
                },
                Msg::SingleCall(call_msg) => match call_msg {
                    SingleCall::InviteCancel(msg) => {
                        let last_msg = self.get_call_content(&msg.invite_type);
                        self.handle_offline_msg_map(
                            &mut map,
                            last_msg,
                            Message::from(msg),
                            conv_type,
                        );
                    }
                    SingleCall::InviteAnswer(msg) => {
                        if msg.agree {
                            let last_msg = self.get_call_content(&msg.invite_type);
                            self.handle_offline_msg_map(
                                &mut map,
                                last_msg,
                                Message::from(msg),
                                conv_type,
                            );
                        }
                    }
                    SingleCall::NotAnswer(msg) => {
                        let last_msg = self.get_call_content(&msg.invite_type);
                        self.handle_offline_msg_map(
                            &mut map,
                            last_msg,
                            Message::from(msg),
                            conv_type,
                        );
                    }
                    SingleCall::HangUp(msg) => {
                        let last_msg = self.get_call_content(&msg.invite_type);
                        self.handle_offline_msg_map(
                            &mut map,
                            last_msg,
                            Message::from(msg),
                            conv_type,
                        );
                    }
                    _ => {}
                },
                // handle the friendship related
                Msg::RecRelationship((fs, _)) => {
                    // receive the friend request, ignore the sequence
                    spawn_local(async move {
                        if let Err(err) = db::db_ins().friendships.put_friendship(&fs).await {
                            error!("save friend error:{:?}", err);
                        }
                    });
                }
                Msg::RelationshipRes((friend, _)) => {
                    // let send_id = ctx.props().user_id.clone();
                    spawn_local(async move {
                        if let Err(err) = db::db_ins()
                            .friendships
                            .agree_by_friend_id(friend.friend_id.as_str())
                            .await
                        {
                            warn!("agree friendship error:{:?}", err);
                            // return ChatsMsg::None;
                        }
                        // select friend if exist
                        let f = db::db_ins().friends.get(&friend.friend_id).await;
                        if !f.friend_id.is_empty() {
                            return;
                        }
                        if let Err(err) = db::db_ins().friends.put_friend(&friend).await {
                            error!("save friend error:{:?}", err);
                            return;
                        }

                        CreateConvState::update(friend);
                    });
                }
                Msg::RecRelationshipDel((friend_id, seq)) => {
                    spawn_local(async move {
                        let mut friend = db::db_ins().friends.get(&friend_id).await;
                        if !friend.friend_id.is_empty() {
                            friend.status = FriendStatus::Delete as i32;
                            if let Err(err) = db::db_ins().friends.put_friend(&friend).await {
                                error!("save friend error:{:?}", err);
                            }
                        }
                    });
                    self.handle_rec_lack_msg(ctx, seq);
                }
                _ => {}
            }
        }

        // sort
        let mut list: Vec<Conversation> = map.into_values().collect();
        list.sort_by(|a, b| b.last_msg_time.cmp(&a.last_msg_time));

        // save the offline message to the conversation list
        for v in list {
            self.operate_msg(ctx, v, false);
        }

        // sync friend list again
        let id = ctx.props().user_id.clone();
        spawn_local(async move {
            // pull friends list
            let offline_time = utils::get_local_storage(OFFLINE_TIME)
                .unwrap_or_default()
                .parse::<i64>()
                .unwrap_or_default();
            match api::friends()
                .get_friend_list_by_id(&id, offline_time)
                .await
            {
                Ok(res) => {
                    db::db_ins().friends.put_friend_list(&res).await;
                }
                Err(e) => {
                    log::error!("获取联系人列表错误: {:?}", e)
                }
            }
        });
        // send sync offline message complete message to msg_list component
        Dispatch::<RefreshMsgListState>::global().reduce_mut(|s| s.refresh = !s.refresh);
    }
}
