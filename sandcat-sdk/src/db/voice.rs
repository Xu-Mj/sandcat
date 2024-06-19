use std::fmt::Debug;

use crate::{error::Result, model::voice::Voice};

#[async_trait::async_trait(?Send)]
pub trait Voices: Debug {
    async fn save(&self, voice: &Voice) -> Result<()>;

    async fn get(&self, local_id: &str) -> Result<Voice>;

    async fn del(&self, local_id: &str) -> Result<()>;
}
