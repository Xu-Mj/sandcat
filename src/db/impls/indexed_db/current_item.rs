// use local storage to store current conv info and current friend info
use gloo::utils::window;
use wasm_bindgen::JsValue;

use crate::{
    i18n::LanguageType,
    model::{ComponentType, CurrentItem, UnreadItem},
};

pub const CONV_LOCAL_STORAGE_KEY: &str = "__CURRENT_CONV__";
pub const FRIEND_LOCAL_STORAGE_KEY: &str = "__CURRENT_FRIEND__";
pub const COMPONENT_TYPE_LOCAL_STORAGE_KEY: &str = "__CURRENT_COMPONENT__";
pub const UNREAD_COUNT_LOCAL_STORAGE_KEY: &str = "__UNREAD_COUNT__";
pub const LANGUAGE_LOCAL_STORAGE_KEY: &str = "__LANGUAGE__";

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

pub fn get_unread_count() -> UnreadItem {
    let value = window()
        .local_storage()
        .unwrap()
        .unwrap()
        .get(UNREAD_COUNT_LOCAL_STORAGE_KEY)
        .unwrap()
        .unwrap_or_default();
    serde_json::from_str(value.as_str()).unwrap_or_default()
}

pub fn save_unread_count(value: UnreadItem) -> Result<(), JsValue> {
    let value = serde_json::to_string(&value).unwrap();
    window()
        .local_storage()
        .unwrap()
        .unwrap()
        .set(UNREAD_COUNT_LOCAL_STORAGE_KEY, &value)
}

pub fn get_language() -> LanguageType {
    let value = window()
        .local_storage()
        .unwrap()
        .unwrap()
        .get(LANGUAGE_LOCAL_STORAGE_KEY)
        .unwrap()
        .unwrap_or_default();
    LanguageType::from(value)
}

pub fn save_language(value: LanguageType) -> Result<(), JsValue> {
    window()
        .local_storage()
        .unwrap()
        .unwrap()
        .set(LANGUAGE_LOCAL_STORAGE_KEY, &value.to_string())
}
