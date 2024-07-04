use crate::error::Result;
use crate::pb::message::Msg;

#[async_trait::async_trait(?Send)]
pub trait MsgApi {
    async fn pull_offline_msg(
        &self,
        user_id: &str,
        send_start: i64,
        send_end: i64,
        start: i64,
        end: i64,
    ) -> Result<Vec<Msg>>;

    async fn del_msg(&self, user_id: &str, msg_id: Vec<String>) -> Result<()>;
}
