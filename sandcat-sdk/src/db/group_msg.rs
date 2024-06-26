use std::fmt::Debug;

use indexmap::IndexMap;
use yew::AttrValue;

use crate::error::Result;
use crate::model::message::{Message, ServerResponse};

#[async_trait::async_trait(?Send)]
pub trait GroupMessages: Debug {
    async fn put(&self, group: &Message) -> Result<()>;

    async fn get_messages(
        &self,
        friend_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<IndexMap<AttrValue, Message>>;

    async fn get_last_msg(&self, group_id: &str) -> Result<Message>;

    async fn update_msg_status(&self, msg: &ServerResponse) -> Result<()>;

    async fn batch_delete(&self, group_id: &str) -> Result<()>;
}
