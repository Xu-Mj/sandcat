use async_trait::async_trait;

use crate::error::Result;
use crate::model::user::LoginResp;

#[async_trait(?Send)]
pub trait OAuth2Api {
    async fn github(&self, code: &str, state: &str) -> Result<LoginResp>;
    async fn wechat(&self, code: &str, state: &str) -> Result<LoginResp>;
}
