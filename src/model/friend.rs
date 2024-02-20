use serde::{Deserialize, Serialize};
use yew::AttrValue;

#[derive(Debug, Default, Serialize, Clone, Deserialize, PartialEq)]
pub struct FriendShipRequest {
    // #[serde(skip_serializing_if = "is_zero")]
    // #[serde(default)]
    // pub id: i32,
    pub user_id: AttrValue,
    pub friend_id: AttrValue,
    pub status: AttrValue,
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
    pub status: AttrValue,
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

#[derive(PartialEq, Serialize, Deserialize, Default)]
pub enum FriendStatus {
    #[default]
    Default,
    Apply,
    Agree,
    Deny,
    BlackList,
    Delete,
}

fn is_zero(id: &i32) -> bool {
    *id == 0
}

#[derive(Serialize, Debug, Default, Clone, Deserialize, PartialEq)]
pub struct FriendShipWithUser {
    #[serde(skip_serializing_if = "is_zero")]
    #[serde(default)]
    pub id: i32,
    pub friendship_id: AttrValue,
    pub user_id: AttrValue,
    pub name: AttrValue,
    pub avatar: AttrValue,
    pub gender: AttrValue,
    pub age: i32,
    pub status: AttrValue,
    pub apply_msg: Option<AttrValue>,
    pub source: Option<AttrValue>,
    #[serde(default)]
    pub update_time: chrono::NaiveDateTime,
    #[serde(default)]
    pub read: ReadStatus,
    #[serde(default)]
    pub is_self: bool,
}

pub struct Group {
    pub id: AttrValue,
    pub name: AttrValue,
    pub avatar: AttrValue,
}

pub enum ItemType {
    Friend,
    Group,
}

pub trait ItemInfo {
    fn name(&self) -> AttrValue;

    fn id(&self) -> AttrValue;

    fn get_type(&self) -> ItemType;

    fn avatar(&self) -> AttrValue;
}

impl ItemInfo for Friend {
    fn name(&self) -> AttrValue {
        self.name.clone()
    }

    fn id(&self) -> AttrValue {
        self.id.clone()
    }

    fn get_type(&self) -> ItemType {
        ItemType::Friend
    }

    fn avatar(&self) -> AttrValue {
        self.avatar.clone()
    }
}

impl ItemInfo for Group {
    fn name(&self) -> AttrValue {
        self.name.clone()
    }

    fn id(&self) -> AttrValue {
        self.id.clone()
    }

    fn get_type(&self) -> ItemType {
        ItemType::Group
    }

    fn avatar(&self) -> AttrValue {
        self.avatar.clone()
    }
}
