use html::Scope;
use log::{debug, error};
use yew::prelude::*;

use sandcat_sdk::{
    api, db,
    error::Error,
    model::{
        conversation::Conversation,
        group::{Group, GroupMember, GroupRequest},
        message::{GroupInvitation, GroupInviteNewResponse},
        notification::Notification,
        ContentType,
    },
    pb::message::GroupMemberRole,
    state::{ItemType, UpdateFriendState},
};
use yewdux::Dispatch;

use super::{conversations::ChatsMsg, Chats};

/// Handle group related messages
/// 1. group invitations:
///     save the group information and the members information to db
///     and then create a new conversation, add it into our conversation list
///     store it to db
/// 2. group dismiss:
///     update group information set is_dismissed to true
///     update the conversation list about the last message content
/// 3. members leave:
///     delete member from the db

impl Chats {
    pub async fn handle_group_invitation(ctx: Scope<Self>, msg: GroupInvitation) {
        // create group conversation directly
        // store conversation
        let info = Group::from(msg.info.unwrap());
        let mut conv = Conversation::from(info.clone());
        conv.unread_count = 0;

        if let Err(e) = db::db_ins().convs.put_conv(&conv).await {
            error!("Failed to store conversation: {:?}", e);
            Notification::error("Failed to store conversation").notify();
            return;
        }

        // store group information
        if let Err(err) = db::db_ins().groups.put(&info).await {
            error!("store group error : {:?}", err);
            Notification::error("Failed to store group").notify();
            return;
        };

        // store group members
        let members: Vec<GroupMember> = msg.members.into_iter().map(GroupMember::from).collect();
        if let Err(e) = db::db_ins().group_members.put_list(&members).await {
            error!("save group member error: {:?}", e);
            Notification::error("Failed to store group member").notify();
            return;
        }

        // send back received message
        ctx.send_message(ChatsMsg::SendBackGroupInvitation(info.id.clone()));

        // send add friend state
        ctx.send_message(ChatsMsg::SendCreateGroupToContacts(info));
        ctx.send_message(ChatsMsg::InsertConv(conv));
    }

    pub async fn handle_invite_new(
        ctx: Scope<Self>,
        user_id: String,
        resp: GroupInviteNewResponse,
    ) {
        if resp.members.contains(&user_id.to_string()) {
            debug!("be invited");
            // be invited
            // get group info and group members
            match api::groups()
                .get_with_members(&user_id, &resp.group_id)
                .await
            {
                Ok(response) => {
                    debug!("get group info and group members");
                    // save conversation
                    let mut conv = Conversation::from(response.group.clone());
                    conv.unread_count = 0;

                    if let Err(e) = db::db_ins().convs.put_conv(&conv).await {
                        error!("Failed to store conversation: {:?}", e);
                        Notification::error("Failed to store conversation").notify();
                        return;
                    }

                    // save group
                    if let Err(err) = db::db_ins().groups.put(&response.group).await {
                        error!("store group error: {:?}", err);
                        Notification::error("Failed to store group info").notify();
                        return;
                    }

                    // save group members
                    if let Err(e) = db::db_ins().group_members.put_list(&response.members).await {
                        error!("save group member error: {:?}", e);
                        Notification::error("Failed to store group member").notify();
                        return;
                    }

                    // send back received message
                    ctx.send_message(ChatsMsg::SendBackGroupInvitation(resp.group_id.into()));

                    // send add friend state
                    ctx.send_message(ChatsMsg::SendCreateGroupToContacts(response.group));
                    ctx.send_message(ChatsMsg::InsertConv(conv));
                }
                Err(e) => {
                    error!("get group info and group members error: {:?}", e);
                    Notification::error("Failed to get group info and group members").notify();
                }
            }
        } else {
            // get group members
            match api::groups()
                .get_members(&user_id, &resp.group_id, resp.members)
                .await
            {
                Ok(members) => {
                    // save members
                    if let Err(e) = db::db_ins().group_members.put_list(&members).await {
                        error!("save group member error: {:?}", e);
                        Notification::error("Failed to store group member").notify();
                    }
                }
                Err(e) => {
                    error!("get group members error: {:?}", e);
                    Notification::error("Failed to get group members").notify();
                }
            }
        }
    }

    pub fn create_group(ctx: &Context<Self>, nodes: Vec<String>) {
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
                let friend = db::db_ins().friends.get(node).await;
                if !friend.fs_id.is_empty() {
                    let mut name = friend.name.clone();
                    if friend.remark.is_some() {
                        name = friend.remark.as_ref().unwrap().clone();
                    }
                    group_name.push_str(&name);
                    if i < 8 {
                        avatar.push(friend.avatar.to_string());
                    }
                    values.push(GroupMember::from(friend));
                }
            }

            // push self
            let user = match db::db_ins().users.get(user_id.as_str()).await {
                Ok(user) => user,
                Err(e) => {
                    error!("get user error:{:?}", e);
                    Notification::error("query user error").notify();
                    return ChatsMsg::None;
                }
            };

            group_name.push_str(&user.name);
            group_name.push_str("、Group");
            let group_req = GroupRequest {
                owner: user_id.to_string(),
                avatar: avatar.join(","),
                group_name,
                members_id: nodes,
                id: String::new(),
            };

            let mut member = GroupMember::from(user);
            member.role = GroupMemberRole::Owner as i32;

            values.push(member);
            // send create request
            match api::groups().create(group_req, &user_id).await {
                Ok(g) => {
                    log::debug!("group created: {:?}", g);

                    // sotre the group info to database
                    if let Err(err) = db::db_ins().groups.put(&g).await {
                        log::error!("create group error: {:?}", err);
                        Notification::error("Failed to store group").notify();
                        return ChatsMsg::None;
                    }

                    // store group members to db
                    for v in values.iter_mut() {
                        v.group_id = g.id.clone();
                        if let Err(e) = db::db_ins().group_members.put(v).await {
                            log::error!("save group member error: {:?}", e);
                            Notification::error("Failed to store group member").notify();
                            continue;
                        }
                    }

                    // send message to contacts component
                    cloned_ctx.send_message(ChatsMsg::SendCreateGroupToContacts(g.clone()));

                    // store conversation info to db
                    let mut conv = Conversation::from(g);
                    conv.unread_count = 0;
                    if let Err(e) = db::db_ins().convs.put_conv(&conv).await {
                        error!("failed to store conversation to db: {:?}", e);
                        Notification::error("Failed to store group").notify();
                        return ChatsMsg::None;
                    }

                    // insert conversation to ui list
                    ChatsMsg::InsertConv(conv)
                }
                Err(err) => {
                    log::error!("create group request error: {:?}", err);
                    Notification::error("Failed to create group").notify();
                    ChatsMsg::None
                }
            }
        });
    }

    pub async fn dismiss_group(group_id: String) -> Result<Conversation, Error> {
        // update group to dismissed
        let group = db::db_ins().groups.dismiss(&group_id).await?;

        // select owner info
        let mem = db::db_ins()
            .group_members
            .get_by_group_id_and_friend_id(&group_id, group.owner.as_str())
            .await?
            .ok_or(Error::local_not_found("group member not found"))?;

        // get the conversation information
        let conv = if let Ok(Some(mut conv)) = db::db_ins().convs.get_by_frined_id(&group_id).await
        {
            let message = format!("{} dismissed this group", mem.group_name);
            conv.last_msg = message.clone().into();
            conv.unread_count = 0;
            conv
        } else {
            Conversation {
                friend_id: group_id.into(),
                avatar: group.avatar,
                name: group.name,
                last_msg: format!("{} dismissed this group", mem.group_name).into(),
                unread_count: 0,
                last_msg_time: chrono::Utc::now().timestamp_millis(),
                last_msg_type: ContentType::Text,
                remark: group.remark,
                ..Default::default()
            }
        };

        // create a new conversation to notify user the group was dismissed
        db::db_ins().convs.put_conv(&conv).await?;

        Ok(conv)
    }

    pub async fn handle_group_update(group: Group) {
        // update group information
        if let Err(err) = db::db_ins().groups.put(&group).await {
            log::error!("update group fail:{:?}", err);
            Notification::error("Failed to update group").notify();
        }
        // update conversation
        Dispatch::<UpdateFriendState>::global().reduce_mut(|s| {
            s.id = group.id;
            s.name = Some(group.name);
            s.avatar = Some(group.avatar);
            s.type_ = ItemType::Group;
        });
    }
}
