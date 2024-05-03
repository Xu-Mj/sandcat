use gloo_net::http::Request;
use wasm_bindgen::JsValue;

use crate::group::GroupApi;
use abi::model::{
    group::{Group, GroupDelete, GroupRequest},
    message::GroupInvitation,
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
    async fn create_group(&self, data: GroupRequest, user_id: &str) -> Result<Group, JsValue> {
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

    async fn delete_group(&self, data: GroupDelete) -> Result<(), JsValue> {
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
}
