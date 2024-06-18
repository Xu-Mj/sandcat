use crate::error::Error;
use crate::model::user::{
    LoginRequest, LoginResp, User, UserRegister, UserUpdate, UserWithMatchType,
};

#[async_trait::async_trait(?Send)]
pub trait UserApi {
    async fn send_mail(&self, email: String) -> Result<(), Error>;

    async fn register(&self, register: UserRegister) -> Result<(), Error>;

    async fn update(&self, user: UserUpdate) -> Result<User, Error>;

    async fn search_friend(
        &self,
        pattern: String,
        search_user: &str,
    ) -> Result<Option<UserWithMatchType>, Error>;

    async fn sign_in(&self, req: LoginRequest) -> Result<LoginResp, Error>;

    async fn sign_out(&self, user_id: &str) -> Result<(), Error>;

    async fn refresh_token(&self, token: &str, is_refresh: bool) -> Result<String, Error>;
}
