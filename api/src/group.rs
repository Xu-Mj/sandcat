use wasm_bindgen::JsValue;

use abi::model::group::{Group, GroupDelete, GroupRequest};

#[async_trait::async_trait(?Send)]
pub trait GroupApi {
    async fn create_group(&self, data: GroupRequest, user_id: &str) -> Result<Group, JsValue>;
    async fn delete_group(&self, data: GroupDelete) -> Result<(), JsValue>;
}
