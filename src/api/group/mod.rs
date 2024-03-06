use nanoid::nanoid;
// use gloo_net::http::Request;
use wasm_bindgen::JsValue;
use yew::AttrValue;

use crate::model::group::{Group, GroupRequest};

// use super::{token, AUTHORIZE_HEADER};

pub async fn create_group(user_id: AttrValue, data: GroupRequest) -> Result<Group, JsValue> {
    // Request::post(&format!("/api/group{}", user_id.as_str()))
    //     .header(AUTHORIZE_HEADER, token().as_str())
    //     .json(&ids)
    //     .map_err(|err| JsValue::from(err.to_string()))?
    //     .send()
    //     .await
    //     .map_err(|err| JsValue::from(err.to_string()))?
    //     .json()
    //     .await
    //     .map_err(|err| JsValue::from(err.to_string()))
    log::debug!("send create group reeques by {:?}", user_id);
    Ok(Group {
        id: nanoid!().into(),
        name: data.group_name.into(),
        avatar: "./images/avatars/avatar1.png".to_string().into(),
        members_id: data.members_id,
        create_time: chrono::Local::now().timestamp_millis(),
        publish_msg: "test publish message".to_string().into(),
    })
}
