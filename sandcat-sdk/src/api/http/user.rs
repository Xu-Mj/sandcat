use base64::prelude::BASE64_STANDARD_NO_PAD;
use base64::Engine;
use gloo_net::http::Request;
use serde::Serialize;

use crate::api::{token, AUTHORIZE_HEADER};
use crate::model::user::{
    LoginRequest, LoginResp, User, UserRegister, UserUpdate, UserWithMatchType,
};

use crate::api::user::UserApi;
use crate::error::Result;

use super::RespStatus;
pub struct UserHttp;

#[derive(Serialize, Debug)]
pub struct MailRequest {
    pub email: String,
}

#[derive(Serialize, Debug)]
pub struct ChangePwdRequest {
    pub email: String,
    pub user_id: String,
    pub pwd: String,
    pub code: String,
}

#[async_trait::async_trait(?Send)]
impl UserApi for UserHttp {
    /// 向指定邮箱中发送邮件
    async fn send_mail(&self, email: String) -> Result<()> {
        log::debug!("send mail to {:?}", &email);
        Request::post("/api/user/mail/send")
            .json(&MailRequest { email })?
            .send()
            .await?
            .success()
            .await?;
        Ok(())
    }

    /// 用户注册
    async fn register(&self, register: UserRegister) -> Result<()> {
        Request::post("/api/user")
            .json(&register)?
            .send()
            .await?
            .success()
            .await?;
        Ok(())
    }

    async fn update(&self, user: UserUpdate) -> Result<User> {
        let user = Request::put("/api/user")
            .header(AUTHORIZE_HEADER, &token())
            .json(&user)?
            .send()
            .await?
            .success()
            .await?
            .json()
            .await?;
        Ok(user)
    }

    // 查找好友
    async fn search_friend(
        &self,
        pattern: String,
        search_user: &str,
    ) -> Result<Option<UserWithMatchType>> {
        let friend = Request::get(format!("/api/user/{}/search/{}", search_user, pattern).as_str())
            .header(AUTHORIZE_HEADER, &token())
            .send()
            .await?
            .success()
            .await?
            .json()
            .await?;
        Ok(friend)
    }

    async fn sign_in(&self, req: LoginRequest) -> Result<LoginResp> {
        let resp = Request::post("/api/user/login")
            .json(&req)?
            .send()
            .await?
            .success()
            .await?
            .json()
            .await?;
        Ok(resp)
    }

    async fn sign_out(&self, user_id: &str) -> Result<()> {
        Request::delete(format!("/api/user/{}", user_id).as_str())
            .header(AUTHORIZE_HEADER, &token())
            .send()
            .await?
            .success()
            .await?;
        Ok(())
    }

    async fn refresh_token(&self, token: &str, is_refresh: bool) -> Result<String> {
        let token = Request::get(format!("/api/user/refresh_token/{token}/{is_refresh}").as_ref())
            .send()
            .await?
            .success()
            .await?
            .text()
            .await?;
        Ok(token)
    }

    async fn change_pwd(
        &self,
        email: String,
        user_id: String,
        pwd: String,
        code: String,
    ) -> Result<()> {
        let pwd = BASE64_STANDARD_NO_PAD.encode(&pwd);
        Request::put("/api/user/pwd")
            .json(&ChangePwdRequest {
                email,
                user_id,
                pwd,
                code,
            })?
            .send()
            .await?
            .success()
            .await?;
        Ok(())
    }
}
