use gloo_net::http::Request;
use wasm_bindgen::JsValue;
use yew::AttrValue;

use crate::model::{
    group::{Group, GroupRequest},
    message::CreateGroup,
};

use super::{token, AUTHORIZE_HEADER};

pub async fn create_group(data: GroupRequest, user_id: AttrValue) -> Result<Group, JsValue> {
    let response: CreateGroup = Request::post(format!("/api/group/{}", user_id).as_str())
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
