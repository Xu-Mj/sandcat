use wasm_bindgen::JsValue;

use crate::model::message::Message;

#[async_trait::async_trait(?Send)]
pub trait GroupMessages {
    async fn put(&self, group: &Message) -> Result<(), JsValue>;
    async fn get_messages(
        &self,
        friend_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<Message>, JsValue>;
}
