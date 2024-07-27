use crate::error::Result;
use crate::model::user::{
    LoginRequest, LoginResp, User, UserRegister, UserUpdate, UserWithMatchType,
};

#[async_trait::async_trait(?Send)]
pub trait UserApi {
    async fn send_mail(&self, email: String) -> Result<()>;

    async fn register(&self, register: UserRegister) -> Result<()>;

    async fn update(&self, user: UserUpdate) -> Result<User>;

    async fn search_friend(
        &self,
        pattern: String,
        search_user: &str,
    ) -> Result<Option<UserWithMatchType>>;

    async fn sign_in(&self, req: LoginRequest) -> Result<LoginResp>;

    async fn sign_out(&self, user_id: &str) -> Result<()>;

    async fn refresh_token(&self, token: &str, is_refresh: bool) -> Result<String>;

    async fn change_pwd(
        &self,
        email: String,
        user_id: String,
        pwd: String,
        code: String,
    ) -> Result<()>;
}
