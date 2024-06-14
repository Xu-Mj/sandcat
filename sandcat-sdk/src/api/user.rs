use wasm_bindgen::JsValue;

use crate::model::user::{
    LoginRequest, LoginResp, User, UserRegister, UserUpdate, UserWithMatchType,
};

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

    async fn signin(&self, req: LoginRequest) -> Result<LoginResp, JsValue>;

    async fn signout(&self, user_id: &str) -> Result<(), JsValue>;

    async fn refresh_token(&self, user_id: &str) -> Result<String, JsValue>;
}
