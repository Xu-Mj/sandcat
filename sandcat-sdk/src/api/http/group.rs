use gloo_net::http::Request;
use wasm_bindgen::JsValue;

use crate::api::group::GroupApi;
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
    async fn create(&self, data: GroupRequest, user_id: &str) -> Result<Group, JsValue> {
        let response: GroupInvitation = Request::post(format!("/api/group/{}", user_id).as_str())
            .header(&self.auth_header, &self.token)
            .json(&data)
            .map_err(|err| JsValue::from(err.to_string()))?
            .send()
            .await
            .map_err(|err| JsValue::from(err.to_string()))?
            .success()?
            .json()
            .await
            .map_err(|err| JsValue::from(err.to_string()))?;
        // log::debug!("send create group reeques by {:?}", user_id);
        Ok(Group::from(response.info.unwrap()))
    }

    async fn delete(&self, data: GroupDelete) -> Result<(), JsValue> {
        Request::delete("/api/group")
            .header(&self.auth_header, &self.token)
            .json(&data)
            .map_err(|err| JsValue::from(err.to_string()))?
            .send()
            .await
            .map_err(|err| JsValue::from(err.to_string()))?
            .success()?;

        Ok(())
    }

    async fn update(&self, user_id: &str, data: GroupUpdate) -> Result<Group, JsValue> {
        let group: GroupFromServer = Request::put(format!("/api/group/{}", user_id).as_str())
            .header(&self.auth_header, &self.token)
            .json(&data)
            .map_err(|err| JsValue::from(err.to_string()))?
            .send()
            .await
            .map_err(|err| JsValue::from(err.to_string()))?
            .success()?
            .json()
            .await
            .map_err(|err| JsValue::from(err.to_string()))?;

        Ok(Group::from(group))
    }

    async fn invite(&self, data: GroupInviteNew) -> Result<(), JsValue> {
        Request::post("/api/group/invite")
            .header(&self.auth_header, &self.token)
            .json(&data)
            .map_err(|err| JsValue::from(err.to_string()))?
            .send()
            .await
            .map_err(|err| JsValue::from(err.to_string()))?
            .success()?;
        Ok(())
    }
}
