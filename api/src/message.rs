use wasm_bindgen::JsValue;

use sandcat_sdk::pb::message::Msg;

#[async_trait::async_trait(?Send)]
pub trait MsgApi {
    async fn pull_offline_msg(
        &self,
        user_id: &str,
        start: i64,
        end: i64,
    ) -> Result<Vec<Msg>, JsValue>;

    async fn del_msg(&self, user_id: &str, msg_id: Vec<String>) -> Result<(), JsValue>;
}
