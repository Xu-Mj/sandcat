use indexmap::IndexMap;
use wasm_bindgen::JsValue;
use yew::AttrValue;

use crate::model::group::Group;

pub trait GroupInterface {
    async fn new() -> Self
    where
        Self: Sized;

    async fn put(&self, group: &Group) -> Result<(), JsValue>;

    async fn get(&self, id: AttrValue) -> Result<Option<Group>, JsValue>;

    async fn get_list(&self) -> Result<IndexMap<AttrValue, Group>, JsValue>;

    async fn delete(&self, id: AttrValue) -> Result<(), JsValue>;
}
