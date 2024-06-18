use crate::error::Error;
use crate::{
    model::friend::{Friend, FriendShipAgree, FriendShipRequest, FriendShipWithUser},
    pb::message::FriendInfo,
};

#[async_trait::async_trait(?Send)]
pub trait FriendApi {
    async fn apply_friend(
        &self,
        new_friend: FriendShipRequest,
    ) -> Result<FriendShipWithUser, Error>;

    async fn query_friend(&self, friend_id: &str) -> Result<FriendInfo, Error>;

    async fn agree_friend(&self, friendship: FriendShipAgree) -> Result<Friend, Error>;

    async fn get_friend_list_by_id(&self, id: String) -> Result<Vec<Friend>, Error>;

    async fn update_remark(
        &self,
        user_id: String,
        friend_id: String,
        remark: String,
    ) -> Result<(), Error>;

    async fn delete_friend(&self, user_id: String, friend_id: String) -> Result<(), Error>;
}
