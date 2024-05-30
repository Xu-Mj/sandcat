use wasm_bindgen::JsValue;

use crate::{
    model::group::{Group, GroupDelete, GroupRequest},
    pb::message::{GroupInviteNew, GroupUpdate},
};

#[async_trait::async_trait(?Send)]
pub trait GroupApi {
    async fn create(&self, data: GroupRequest, user_id: &str) -> Result<Group, JsValue>;
    async fn invite(&self, data: GroupInviteNew) -> Result<(), JsValue>;
    async fn delete(&self, data: GroupDelete) -> Result<(), JsValue>;
    async fn update(&self, user_id: &str, group: GroupUpdate) -> Result<Group, JsValue>;
}
