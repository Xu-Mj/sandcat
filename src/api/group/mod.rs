// use gloo_net::http::Request;
use wasm_bindgen::JsValue;
use yew::AttrValue;

use crate::model::group::Group;

// use super::{token, AUTHORIZE_HEADER};

pub async fn create_group(user_id: AttrValue, ids: Vec<String>) -> Result<Group, JsValue> {
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

    Ok(Group {
        id: "123".to_string().into(),
        name: format!("{}、test_group", user_id).into(),
        avatar: "./images/avatars/avatar1.png".to_string().into(),
        members_id: ids,
        create_time: chrono::Local::now().timestamp_millis(),
        publish_msg: "test publish message".to_string().into(),
    })
}
