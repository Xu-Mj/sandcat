use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Seq {
    pub id: i64,
    pub sequence: i64,
}
