use serde::{Deserialize, Serialize};
use yew::AttrValue;

use super::{ItemInfo, RightContentType};
#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum FriendStatus {
    Default = 0,
    #[default]
    Pending = 1,
    Accepted = 2,
    Rejected = 3,
    Blacked = 4,
    Cancelled = 5,
    Delete = 6,
    Failed = 7,
}

#[derive(Debug, Default, Serialize, Clone, Deserialize, PartialEq)]
pub struct FriendShipRequest {
    // #[serde(skip_serializing_if = "is_zero")]
    // #[serde(default)]
    // pub id: i32,
    pub user_id: AttrValue,
    pub friend_id: AttrValue,
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
    pub fs_id: AttrValue,
    pub user_id: AttrValue,
    pub name: AttrValue,
    pub account: AttrValue,
    pub avatar: AttrValue,
    pub gender: AttrValue,
    pub age: i32,
    pub status: i32,
    pub apply_msg: Option<AttrValue>,
    pub source: AttrValue,
    pub region: Option<AttrValue>,
    pub create_time: i64,
    #[serde(default)]
    pub read: ReadStatus,
    #[serde(default)]
    pub is_self: bool,
}

/// we must guarantee the order of the fields and the count of the fields
/// the bincode has not support the default value for the lack fields of structure
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, Debug)]
pub struct FriendshipWithUser4Response {
    pub fs_id: AttrValue,
    pub user_id: AttrValue,
    pub name: AttrValue,
    pub avatar: AttrValue,
    pub gender: AttrValue,
    pub age: i32,
    pub region: Option<AttrValue>,
    pub status: i32,
    pub apply_msg: Option<AttrValue>,
    pub source: AttrValue,
    pub create_time: i64,
    pub account: AttrValue,
}

impl From<FriendshipWithUser4Response> for FriendShipWithUser {
    fn from(value: FriendshipWithUser4Response) -> Self {
        Self {
            fs_id: value.fs_id,
            user_id: value.user_id,
            name: value.name,
            account: value.account,
            avatar: value.avatar,
            age: value.age,
            read: ReadStatus::False,
            region: value.region,
            status: value.status,
            apply_msg: value.apply_msg,
            source: value.source,
            create_time: value.create_time,
            is_self: false,
            gender: value.gender,
        }
    }
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

    fn status(&self) -> FriendStatus {
        self.status.clone()
    }
}
