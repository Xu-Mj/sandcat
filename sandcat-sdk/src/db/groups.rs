use std::fmt::Debug;

use indexmap::IndexMap;
use yew::AttrValue;

use crate::error::Result;
use crate::model::group::Group;

#[async_trait::async_trait(?Send)]
pub trait GroupInterface: Debug {
    async fn put(&self, group: &Group) -> Result<()>;

    async fn get(&self, id: &str) -> Result<Option<Group>>;

    async fn get_list(&self) -> Result<IndexMap<AttrValue, Group>>;

    async fn delete(&self, id: &str) -> Result<()>;

    async fn dismiss(&self, id: &str) -> Result<Group>;
}
