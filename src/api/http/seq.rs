use async_trait::async_trait;
use reqwasm::http::Request;
use wasm_bindgen::JsValue;

use crate::api::seq::{Seq, SeqApi};

pub struct SeqHttp {
    token: String,
    auth_header: String,
}

impl SeqHttp {
    pub fn new(token: String, auth_header: String) -> Self {
        Self { token, auth_header }
    }
}

#[async_trait(?Send)]
impl SeqApi for SeqHttp {
    async fn get_seq(&self, user_id: &str) -> Result<Seq, JsValue> {
        let resp = Request::get(format!("/api/message/seq/{}", user_id).as_str())
            .header(&self.auth_header, &self.token)
            .send()
            .await
            .map_err(|err| JsValue::from(err.to_string()))?;
        if resp.status() != 200 {
            return Err(JsValue::from("获取seq失败"));
        }
        let seq = resp
            .json()
            .await
            .map_err(|err| JsValue::from(err.to_string()))?;
        Ok(seq)
    }
}
