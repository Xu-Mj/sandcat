use async_trait::async_trait;
use gloo_net::http::Request;

use crate::{api::oauth2::OAuth2Api, error::Result, model::user::LoginResp};

use super::RespStatus;

pub struct OAuth2Http;

#[async_trait(?Send)]
impl OAuth2Api for OAuth2Http {
    async fn github(&self, code: &str, state: &str) -> Result<LoginResp> {
        let resp = Request::get(&format!(
            "/api/user/auth/github/callback?code={}&state={}",
            code, state
        ))
        .send()
        .await?
        .success()?
        .json()
        .await?;
        Ok(resp)
    }
    async fn google(&self, code: &str, state: &str) -> Result<LoginResp> {
        let resp = Request::get(&format!(
            "/api/user/auth/google/callback?code={}&state={}",
            code, state
        ))
        .send()
        .await?
        .success()?
        .json()
        .await?;
        Ok(resp)
    }
}
