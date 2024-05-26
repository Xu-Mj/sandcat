use std::fmt::Debug;

use wasm_bindgen::JsValue;

use crate::model::voice::Voice;

#[async_trait::async_trait(?Send)]
pub trait Voices: Debug {
    async fn save(&self, voice: &Voice) -> Result<(), JsValue>;

    async fn get(&self, local_id: &str) -> Result<Voice, JsValue>;
}
