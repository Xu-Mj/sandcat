use wasm_bindgen::JsValue;

use crate::model::conversation::Conversation;

pub trait Conversations {
    async fn new() -> Self
    where
        Self: Sized;

    async fn mute(&self, conv: &Conversation) -> Result<(), JsValue>;

    /*     async fn put_conv(
        &self,
        conv: &Conversation,
        is_clean_unread_count: bool,
    ) -> Result<Conversation, JsValue>;

    async fn get_convs2(&self) -> Result<IndexMap<AttrValue, Conversation>, JsValue>;

    async fn get_by_frined_id(&self, friend_id: AttrValue) -> Conversation;

    async fn delete(&self, friend_id: AttrValue) -> Result<(), JsValue>; */
}

pub struct ConversationsImpl;
impl Conversations for ConversationsImpl {
    async fn new() -> Self {
        Self {}
    }

    async fn mute(&self, conv: &Conversation) -> Result<(), JsValue> {
        log::debug!("conv: {:?}", conv);
        Ok(())
    }
}
