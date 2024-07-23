use gloo_net::http::Request;

use crate::api::group::GroupApi;
use crate::api::{token, AUTHORIZE_HEADER};
use crate::error::Result;
use crate::pb::message::{GroupInviteNew, RemoveMemberRequest};
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
}
