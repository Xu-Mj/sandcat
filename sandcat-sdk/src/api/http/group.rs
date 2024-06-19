use gloo_net::http::Request;

use crate::api::group::GroupApi;
use crate::error::Result;
use crate::pb::message::GroupInviteNew;
use crate::{
    model::{
        group::{Group, GroupDelete, GroupFromServer, GroupRequest},
        message::GroupInvitation,
    },
    pb::message::GroupUpdate,
};

use super::RespStatus;

pub struct GroupHttp {
    token: String,
    auth_header: String,
}

impl GroupHttp {
    pub fn new(token: String, auth_header: String) -> Self {
        Self { token, auth_header }
    }
}

#[async_trait::async_trait(?Send)]
impl GroupApi for GroupHttp {
    async fn create(&self, data: GroupRequest, user_id: &str) -> Result<Group> {
        let response: GroupInvitation = Request::post(format!("/api/group/{}", user_id).as_str())
            .header(&self.auth_header, &self.token)
            .json(&data)?
            .send()
            .await?
            .success()?
            .json()
            .await?;
        Ok(Group::from(response.info.unwrap()))
    }

    async fn invite(&self, data: GroupInviteNew) -> Result<()> {
        Request::post("/api/group/invite")
            .header(&self.auth_header, &self.token)
            .json(&data)?
            .send()
            .await?
            .success()?;
        Ok(())
    }

    async fn delete(&self, data: GroupDelete) -> Result<()> {
        Request::delete("/api/group")
            .header(&self.auth_header, &self.token)
            .json(&data)?
            .send()
            .await?
            .success()?;

        Ok(())
    }

    async fn update(&self, user_id: &str, data: GroupUpdate) -> Result<Group> {
        let group: GroupFromServer = Request::put(format!("/api/group/{}", user_id).as_str())
            .header(&self.auth_header, &self.token)
            .json(&data)?
            .send()
            .await?
            .success()?
            .json()
            .await?;

        Ok(Group::from(group))
    }
}
