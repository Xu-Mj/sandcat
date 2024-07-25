use crate::{
    error::Result,
    model::group::{Group, GroupAndMembers, GroupDelete, GroupMember, GroupRequest},
    pb::message::{GroupInviteNew, GroupUpdate, RemoveMemberRequest},
};

#[async_trait::async_trait(?Send)]
pub trait GroupApi {
    async fn create(&self, data: GroupRequest, user_id: &str) -> Result<Group>;

    async fn invite(&self, data: GroupInviteNew) -> Result<()>;

    async fn remove_mem(&self, data: &RemoveMemberRequest) -> Result<()>;

    async fn delete(&self, data: GroupDelete) -> Result<()>;

    async fn update(&self, user_id: &str, group: GroupUpdate) -> Result<Group>;

    async fn get_by_id(&self, user_id: &str, group_id: &str) -> Result<Group>;

    async fn get_with_members(&self, user_id: &str, group_id: &str) -> Result<GroupAndMembers>;

    async fn get_members(
        &self,
        user_id: &str,
        group_id: &str,
        mem_ids: Vec<String>,
    ) -> Result<Vec<GroupMember>>;
}
