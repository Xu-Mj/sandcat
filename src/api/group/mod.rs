use gloo_net::http::Request;
use wasm_bindgen::JsValue;
use yew::AttrValue;

use crate::model::group::{Group, GroupRequest};

use super::{token, AUTHORIZE_HEADER};

pub async fn create_group(data: GroupRequest) -> Result<Group, JsValue> {
    let response: GroupRequest = Request::post("/api/group")
        .header(AUTHORIZE_HEADER, token().as_str())
        .json(&data)
        .map_err(|err| JsValue::from(err.to_string()))?
        .send()
        .await
        .map_err(|err| JsValue::from(err.to_string()))?
        .json()
        .await
        .map_err(|err| JsValue::from(err.to_string()))?;
    // log::debug!("send create group reeques by {:?}", user_id);
    Ok(Group {
        id: response.id.into(),
        name: response.group_name.into(),
        avatar: response.avatar.join(",").into(),
        members_id: data.members_id,
        create_time: chrono::Local::now().timestamp_millis(),
        publish_msg: AttrValue::default(),
    })
}
