use std::fmt::Debug;

use indexmap::IndexMap;
use wasm_bindgen::JsValue;
use yew::AttrValue;

use crate::model::conversation::Conversation;

#[async_trait::async_trait(?Send)]
pub trait Conversations: Debug {
    async fn mute(&self, conv: &Conversation) -> Result<(), JsValue>;

    async fn put_conv(&self, conv: &Conversation) -> Result<(), JsValue>;

    async fn self_update_conv(&self, conv: Conversation) -> Result<Conversation, JsValue>;

    async fn get_convs(&self) -> Result<IndexMap<AttrValue, Conversation>, JsValue>;

    async fn get_by_frined_id(&self, friend_id: &str) -> Result<Option<Conversation>, JsValue>;

    async fn delete(&self, friend_id: &str) -> Result<(), JsValue>;
}
