use std::fmt::Debug;

use indexmap::IndexMap;
use yew::AttrValue;

use crate::error::Result;
use crate::model::message::{Message, ServerResponse};

#[async_trait::async_trait(?Send)]
pub trait Messages: Debug {
    async fn get_last_msg(&self, friend_id: &str) -> Result<Option<Message>>;

    async fn get(&self, local_id: &str) -> Result<Option<Message>>;

    async fn get_messages(
        &self,
        friend_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<IndexMap<AttrValue, Message>>;

    async fn add_message(&self, msg: &Message) -> Result<()>;

    async fn update_msg_status(&self, msg: &ServerResponse) -> Result<()>;

    async fn update_read_status(&self, friend_id: &str) -> Result<Vec<i64>>;

    async fn unread_count(&self) -> usize;

    async fn delete_batch(&self, friend_id: &str) -> Result<()>;

    async fn delete(&self, local_id: &AttrValue) -> Result<()>;
}
