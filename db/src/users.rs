use wasm_bindgen::JsValue;

use abi::model::user::User;

#[async_trait::async_trait(?Send)]
pub trait Users {
    async fn add(&self, user: &User);
    async fn get(&self, id: &str) -> Result<User, JsValue>;
}
