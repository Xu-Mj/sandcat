use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Seq {
    pub seq: i64,
}

#[async_trait(?Send)]
pub trait SeqApi {
    async fn get_seq(&self, user_id: &str) -> Result<Seq, JsValue>;
}
