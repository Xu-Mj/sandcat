use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Seq {
    pub id: i64,
    pub local_seq: i64,
}
