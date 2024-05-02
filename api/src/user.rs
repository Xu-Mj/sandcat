use wasm_bindgen::JsValue;

use abi::model::user::{User, UserRegister, UserUpdate, UserWithMatchType};

#[async_trait::async_trait(?Send)]
pub trait UserApi {
    async fn send_mail(&self, email: String) -> Result<(), JsValue>;

    async fn register(&self, register: UserRegister) -> Result<(), JsValue>;

    async fn update(&self, user: UserUpdate) -> Result<User, JsValue>;

    async fn search_friend(
        &self,
        pattern: String,
        search_user: &str,
    ) -> Result<Option<UserWithMatchType>, JsValue>;
}
