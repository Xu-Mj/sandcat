use gloo_net::http::Request;
use wasm_bindgen::JsValue;

use crate::model::{
    group::{Group, GroupDelete, GroupRequest},
    message::GroupInvitation,
};

use super::{token, AUTHORIZE_HEADER};

pub async fn create_group(data: GroupRequest, user_id: &str) -> Result<Group, JsValue> {
    let response: GroupInvitation = Request::post(format!("/api/group/{}", user_id).as_str())
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
    Ok(response.info)
}

pub async fn delete_group(data: GroupDelete) -> Result<(), JsValue> {
    Request::delete("/api/group")
        .header(AUTHORIZE_HEADER, token().as_str())
        .json(&data)
        .map_err(|err| JsValue::from(err.to_string()))?
        .send()
        .await
        .map_err(|err| JsValue::from(err.to_string()))?;
    log::debug!("send delete group reequest {:?}", data);
    Ok(())
}
