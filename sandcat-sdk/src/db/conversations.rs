use std::fmt::Debug;

use indexmap::IndexMap;
use yew::AttrValue;

use crate::error::Result;
use crate::model::conversation::Conversation;

#[async_trait::async_trait(?Send)]
pub trait Conversations: Debug {
    async fn mute(&self, conv: &Conversation) -> Result<()>;

    async fn put_conv(&self, conv: &Conversation) -> Result<()>;

    async fn self_update_conv(&self, conv: Conversation) -> Result<Conversation>;

    async fn get_pined_convs(&self) -> Result<IndexMap<AttrValue, Conversation>>;

    async fn get_convs(&self) -> Result<IndexMap<AttrValue, Conversation>>;

    async fn get_by_frined_id(&self, friend_id: &str) -> Result<Option<Conversation>>;

    async fn delete(&self, friend_id: &str) -> Result<()>;
}
