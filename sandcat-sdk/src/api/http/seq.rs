use async_trait::async_trait;
use gloo_net::http::Request;

use crate::api::seq::{Seq, SeqApi};
use crate::api::{token, AUTHORIZE_HEADER};
use crate::error::Result;

use super::RespStatus;

pub struct SeqHttp;

#[async_trait(?Send)]
impl SeqApi for SeqHttp {
    async fn get_seq(&self, user_id: &str) -> Result<Seq> {
        let seq = Request::get(format!("/api/message/seq/{}", user_id).as_str())
            .header(AUTHORIZE_HEADER, &token())
            .send()
            .await?
            .success()
            .await?
            .json()
            .await?;
        Ok(seq)
    }
}
