use std::fmt::Debug;

use indexmap::IndexMap;
use yew::AttrValue;

use crate::error::Result;
use crate::model::conversation::Conversation;

#[async_trait::async_trait(?Send)]
pub trait Conversations: Debug {
    async fn put_conv(&self, conv: &Conversation) -> Result<()>;

    /// dismiss group; update conversation
    async fn dismiss_group(&self, conv_id: &str) -> Result<()>;

    async fn self_update_conv(&self, conv: &mut Conversation) -> Result<()>;

    async fn get_pined_convs(&self) -> Result<IndexMap<AttrValue, Conversation>>;

    async fn get_convs(&self) -> Result<IndexMap<AttrValue, Conversation>>;

    async fn get_by_frined_id(&self, friend_id: &str) -> Result<Option<Conversation>>;

    async fn delete(&self, friend_id: &str) -> Result<()>;
}
