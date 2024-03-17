use wasm_bindgen::JsValue;

use crate::model::group::GroupMember;

#[async_trait::async_trait(?Send)]
pub trait GroupMembers {
    async fn put(&self, mem: &GroupMember) -> Result<(), JsValue>;
    async fn put_list(&self, members: Vec<GroupMember>) -> Result<(), JsValue>;
    async fn get(&self, id: i64) -> Result<Option<GroupMember>, JsValue>;
    async fn get_by_group_id_and_friend_id(
        &self,
        group_id: &str,
        friend_id: &str,
    ) -> Result<Option<GroupMember>, JsValue>;
    async fn get_list_by_group_id(&self, group_id: &str) -> Result<Vec<GroupMember>, JsValue>;
}
