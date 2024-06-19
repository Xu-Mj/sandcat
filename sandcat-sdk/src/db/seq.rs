use std::fmt::Debug;

use crate::{error::Error, model::seq::Seq};

/// seq's id is always 1
#[async_trait::async_trait(?Send)]
pub trait SeqInterface: Debug {
    async fn put(&self, seq: &Seq) -> Result<(), Error>;

    async fn get(&self) -> Result<Seq, Error>;
}
