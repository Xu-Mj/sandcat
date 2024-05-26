use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Voice {
    pub local_id: String,
    pub data: Vec<u8>,
    pub duration: i64,
}
