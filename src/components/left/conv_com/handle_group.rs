use crate::{
    api, db,
    model::{
        conversation::Conversation,
        group::{Group, GroupMember, GroupRequest},
        message::GroupInvitation,
        ContentType,
    },
};
use yew::prelude::*;

use super::{conversations::ChatsMsg, Chats};

impl Chats {
    pub fn handle_group_invitation(&mut self, ctx: &Context<Self>, msg: GroupInvitation) {
        // create group conversation directly
        let clone_ctx = ctx.link().clone();
        ctx.link().send_future(async move {
            // store conversation
            let info = Group::from(msg.info.unwrap());
            let conv = Conversation::from(info.clone());
            db::convs().await.put_conv(&conv, true).await.unwrap();

            // store group information
            if let Err(err) = db::groups().await.put(&info).await {
                log::error!("store group error : {:?}", err);
            };

            // store group members
            if let Err(e) = db::group_members().await.put_list(msg.members).await {
                log::error!("save group member error: {:?}", e);
            }

            // send back received message
            clone_ctx.send_message(ChatsMsg::SendBackGroupInvitation(info.id.clone()));

            // send add friend state
            clone_ctx.send_message(ChatsMsg::SendCreateGroupToContacts(info));
            ChatsMsg::InsertConv(conv)
        });
    }

    pub fn create_group(&mut self, ctx: &Context<Self>, nodes: Vec<String>) {
        // log::debug!("get group mems: {:?} ; ", nodes);
        let user_id = ctx.props().user_id.clone();
        let self_avatar = ctx.props().avatar.clone();

        // clone ctx to send message
        let cloned_ctx = ctx.link().clone();

        ctx.link().send_future(async move {
            if nodes.is_empty() {
                return ChatsMsg::ShowSelectFriendList;
            }
            let mut values = Vec::with_capacity(nodes.len());
            // let mut ids = Vec::with_capacity(nodes.len());
            let mut avatar = Vec::with_capacity(nodes.len());
            // push self avatar
            avatar.push(self_avatar.to_string());
            let mut group_name = String::new();
            for (i, node) in nodes.iter().enumerate() {
                let friend = db::friends().await.get(node).await;
                if !friend.fs_id.is_empty() {
                    let mut name = friend.name.clone();
                    if friend.remark.is_some() {
                        name = friend.remark.as_ref().unwrap().clone();
                    }
                    group_name.push_str(name.as_str());
                    if i < 8 {
                        avatar.push(friend.avatar.clone().to_string());
                    }
                    values.push(GroupMember::from(friend));
                }
            }

            group_name.push_str("、Group");
            let group_req = GroupRequest {
                owner: user_id.to_string(),
                avatar: avatar.join(","),
                group_name,
                members_id: nodes,
                id: String::new(),
            };
            // push self
            values.push(GroupMember::from(
                db::users().await.get(user_id.as_str()).await.unwrap(),
            ));
            // send create request
            match api::groups()
                .create_group(group_req, user_id.as_str())
                .await
            {
                Ok(g) => {
                    log::debug!("group created: {:?}", g);

                    // sotre the group info to database
                    if let Err(err) = db::groups().await.put(&g).await {
                        log::error!("create group error: {:?}", err);
                        return ChatsMsg::None;
                    }

                    // store group members to db
                    for v in values.iter_mut() {
                        v.group_id = g.id.clone();
                        if let Err(e) = db::group_members().await.put(v).await {
                            log::error!("save group member error: {:?}", e);
                            continue;
                        }
                    }

                    // send message to contacts component
                    cloned_ctx.send_message(ChatsMsg::SendCreateGroupToContacts(g.clone()));

                    // store conversation info to db
                    let conv = Conversation::from(g);
                    db::convs().await.put_conv(&conv, true).await.unwrap();

                    // insert conversation to ui list
                    ChatsMsg::InsertConv(conv)
                }
                Err(err) => {
                    log::error!("create group request error: {:?}", err);
                    ChatsMsg::None
                }
            }
        });
    }
    pub fn handle_group_dismiss(&mut self, ctx: &Context<Self>, group_id: String) {
        let key = AttrValue::from(group_id.clone());
        if let Some(conv) = self.list.get_mut(&key) {
            conv.last_msg_time = chrono::Local::now().timestamp_millis();
            conv.last_msg_type = ContentType::Text;
            let mut conv = conv.clone();
            ctx.link().send_future(async move {
                // query group information and owner info
                if let Ok(Some(group)) = db::groups().await.get(&group_id).await {
                    if let Ok(Some(mem)) = db::group_members()
                        .await
                        .get_by_group_id_and_friend_id(&group_id, group.owner.as_str())
                        .await
                    {
                        let message = format!("{} dismissed this group", mem.group_name);
                        conv.last_msg = message.clone().into();

                        if let Err(e) = db::convs().await.put_conv(&conv, true).await {
                            log::error!("dismiss group error: {:?}", e);
                        } else {
                            return ChatsMsg::DismissGroup(key, message);
                        }
                    }
                }
                ChatsMsg::None
            })
        }
    }
}
