use crate::model::ContentType;
use crate::model::RightContentType;
use serde::{Deserialize, Serialize};
use yew::AttrValue;
fn is_zero(id: &i32) -> bool {
    *id == 0
}
/// 会话表
///
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct Conversation {
    #[serde(skip_serializing_if = "is_zero")]
    pub id: i32,
    pub conv_type: RightContentType,
    pub friend_id: AttrValue,
    pub name: AttrValue,
    pub avatar: AttrValue,
    pub last_msg: AttrValue,
    // 需要根据时间来排序
    pub last_msg_time: i64,
    pub last_msg_type: ContentType,
    pub unread_count: u16,
}
