use async_trait::async_trait;
use wasm_bindgen::JsValue;
use web_sys::File;

#[async_trait(?Send)]
pub trait FileApi {
    async fn upload_file(&self, file: File) -> Result<String, JsValue>;
}
