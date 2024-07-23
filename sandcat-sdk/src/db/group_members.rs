use std::fmt::Debug;

use crate::error::Result;
use crate::model::group::GroupMember;

#[async_trait::async_trait(?Send)]
pub trait GroupMembers: Debug {
    async fn put(&self, mem: &GroupMember) -> Result<()>;

    async fn put_list(&self, members: &[GroupMember]) -> Result<()>;

    async fn get(&self, id: i64) -> Result<Option<GroupMember>>;

    async fn get_by_group_id_and_friend_id(
        &self,
        group_id: &str,
        friend_id: &str,
    ) -> Result<Option<GroupMember>>;

    async fn get_list_by_group_id(&self, group_id: &str) -> Result<Vec<GroupMember>>;

    async fn delete(&self, group_id: &str, user_id: &str) -> Result<()>;
}
