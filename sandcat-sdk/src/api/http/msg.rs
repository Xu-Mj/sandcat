use async_trait::async_trait;
use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

use crate::api::message::MsgApi;
use crate::api::{token, AUTHORIZE_HEADER};
use crate::error::Result;
use crate::pb::message::Msg;

use super::RespStatus;

pub struct MsgHttp;

#[derive(Debug, Serialize, Deserialize)]
pub struct PullOfflineMsgReq {
    pub user_id: String,
    pub send_start: i64,
    pub send_end: i64,
    pub start: i64,
    pub end: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DelMsgReq {
    pub user_id: String,
    pub msg_id: Vec<String>,
}

#[async_trait(?Send)]
impl MsgApi for MsgHttp {
    async fn pull_offline_msg(
        &self,
        user_id: &str,
        send_start: i64,
        send_end: i64,
        start: i64,
        end: i64,
    ) -> Result<Vec<Msg>> {
        let request = PullOfflineMsgReq {
            user_id: user_id.to_string(),
            send_start,
            send_end,
            start,
            end,
        };
        let messages = Request::post("/api/message")
            .header(AUTHORIZE_HEADER, &token())
            .json(&request)?
            .send()
            .await?
            .success()?
            .json()
            .await?;
        Ok(messages)
    }

    async fn del_msg(&self, user_id: &str, msg_id: Vec<String>) -> Result<()> {
        let request = DelMsgReq {
            user_id: user_id.to_string(),
            msg_id,
        };
        Request::delete("/api/message")
            .header(AUTHORIZE_HEADER, &token())
            .json(&request)?
            .send()
            .await?
            .success()?;
        Ok(())
    }
}
