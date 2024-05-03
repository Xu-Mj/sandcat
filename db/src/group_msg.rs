use indexmap::IndexMap;
use wasm_bindgen::JsValue;
use yew::AttrValue;

use abi::model::message::{Message, ServerResponse};

#[async_trait::async_trait(?Send)]
pub trait GroupMessages {
    async fn put(&self, group: &Message) -> Result<(), JsValue>;

    async fn get_messages(
        &self,
        friend_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<IndexMap<AttrValue, Message>, JsValue>;

    async fn get_last_msg(&self, group_id: &str) -> Result<Message, JsValue>;

    async fn update_msg_status(&self, msg: &ServerResponse) -> Result<(), JsValue>;
}
