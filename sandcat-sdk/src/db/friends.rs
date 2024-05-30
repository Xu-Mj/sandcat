use std::fmt::Debug;

use indexmap::IndexMap;
use wasm_bindgen::JsValue;
use yew::AttrValue;

use crate::model::friend::Friend;

#[async_trait::async_trait(?Send)]
pub trait Friends: Debug {
    async fn put_friend(&self, friend: &Friend);

    async fn put_friend_list(&self, friends: &[Friend]);

    // async fn get(&self, id: &str) -> Friend;/

    async fn get(&self, friend_id: &str) -> Friend;

    async fn get_list(&self) -> Result<IndexMap<AttrValue, Friend>, JsValue>;
    async fn get_list_by_ids(&self, ids: Vec<String>) -> Result<Vec<Friend>, JsValue>;

    async fn delete_friend(&self, id: &str) -> Result<(), JsValue>;
}
