use std::collections::HashMap;

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yewdux::Dispatch;

use crate::{
    db,
    model::{
        conversation::Conversation,
        message::{
            convert_server_msg, GroupMsg, InviteType, Message, Msg, SingleCall,
            DEFAULT_HELLO_MESSAGE,
        },
        ContentType, RightContentType,
    },
    pb::message::Msg as PbMsg,
    state::OfflineMsgState,
};

use super::{conversations::ChatsMsg, Chats};

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

        // let (last_msg, last_msg_type) = get_invite_type(invite_type);
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
            db::messages().await.add_message(&mut msg).await.unwrap();
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
        let mut map: HashMap<AttrValue, Conversation> = HashMap::with_capacity(messages.len());

        for item in messages.into_iter() {
            // let friend_id = item.send_id.clone();
            let msg = convert_server_msg(item).unwrap();
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
                    GroupMsg::Message(msg) => {
                        spawn_local(async move {
                            db::group_msgs().await.put(&msg).await.unwrap();
                        });
                    }
                    GroupMsg::MemberExit((mem_id, group_id, _)) => {
                        // todo send a exit message to the group
                        spawn_local(async move {
                            db::group_members()
                                .await
                                .delete(&mem_id, &group_id)
                                .await
                                .unwrap();
                        });
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
                        db::friendships().await.put_friendship(&fs).await;
                    });
                }
                Msg::RelationshipRes((friend, _)) => {
                    let send_id = ctx.props().user_id.clone();
                    ctx.link().send_future(async move {
                        db::friendships()
                            .await
                            .agree_by_friend_id(friend.friend_id.as_str())
                            .await;
                        // select friend if exist
                        let f = db::friends().await.get(&friend.friend_id).await;
                        if !f.friend_id.is_empty() {
                            return ChatsMsg::None;
                        }
                        db::friends().await.put_friend(&friend).await;
                        // send hello message
                        let mut msg = Message {
                            local_id: nanoid::nanoid!().into(),
                            send_id,
                            friend_id: friend.friend_id.clone(),
                            content_type: ContentType::Text,
                            content: friend
                                .hello
                                .unwrap_or_else(|| AttrValue::from(DEFAULT_HELLO_MESSAGE)),
                            create_time: chrono::Local::now().timestamp_millis(),
                            is_read: true,
                            is_self: true,
                            ..Default::default()
                        };
                        db::messages()
                            .await
                            .add_message(&mut msg)
                            .await
                            .map_err(|err| log::error!("save message fail:{:?}", err))
                            .unwrap();

                        ChatsMsg::SendMessage(Msg::Single(msg))
                    });
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

        // send sync offline message complete message to msg_list component
        Dispatch::<OfflineMsgState>::global().reduce_mut(|s| s.complete = ());
    }
}
