use crate::api::{token, AUTHORIZE_HEADER};
use crate::model::friend::{Friend, FriendShipAgree, FriendShipRequest, FriendShipWithUser};
use crate::model::user::{User, UserRegister};
use gloo_net::http::Request;
use serde::Serialize;
use wasm_bindgen::JsValue;
use yew::AttrValue;

// 根据id查询用户信息
#[allow(dead_code)]
pub async fn get_info_by_id(id: String) -> Result<User, JsValue> {
    Request::get(format!("/api/user/{}", id).as_str())
        .header(AUTHORIZE_HEADER, token().as_str())
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
        .header(AUTHORIZE_HEADER, token().as_str())
        .send()
        .await
        .map_err(|err| JsValue::from(err.to_string()))?
        .json()
        .await
        .map_err(|err| JsValue::from(err.to_string()))
}

// 获取好友列表, 服务端需要增加好友表及其逻辑，包括好友请求表，实际好友关系表（因为需要额外字段：备注，添加时间等）
pub async fn get_friend_list_by_id(id: String) -> Result<Vec<Friend>, JsValue> {
    let friends: Vec<Friend> = Request::get(format!("/api/friend/{}", id).as_str())
        .header(AUTHORIZE_HEADER, token().as_str())
        .send()
        .await
        .map_err(|err| JsValue::from(err.to_string()))?
        .json()
        .await
        .map_err(|err| JsValue::from(err.to_string()))?;
    Ok(friends)
}

// 查找好友
pub async fn search_friend(pattern: String, search_user: AttrValue) -> Result<Vec<User>, JsValue> {
    let friends: Vec<User> =
        Request::get(format!("/api/user/{}/search/{}", search_user.as_str(), pattern).as_str())
            .header(AUTHORIZE_HEADER, token().as_str())
            .send()
            .await
            .map_err(|err| JsValue::from(err.to_string()))?
            .json()
            .await
            .map_err(|err| JsValue::from(err.to_string()))?;
    Ok(friends)
}

// 请求添加好友
pub async fn apply_friend(new_friend: FriendShipRequest) -> Result<FriendShipWithUser, JsValue> {
    let friendship: FriendShipWithUser = Request::post("/api/friend")
        .header(AUTHORIZE_HEADER, token().as_str())
        .json(&new_friend)
        .map_err(|err| JsValue::from(err.to_string()))?
        .send()
        .await
        .map_err(|err| JsValue::from(err.to_string()))?
        .json()
        .await
        .map_err(|err| JsValue::from(err.to_string()))?;
    Ok(friendship)
}

// 同意好友请求
pub async fn agree_friend(friendship: FriendShipAgree) -> Result<Friend, JsValue> {
    let friend: Friend = Request::put("/api/friend/agree")
        .header(AUTHORIZE_HEADER, token().as_str())
        .json(&friendship)
        .map_err(|err| JsValue::from(err.to_string()))?
        .send()
        .await
        .map_err(|err| {
            log::debug!("error: {:?}", &err);
            JsValue::from(err.to_string())
        })?
        .json()
        .await
        .map_err(|err| {
            log::debug!("error: {:?}", &err);
            JsValue::from(err.to_string())
        })?;
    Ok(friend)
}

#[derive(Serialize, Debug)]
pub struct MailRequest {
    pub email: String,
}
/// 向指定邮箱中发送邮件
pub async fn send_mail(email: String) -> Result<(), JsValue> {
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
pub async fn register(register: UserRegister) -> Result<AttrValue, JsValue> {
    Request::post("/api/user")
        .json(&register)
        .map_err(|err| JsValue::from(err.to_string()))?
        .send()
        .await
        .map_err(|err| JsValue::from(err.to_string()))?;
    Ok(AttrValue::from("value"))
}
