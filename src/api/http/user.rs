use crate::api::user::UserApi;
use crate::model::user::{User, UserRegister};
use gloo_net::http::Request;
use serde::Serialize;
use wasm_bindgen::JsValue;
use yew::AttrValue;
pub struct UserHttp {
    token: String,
    auth_header: String,
}

impl UserHttp {
    pub fn new(token: String, auth_header: String) -> Self {
        Self { token, auth_header }
    }
}

#[derive(Serialize, Debug)]
pub struct MailRequest {
    pub email: String,
}

#[async_trait::async_trait(?Send)]
impl UserApi for UserHttp {
    // 查找好友
    async fn search_friend(
        &self,
        pattern: String,
        search_user: &str,
    ) -> Result<Vec<User>, JsValue> {
        let friends: Vec<User> =
            Request::get(format!("/api/user/{}/search/{}", search_user, pattern).as_str())
                .header(&self.auth_header, &self.token)
                .send()
                .await
                .map_err(|err| JsValue::from(err.to_string()))?
                .json()
                .await
                .map_err(|err| JsValue::from(err.to_string()))?;
        Ok(friends)
    }

    /// 向指定邮箱中发送邮件
    async fn send_mail(&self, email: String) -> Result<(), JsValue> {
        log::debug!("send mail to {:?}", &email);
        Request::post("/api/user/mail/send")
            .json(&MailRequest { email })
            .map_err(|err| JsValue::from(err.to_string()))?
            .send()
            .await
            .map_err(|err| JsValue::from(err.to_string()))?;
        Ok(())
    }

    /// 用户注册
    async fn register(&self, register: UserRegister) -> Result<AttrValue, JsValue> {
        Request::post("/api/user")
            .json(&register)
            .map_err(|err| JsValue::from(err.to_string()))?
            .send()
            .await
            .map_err(|err| JsValue::from(err.to_string()))?;
        Ok(AttrValue::from("value"))
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
