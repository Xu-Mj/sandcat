use std::fmt::Debug;

use wasm_bindgen::JsValue;

use abi::model::group::{GroupMember, GroupMemberFromServer};

#[async_trait::async_trait(?Send)]
pub trait GroupMembers: Debug {
    async fn put(&self, mem: &GroupMember) -> Result<(), JsValue>;
    async fn put_list(&self, members: Vec<GroupMemberFromServer>) -> Result<(), JsValue>;
    async fn get(&self, id: i64) -> Result<Option<GroupMember>, JsValue>;
    async fn get_by_group_id_and_friend_id(
        &self,
        group_id: &str,
        friend_id: &str,
    ) -> Result<Option<GroupMember>, JsValue>;
    async fn get_list_by_group_id(&self, group_id: &str) -> Result<Vec<GroupMember>, JsValue>;
    async fn delete(&self, group_id: &str, user_id: &str) -> Result<(), JsValue>;
}
