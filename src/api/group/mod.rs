use gloo_net::http::Request;
use wasm_bindgen::JsValue;
use yew::AttrValue;

use crate::model::group::Group;

use super::{token, AUTHORIZE_HEADER};

pub async fn create_group(user_id: AttrValue, ids: Vec<String>) -> Result<Group, JsValue> {
    Request::post(&format!("/api/group{}", user_id.as_str()))
        .header(AUTHORIZE_HEADER, token().as_str())
        .json(&ids)
        .map_err(|err| JsValue::from(err.to_string()))?
        .send()
        .await
        .map_err(|err| JsValue::from(err.to_string()))?
        .json()
        .await
        .map_err(|err| JsValue::from(err.to_string()))
}
