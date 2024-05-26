use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Voice {
    pub local_id: String,
    pub data: Vec<u8>,
    pub duration: u8,
}

impl Voice {
    pub fn new(local_id: String, data: Vec<u8>, duration: u8) -> Self {
        Self {
            local_id,
            data,
            duration,
        }
    }
}
