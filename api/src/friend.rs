use wasm_bindgen::JsValue;

use abi::model::friend::{Friend, FriendShipAgree, FriendShipRequest, FriendShipWithUser};

#[async_trait::async_trait(?Send)]
pub trait FriendApi {
    async fn apply_friend(
        &self,
        new_friend: FriendShipRequest,
    ) -> Result<FriendShipWithUser, JsValue>;

    async fn agree_friend(&self, friendship: FriendShipAgree) -> Result<Friend, JsValue>;

    async fn get_friend_list_by_id(&self, id: String) -> Result<Vec<Friend>, JsValue>;

    async fn update_remark(
        &self,
        user_id: String,
        friend_id: String,
        remark: String,
    ) -> Result<(), JsValue>;

    async fn delete_friend(&self, user_id: String, friend_id: String) -> Result<(), JsValue>;
}
