use serde::{Deserialize, Serialize};
use yew::AttrValue;

use super::{ItemInfo, RightContentType};
#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum FriendStatus {
    #[default]
    Pending,
    Accepted,
    Rejected,
    Blacked,
    Cancelled,
    Failed,
}

#[derive(Debug, Default, Serialize, Clone, Deserialize, PartialEq)]
pub struct FriendShipRequest {
    // #[serde(skip_serializing_if = "is_zero")]
    // #[serde(default)]
    // pub id: i32,
    pub user_id: AttrValue,
    pub friend_id: AttrValue,
    pub status: FriendStatus,
    pub apply_msg: Option<AttrValue>,
    pub source: Option<AttrValue>,
    pub remark: Option<AttrValue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FriendShipAgree {
    pub friendship_id: AttrValue,
    pub response_msg: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Default, Serialize, Clone, Deserialize, PartialEq)]
pub enum ReadStatus {
    #[default]
    False,
    True,
}

/// 用来接收服务端返回的好友信息
#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct Friend {
    pub id: AttrValue,
    pub friend_id: AttrValue,
    pub remark: Option<AttrValue>,
    /// 这里的hello是我们发送给对方的消息
    pub hello: Option<AttrValue>,
    pub status: FriendStatus,
    pub create_time: chrono::NaiveDateTime,
    pub update_time: chrono::NaiveDateTime,
    pub from: Option<AttrValue>,
    pub name: AttrValue,
    pub account: AttrValue,
    pub avatar: AttrValue,
    pub gender: AttrValue,
    pub age: i32,
    pub phone: Option<AttrValue>,
    pub email: Option<AttrValue>,
    pub address: Option<AttrValue>,
    pub birthday: Option<chrono::NaiveDateTime>,
}

// #[derive(PartialEq, Serialize, Deserialize, Default)]
// pub enum FriendStatus {
//     #[default]
//     Default,
//     Apply,
//     Agree,
//     Deny,
//     BlackList,
//     Delete,
// }

#[derive(Serialize, Debug, Default, Clone, Deserialize, PartialEq)]
pub struct FriendShipWithUser {
    pub friendship_id: AttrValue,
    pub user_id: AttrValue,
    pub name: AttrValue,
    pub avatar: AttrValue,
    pub gender: AttrValue,
    pub age: i32,
    pub status: FriendStatus,
    pub apply_msg: Option<AttrValue>,
    pub source: Option<AttrValue>,
    #[serde(default)]
    pub update_time: chrono::NaiveDateTime,
    #[serde(default)]
    pub read: ReadStatus,
    #[serde(default)]
    pub is_self: bool,
}

impl ItemInfo for Friend {
    fn name(&self) -> AttrValue {
        self.name.clone()
    }

    fn id(&self) -> AttrValue {
        self.friend_id.clone()
    }

    fn get_type(&self) -> RightContentType {
        RightContentType::Friend
    }

    fn avatar(&self) -> AttrValue {
        self.avatar.clone()
    }

    fn time(&self) -> i64 {
        self.create_time.timestamp_millis()
    }

    fn remark(&self) -> Option<AttrValue> {
        self.remark.clone()
    }

    fn signature(&self) -> Option<AttrValue> {
        // self.signature
        None
    }

    fn region(&self) -> Option<AttrValue> {
        self.address.clone()
    }

    fn owner(&self) -> AttrValue {
        self.friend_id.clone()
    }
}
