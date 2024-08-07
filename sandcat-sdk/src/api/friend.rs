use crate::{
    error::Result,
    model::friend::{
        Friend, FriendRelationSync, FriendShipAgree, FriendShipRequest, FriendShipWithUser,
    },
    pb::message::FriendInfo,
};

#[async_trait::async_trait(?Send)]
pub trait FriendApi {
    async fn apply_friend(&self, new_friend: FriendShipRequest) -> Result<FriendShipWithUser>;

    async fn query_friend(&self, friend_id: &str) -> Result<FriendInfo>;

    async fn agree_friend(&self, friendship: FriendShipAgree) -> Result<Friend>;

    async fn get_friend_list_by_id(
        &self,
        id: &str,
        offline_time: i64,
    ) -> Result<FriendRelationSync>;

    async fn update_remark(&self, user_id: String, friend_id: String, remark: String)
        -> Result<()>;

    async fn delete_friend(&self, fs_id: String, user_id: String, friend_id: String) -> Result<()>;
}
