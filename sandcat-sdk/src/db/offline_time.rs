use async_trait::async_trait;
use std::fmt::Debug;

use crate::{error::Result, model::offline_time::OfflineTime};

#[async_trait(?Send)]
pub trait OfflineTimes: Debug {
    async fn save(&self, obj: &OfflineTime) -> Result<()>;

    async fn get(&self) -> Result<Option<OfflineTime>>;

    async fn del(&self) -> Result<()>;
}
