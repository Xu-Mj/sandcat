use gloo_net::http::Request;

use crate::api::group::GroupApi;
use crate::api::{token, AUTHORIZE_HEADER};
use crate::error::Result;
use crate::model::group::{GroupAndMembers, GroupMember};
use crate::pb::message::{GetMemberReq, GroupInviteNew, RemoveMemberRequest};
use crate::{
    model::{
        group::{Group, GroupDelete, GroupFromServer, GroupRequest},
        message::GroupInvitation,
    },
    pb::message::GroupUpdate,
};

use super::RespStatus;

pub struct GroupHttp;

#[async_trait::async_trait(?Send)]
impl GroupApi for GroupHttp {
    async fn create(&self, data: GroupRequest, user_id: &str) -> Result<Group> {
        let response: GroupInvitation = Request::post(format!("/api/group/{}", user_id).as_str())
            .header(AUTHORIZE_HEADER, &token())
            .json(&data)?
            .send()
            .await?
            .success()?
            .json()
            .await?;
        Ok(Group::from(response.info.unwrap()))
    }

    async fn invite(&self, data: GroupInviteNew) -> Result<()> {
        Request::put("/api/group/invite")
            .header(AUTHORIZE_HEADER, &token())
            .json(&data)?
            .send()
            .await?
            .success()?;
        Ok(())
    }
    async fn remove_mem(&self, data: RemoveMemberRequest) -> Result<()> {
        Request::delete("/api/group/member")
            .header(AUTHORIZE_HEADER, &token())
            .json(&data)?
            .send()
            .await?
            .success()?;
        Ok(())
    }

    async fn delete(&self, data: GroupDelete) -> Result<()> {
        Request::delete("/api/group")
            .header(AUTHORIZE_HEADER, &token())
            .json(&data)?
            .send()
            .await?
            .success()?;

        Ok(())
    }

    async fn update(&self, user_id: &str, data: GroupUpdate) -> Result<Group> {
        let group: GroupFromServer = Request::put(format!("/api/group/{}", user_id).as_str())
            .header(AUTHORIZE_HEADER, &token())
            .json(&data)?
            .send()
            .await?
            .success()?
            .json()
            .await?;

        Ok(Group::from(group))
    }

    async fn get_by_id(&self, user_id: &str, group_id: &str) -> Result<Group> {
        let resp = Request::get(format!("/api/group/{user_id}/{group_id}").as_str())
            .header(AUTHORIZE_HEADER, &token())
            .send()
            .await?
            .success()?
            .json()
            .await?;
        Ok(resp)
    }

    async fn get_with_members(&self, user_id: &str, group_id: &str) -> Result<GroupAndMembers> {
        let resp = Request::get(format!("/api/group/member/{user_id}/{group_id}").as_str())
            .header(AUTHORIZE_HEADER, &token())
            .send()
            .await?
            .success()?
            .json()
            .await?;
        Ok(resp)
    }

    async fn get_members(
        &self,
        user_id: &str,
        group_id: &str,
        mem_ids: Vec<String>,
    ) -> Result<Vec<GroupMember>> {
        let req = GetMemberReq {
            group_id: group_id.to_string(),
            user_id: user_id.to_string(),
            mem_ids,
        };
        let resp = Request::post("/api/group/member")
            .header(AUTHORIZE_HEADER, &token())
            .json(&req)?
            .send()
            .await?
            .success()?
            .json()
            .await?;
        Ok(resp)
    }
}
