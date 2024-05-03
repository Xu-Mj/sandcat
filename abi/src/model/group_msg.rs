use serde::{Deserialize, Serialize};
use yew::AttrValue;

use super::{ContentType, MessageItem};

fn is_zero(id: &i32) -> bool {
    *id == 0
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct GroupMsg {
    #[serde(skip_serializing_if = "is_zero")]
    #[serde(default)]
    pub id: i32,
    pub msg_id: AttrValue,
    pub send_id: AttrValue,
    // GROUP ID
    pub group_id: AttrValue,
    // 是MessageType类型，需要做转换
    // pub msg_type: MessageType,
    #[serde(default)]
    pub content_type: ContentType,
    // 如果是文件类型，那么content存储文件的路径
    pub content: AttrValue,
    #[serde(default)]
    pub create_time: i64,
    // don't offer read flag for now
    // #[serde(default)]
    // pub is_read: bool,
    #[serde(default)]
    pub is_self: bool,
    #[serde(skip)]
    pub file_content: AttrValue,
}

impl MessageItem for GroupMsg {
    fn id(&self) -> AttrValue {
        self.msg_id.clone()
    }

    fn msg(&self) -> AttrValue {
        self.content.clone()
    }

    fn time(&self) -> i64 {
        self.create_time
    }

    fn send_id(&self) -> AttrValue {
        self.send_id.clone()
    }

    fn content_type(&self) -> ContentType {
        self.content_type
    }

    fn is_self(&self) -> bool {
        self.is_self
    }
}
