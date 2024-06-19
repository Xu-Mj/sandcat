use std::fmt::Debug;

use crate::{error::Result, model::seq::Seq};

/// seq's id is always 1
#[async_trait::async_trait(?Send)]
pub trait SeqInterface: Debug {
    async fn put(&self, seq: &Seq) -> Result<()>;

    async fn get(&self) -> Result<Seq>;
}
