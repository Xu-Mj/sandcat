use wasm_bindgen::JsValue;

use abi::model::friend::FriendShipWithUser;

#[async_trait::async_trait(?Send)]
pub trait Friendships {
    async fn agree(&self, friendship_id: &str);
    async fn agree_by_friend_id(&self, friend_id: &str);
    async fn put_friendship(&self, friendship: &FriendShipWithUser);
    async fn get_friendship(&self, friendship_id: &str) -> Option<FriendShipWithUser>;
    async fn get_friendship_by_friend_id(&self, friend_id: &str) -> Option<FriendShipWithUser>;
    async fn get_unread_count(&self) -> usize;
    async fn clean_unread_count(&self) -> Result<Vec<String>, JsValue>;
    // async fn update_status(&self, fs: &str) -> Result<(), JsValue>;
    async fn get_list(&self) -> Vec<FriendShipWithUser>;
}
