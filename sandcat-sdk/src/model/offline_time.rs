use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct OfflineTime {
    pub id: i64,
    pub time: i64,
}

impl OfflineTime {
    pub fn new(time: i64) -> Self {
        Self { id: 1, time }
    }
}
