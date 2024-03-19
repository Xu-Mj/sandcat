use wasm_bindgen::JsValue;
use yew::AttrValue;

use crate::model::user::{User, UserRegister};

#[async_trait::async_trait(?Send)]
pub trait UserApi {
    async fn send_mail(&self, email: String) -> Result<(), JsValue>;
    async fn register(&self, register: UserRegister) -> Result<AttrValue, JsValue>;
    async fn search_friend(&self, pattern: String, search_user: &str)
        -> Result<Vec<User>, JsValue>;
}
