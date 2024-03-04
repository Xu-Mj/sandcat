// use local storage to store current conv info and current friend info

use crate::pages::{ComponentType, CurrentItem};
use gloo::utils::window;
use wasm_bindgen::JsValue;

pub const CONV_LOCAL_STORAGE_KEY: &str = "__CURRENT_CONV__";
pub const FRIEND_LOCAL_STORAGE_KEY: &str = "__CURRENT_FRIEND__";
pub const COMPONENT_TYPE_LOCAL_STORAGE_KEY: &str = "__CURRENT_COMPONENT__";
// pub const UNREAD_MSG_COUNT_LOCAL_STORAGE_KEY: &str = "__UNREAD_MSG_COUNT__";

pub fn save_conv(conv: &CurrentItem) -> Result<(), JsValue> {
    let value = serde_json::to_string(conv).unwrap();
    window()
        .local_storage()
        .unwrap()
        .unwrap()
        .set(CONV_LOCAL_STORAGE_KEY, value.as_str())
}

pub fn get_conv() -> CurrentItem {
    let value = window()
        .local_storage()
        .unwrap()
        .unwrap()
        .get(CONV_LOCAL_STORAGE_KEY)
        .unwrap()
        .unwrap_or_default();
    serde_json::from_str(value.as_str()).unwrap_or_default()
}

pub fn save_friend(conv: &CurrentItem) -> Result<(), JsValue> {
    let value = serde_json::to_string(conv).unwrap();
    window()
        .local_storage()
        .unwrap()
        .unwrap()
        .set(FRIEND_LOCAL_STORAGE_KEY, value.as_str())
}

pub fn get_friend() -> CurrentItem {
    let value = window()
        .local_storage()
        .unwrap()
        .unwrap()
        .get(FRIEND_LOCAL_STORAGE_KEY)
        .unwrap()
        .unwrap_or_default();
    serde_json::from_str(value.as_str()).unwrap_or_default()
}

pub fn save_com_type(com: &ComponentType) -> Result<(), JsValue> {
    let value = serde_json::to_string(com).unwrap();
    window()
        .local_storage()
        .unwrap()
        .unwrap()
        .set(COMPONENT_TYPE_LOCAL_STORAGE_KEY, value.as_str())
}

pub fn get_com_type() -> ComponentType {
    let value = window()
        .local_storage()
        .unwrap()
        .unwrap()
        .get(COMPONENT_TYPE_LOCAL_STORAGE_KEY)
        .unwrap()
        .unwrap_or_default();
    serde_json::from_str(value.as_str()).unwrap_or_default()
}

/* pub fn get_unread_count() -> usize {
    let value = window()
        .local_storage()
        .unwrap()
        .unwrap()
        .get(UNREAD_MSG_COUNT_LOCAL_STORAGE_KEY)
        .unwrap()
        .unwrap_or_default();
    if value.is_empty() {
        return 0;
    }
    value.parse::<usize>().unwrap()
}

pub fn save_unread_count(count: usize) -> Result<(), JsValue> {
    window().local_storage().unwrap().unwrap().set(
        UNREAD_MSG_COUNT_LOCAL_STORAGE_KEY,
        count.to_string().as_str(),
    )
}
 */
