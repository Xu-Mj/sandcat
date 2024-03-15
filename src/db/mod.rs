pub mod conversations;
pub mod impls;

pub use impls::indexed_db::*;

use self::conversations::{Conversations, ConversationsImpl};

pub mod groups;

#[allow(dead_code)]
pub async fn convs() -> Box<dyn Conversations> {
    Box::new(ConversationsImpl::new().await)
}
