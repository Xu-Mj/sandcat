use indexmap::IndexMap;
use wasm_bindgen::JsValue;
use yew::AttrValue;

use crate::model::message::{Message, ServerResponse};

#[async_trait::async_trait(?Send)]
pub trait Messages {
    async fn get_last_msg(&self, friend_id: &str) -> Result<Message, JsValue>;

    async fn get_messages(
        &self,
        friend_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<IndexMap<AttrValue, Message>, JsValue>;

    async fn add_message(&self, msg: &mut Message) -> Result<(), JsValue>;

    async fn update_msg_status(&self, msg: &ServerResponse) -> Result<(), JsValue>;

    async fn update_read_status(&self, friend_id: &str) -> Result<(), JsValue>;

    async fn unread_count(&self) -> usize;
}
