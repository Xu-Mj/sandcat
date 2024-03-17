use wasm_bindgen::JsValue;

use crate::model::message::Message;

#[async_trait::async_trait(?Send)]
pub trait Messages {
    async fn get_last_msg(&self, friend_id: &str) -> Result<Message, JsValue>;
    async fn get_messages(
        &self,
        friend_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<Message>, JsValue>;
    async fn add_message(&self, msg: &mut Message) -> Result<(), JsValue>;
}
