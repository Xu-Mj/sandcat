use std::fmt::Debug;

use crate::{error::Error, model::voice::Voice};

#[async_trait::async_trait(?Send)]
pub trait Voices: Debug {
    async fn save(&self, voice: &Voice) -> Result<(), Error>;

    async fn get(&self, local_id: &str) -> Result<Voice, Error>;

    async fn del(&self, local_id: &str) -> Result<(), Error>;
}
