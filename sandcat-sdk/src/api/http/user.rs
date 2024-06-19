use gloo_net::http::Request;
use serde::Serialize;

use crate::model::user::{
    LoginRequest, LoginResp, User, UserRegister, UserUpdate, UserWithMatchType,
};

use crate::api::user::UserApi;
use crate::error::Result;

use super::RespStatus;
pub struct UserHttp<'a> {
    token: Box<dyn Fn() -> String + 'a>,
    auth_header: String,
}

impl<'a> UserHttp<'a> {
    pub fn new(token: impl Fn() -> String + 'a, auth_header: String) -> Self {
        Self {
            token: Box::new(token),
            auth_header,
        }
    }
    pub fn get_token(&self) -> String {
        (self.token)()
    }
}

#[derive(Serialize, Debug)]
pub struct MailRequest {
    pub email: String,
}

#[async_trait::async_trait(?Send)]
impl<'a> UserApi for UserHttp<'a> {
    /// 向指定邮箱中发送邮件
    async fn send_mail(&self, email: String) -> Result<()> {
        log::debug!("send mail to {:?}", &email);
        Request::post("/api/user/mail/send")
            .json(&MailRequest { email })?
            .send()
            .await?
            .success()?;
        Ok(())
    }

    /// 用户注册
    async fn register(&self, register: UserRegister) -> Result<()> {
        Request::post("/api/user")
            .json(&register)?
            .send()
            .await?
            .success()?;
        Ok(())
    }

    async fn update(&self, user: UserUpdate) -> Result<User> {
        let user = Request::put("/api/user")
            .json(&user)?
            .send()
            .await?
            .success()?
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
            .header(&self.auth_header, &self.get_token())
            .send()
            .await?
            .success()?
            .json()
            .await?;
        Ok(friend)
    }

    async fn sign_in(&self, req: LoginRequest) -> Result<LoginResp> {
        let resp = Request::post("/api/user/login")
            .json(&req)?
            .send()
            .await?
            .success()?
            .json()
            .await?;
        Ok(resp)
    }

    async fn sign_out(&self, user_id: &str) -> Result<()> {
        Request::delete(format!("/api/user/{}", user_id).as_str())
            .header(&self.auth_header, &self.get_token())
            .send()
            .await?
            .success()?;
        Ok(())
    }

    async fn refresh_token(&self, token: &str, is_refresh: bool) -> Result<String> {
        let token = Request::get(format!("/api/user/refresh_token/{token}/{is_refresh}").as_ref())
            .send()
            .await?
            .success()?
            .text()
            .await?;
        Ok(token)
    }
}

/* // 根据id查询用户信息
#[allow(dead_code)]
pub async fn get_info_by_id(id: String) -> Result<User, JsValue> {
    Request::get(format!("/api/user/{}", id).as_str())
        .header(&self.auth_header, &self.token)
        .send()
        .await
        .map_err(|err| JsValue::from(err.to_string()))?
        .json()
        .await
        .map_err(|err| JsValue::from(err.to_string()))
}

// 获取好友请求列表
#[allow(dead_code)]
pub async fn get_friend_apply_list_by_id(id: String) -> Result<Vec<FriendShipWithUser>, JsValue> {
    Request::get(format!("/api/friend/{}/apply", id).as_str())
        .header(&self.auth_header, &self.token)
        .send()
        .await
        .map_err(|err| JsValue::from(err.to_string()))?
        .json()
        .await
        .map_err(|err| JsValue::from(err.to_string()))
}
 */
