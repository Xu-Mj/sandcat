use std::fmt::Debug;

use crate::error::Result;
use crate::model::friend::FriendShipWithUser;

#[async_trait::async_trait(?Send)]
pub trait Friendships: Debug {
    async fn agree(&self, friendship_id: &str) -> Result<()>;

    async fn agree_by_friend_id(&self, friend_id: &str) -> Result<()>;

    async fn put_friendship(&self, friendship: &FriendShipWithUser) -> Result<()>;

    async fn get_friendship(&self, friendship_id: &str) -> Result<Option<FriendShipWithUser>>;

    async fn get_friendship_by_friend_id(
        &self,
        friend_id: &str,
    ) -> Result<Option<FriendShipWithUser>>;

    async fn get_unread_count(&self) -> Result<usize>;

    async fn clean_unread_count(&self) -> Result<Vec<String>>;

    // async fn update_status(&self, fs: &str) -> Result<(), JsValue>;
    async fn get_list(&self) -> Result<Vec<FriendShipWithUser>>;
}
