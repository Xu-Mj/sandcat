use indexmap::IndexMap;
use wasm_bindgen::JsValue;
use yew::AttrValue;

use crate::model::group::Group;
#[async_trait::async_trait(?Send)]
pub trait GroupInterface {
    async fn put(&self, group: &Group) -> Result<(), JsValue>;

    async fn get(&self, id: &str) -> Result<Option<Group>, JsValue>;

    async fn get_list(&self) -> Result<IndexMap<AttrValue, Group>, JsValue>;

    async fn delete(&self, id: &str) -> Result<(), JsValue>;
}
