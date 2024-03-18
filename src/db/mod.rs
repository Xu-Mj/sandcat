pub use impls::indexed_db::*;

use self::{
    conv::ConvRepo,
    conversations::Conversations,
    friend::FriendRepo,
    friend_ship::FriendShipRepo,
    friends::Friends,
    friendships::Friendships,
    group::GroupRepo,
    group_members::GroupMembers,
    group_msg::GroupMessages,
    groups::GroupInterface,
    impls::indexed_db::{group_members::GroupMembersRepo, group_msg::GroupMsgRepo},
    message::MessageRepo,
    messages::Messages,
    user::UserRepo,
    users::Users,
};

pub mod conversations;
pub mod friends;
pub mod friendships;
pub mod group_members;
pub mod group_msg;
pub mod groups;
pub mod impls;
pub mod messages;
pub mod users;

pub async fn convs() -> Box<dyn Conversations> {
    Box::new(ConvRepo::new().await)
}

pub async fn groups() -> Box<dyn GroupInterface> {
    Box::new(GroupRepo::new().await)
}

pub async fn friends() -> Box<dyn Friends> {
    Box::new(FriendRepo::new().await)
}

pub async fn friendships() -> Box<dyn Friendships> {
    Box::new(FriendShipRepo::new().await)
}

pub async fn group_members() -> Box<dyn GroupMembers> {
    Box::new(GroupMembersRepo::new().await)
}

pub async fn messages() -> Box<dyn Messages> {
    Box::new(MessageRepo::new().await)
}

pub async fn group_msgs() -> Box<dyn GroupMessages> {
    Box::new(GroupMsgRepo::new().await)
}

pub async fn users() -> Box<dyn Users> {
    Box::new(UserRepo::new().await)
}
