use std::collections::HashMap;

use html::Scope;
use log::{error, warn};
use yew::prelude::*;

use sandcat_sdk::{
    db,
    model::{
        conversation::Conversation,
        friend::FriendStatus,
        message::{convert_server_msg, GroupMsg, InviteType, Message, Msg, SingleCall},
        ContentType, RightContentType,
    },
    pb::message::Msg as PbMsg,
    state::{RefreshMsgListState, UnreadState},
};
use yewdux::Dispatch;

use super::Chats;

impl Chats {
    async fn handle_offline_msg_map(
        map: &mut HashMap<AttrValue, Conversation>,
        last_msg: AttrValue,
        mut msg: Message,
        conv_type: RightContentType,
        cur_user_id: AttrValue,
    ) {
        if cur_user_id != msg.send_id {
            let friend_id = msg.send_id.clone();
            msg.send_id = msg.friend_id.clone();
            msg.friend_id = friend_id;
        } else {
            msg.is_self = true;
        }

        let unread_count = if msg.is_read == 1 || msg.is_self {
            0
        } else {
            1
        };
        let conv = Conversation {
            friend_id: msg.friend_id.clone(),
            last_msg,
            last_msg_time: msg.send_time,
            last_msg_type: msg.content_type,
            unread_count,
            conv_type,
            ..Default::default()
        };

        if msg.content_type == ContentType::Audio {
            // request from file server
            if let Err(e) =
                Self::download_voice_and_save(&msg.content, &msg.local_id, msg.audio_duration).await
            {
                error!("download voice error: {}", e);
            }
            msg.audio_downloaded = true;
        }

        if let Err(e) = db::db_ins().messages.add_message(&mut msg).await {
            error!("save message to db error: {:?}", e);
        }

        if let Some(v) = map.get_mut(&conv.friend_id) {
            v.last_msg = conv.last_msg;
            v.last_msg_time = conv.last_msg_time;
            v.last_msg_type = conv.last_msg_type;
            v.unread_count += conv.unread_count;
        } else {
            map.insert(conv.friend_id.clone(), conv);
        }
    }

    pub fn get_call_content(invite_type: &InviteType) -> AttrValue {
        match invite_type {
            InviteType::Video => AttrValue::from("[视频通话]"),
            InviteType::Audio => AttrValue::from("[语音通话]"),
        }
    }

    // todo handle the friend request and send the group create message to contact
    pub async fn handle_offline_messages(
        ctx: Scope<Self>,
        user_id: AttrValue,
        messages: Vec<PbMsg>,
    ) -> Vec<Conversation> {
        if messages.is_empty() {
            return vec![];
        }

        let mut map: HashMap<AttrValue, Conversation> = HashMap::with_capacity(messages.len());

        for item in messages.into_iter() {
            // let friend_id = item.send_id.clone();
            let msg = match convert_server_msg(item) {
                Ok(msg) => msg,
                Err(e) => {
                    error!("convert_server_msg error: {:?}", e);
                    continue;
                }
            };

            let conv_type = Self::get_msg_type(&msg);
            match msg {
                Msg::Single(msg) => {
                    Self::handle_offline_msg_map(
                        &mut map,
                        msg.content.clone(),
                        msg,
                        conv_type,
                        user_id.clone(),
                    )
                    .await;
                }
                Msg::Group(group_msg) => match group_msg {
                    GroupMsg::Invitation((msg, _)) => {
                        Self::handle_group_invitation(ctx.clone(), msg).await;
                    }
                    GroupMsg::Dismiss((group_id, _)) => {
                        if let Err(err) = Self::dismiss_group(group_id).await {
                            error!("dismiss group failed: {:?}", err);
                        };
                    }
                    GroupMsg::Message(mut msg) => {
                        if msg.content_type == ContentType::Audio {
                            // request from file server
                            if let Err(e) = Self::download_voice_and_save(
                                &msg.content,
                                &msg.local_id,
                                msg.audio_duration,
                            )
                            .await
                            {
                                error!("download voice and save error {:?}", e);
                            }
                            msg.audio_downloaded = true;
                        }
                        if let Err(e) = db::db_ins().group_msgs.put(&msg).await {
                            error!("save message to db error: {:?}", e);
                        }
                    }
                    GroupMsg::MemberExit((mem_id, group_id, _)) => {
                        // todo send a exit message to the group
                        if let Err(e) = db::db_ins().group_members.delete(&mem_id, &group_id).await
                        {
                            error!("remove members error: {:?}", e);
                        }
                    }
                    GroupMsg::Update((group, _)) => {
                        Self::handle_group_update(group).await;

                        // todo send message received
                    }
                    GroupMsg::DismissOrExitReceived(_) | GroupMsg::InvitationReceived(_) => {}
                },
                Msg::SingleCall(call_msg) => match call_msg {
                    SingleCall::InviteCancel(msg) => {
                        let last_msg = Self::get_call_content(&msg.invite_type);
                        Self::handle_offline_msg_map(
                            &mut map,
                            last_msg,
                            Message::from(msg),
                            conv_type,
                            user_id.clone(),
                        )
                        .await;
                    }
                    SingleCall::InviteAnswer(msg) => {
                        if msg.agree {
                            let last_msg = Self::get_call_content(&msg.invite_type);
                            Self::handle_offline_msg_map(
                                &mut map,
                                last_msg,
                                Message::from(msg),
                                conv_type,
                                user_id.clone(),
                            )
                            .await;
                        }
                    }
                    SingleCall::NotAnswer(msg) => {
                        let last_msg = Self::get_call_content(&msg.invite_type);
                        Self::handle_offline_msg_map(
                            &mut map,
                            last_msg,
                            Message::from(msg),
                            conv_type,
                            user_id.clone(),
                        )
                        .await;
                    }
                    SingleCall::HangUp(msg) => {
                        let last_msg = Self::get_call_content(&msg.invite_type);
                        Self::handle_offline_msg_map(
                            &mut map,
                            last_msg,
                            Message::from(msg),
                            conv_type,
                            user_id.clone(),
                        )
                        .await;
                    }
                    _ => {}
                },
                // handle the friendship related
                Msg::RecRelationship((fs, _)) => {
                    // receive the friend request, ignore the sequence
                    if let Err(err) = db::db_ins().friendships.put_friendship(&fs).await {
                        error!("save friend error:{:?}", err);
                    }
                }
                Msg::RelationshipRes((friend, _)) => {
                    if let Err(err) = db::db_ins()
                        .friendships
                        .agree_by_friend_id(friend.friend_id.as_str())
                        .await
                    {
                        warn!("agree friendship error:{:?}", err);
                    }

                    if let Err(err) = db::db_ins().friends.put_friend(&friend).await {
                        error!("save friend error:{:?}", err);
                        continue;
                    }

                    let mut conv = Conversation::from(friend);
                    conv.last_msg = AttrValue::from("new friend");
                    conv.last_msg_type = ContentType::Text;
                    conv.last_msg_time = chrono::Utc::now().timestamp_millis();
                    if let Some(v) = map.get_mut(&conv.friend_id) {
                        v.last_msg = conv.last_msg.clone();
                        v.last_msg_time = conv.last_msg_time;
                        v.last_msg_type = conv.last_msg_type;
                    } else {
                        map.insert(conv.friend_id.clone(), conv);
                    }
                }
                Msg::RecRelationshipDel((friend_id, _seq)) => {
                    let mut friend = db::db_ins().friends.get(&friend_id).await;
                    if !friend.friend_id.is_empty() {
                        friend.status = FriendStatus::Deleted as i32;
                        if let Err(err) = db::db_ins().friends.put_friend(&friend).await {
                            error!("save friend error:{:?}", err);
                        }
                    }
                }
                _ => {}
            }
        }

        // sync friend list again
        // pull friends list
        // Self::pull_friends(&user_id).await;

        // send handle finished state to notify main thread
        // sort
        let list: Vec<Conversation> = map.into_values().collect();
        let mut unread_count = 0;
        // save to db
        for conv in list.iter() {
            unread_count += conv.unread_count;
            if let Err(e) = db::db_ins().convs.put_conv(conv).await {
                error!("save conversation error: {:?}", e);
            }
        }
        UnreadState::incr_msg(unread_count);
        // send sync offline message complete message to msg_list component
        Dispatch::<RefreshMsgListState>::global().reduce_mut(|s| s.refresh = !s.refresh);
        // list.sort_by(|a, b| b.last_msg_time.cmp(&a.last_msg_time));
        list
    }
}
