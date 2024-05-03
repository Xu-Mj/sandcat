use serde::{Deserialize, Serialize};
use yew::AttrValue;

use crate::model::friend::Friend;

use super::group::GroupMember;

/// 用来接收服务端的用户信息
#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Clone)]
// 当前用户表
pub struct UserInfo {
    pub login: bool,
    pub id: AttrValue,
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

/// 用户模型，用来记录当前登录的用户信息
#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Clone)]
pub struct User {
    pub id: AttrValue,
    pub name: AttrValue,
    pub account: AttrValue,
    pub avatar: AttrValue,
    pub gender: AttrValue,
    pub age: i32,
    pub phone: Option<AttrValue>,
    pub email: Option<AttrValue>,
    pub region: Option<AttrValue>,
    pub address: Option<AttrValue>,
    pub birthday: Option<chrono::NaiveDateTime>,
    pub signature: AttrValue,
}
#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Clone)]
pub struct UserWithMatchType {
    pub id: AttrValue,
    pub name: AttrValue,
    pub account: AttrValue,
    pub avatar: AttrValue,
    pub gender: AttrValue,
    pub age: i32,
    pub email: Option<AttrValue>,
    pub region: Option<AttrValue>,
    pub birthday: Option<i64>,
    pub match_type: Option<AttrValue>,
    pub signature: AttrValue,
    pub is_friend: bool,
}

#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Clone)]
pub struct UserUpdate {
    pub id: String,
    pub name: String,
    pub avatar: String,
    pub gender: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub signature: Option<String>,
}

impl From<Friend> for UserWithMatchType {
    fn from(value: Friend) -> Self {
        Self {
            id: value.friend_id,
            name: value.name,
            account: value.account,
            avatar: value.avatar,
            gender: value.gender,
            age: value.age,
            email: None,
            region: value.region,
            birthday: None,
            match_type: Some(value.source),
            signature: value.signature,
            is_friend: true,
        }
    }
}

impl From<User> for UserWithMatchType {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            name: value.name,
            account: value.account,
            avatar: value.avatar,
            gender: value.gender,
            age: value.age,
            email: value.email,
            region: value.address,
            birthday: value.birthday.map(|x| x.timestamp_millis()),
            match_type: None,
            signature: value.signature,
            is_friend: false,
        }
    }
}

impl From<GroupMember> for UserWithMatchType {
    fn from(value: GroupMember) -> Self {
        Self {
            id: value.user_id.clone(),
            name: value.group_name,
            account: value.user_id,
            avatar: value.avatar,
            gender: value.gender,
            age: value.age,
            region: value.region,
            birthday: None,
            match_type: None,
            email: None,
            signature: value.signature,
            is_friend: value.is_friend,
        }
    }
}

impl From<GroupMember> for User {
    fn from(value: GroupMember) -> Self {
        Self {
            id: value.user_id.clone(),
            name: value.group_name,
            account: value.user_id,
            age: 0,
            gender: value.gender,
            avatar: value.avatar,
            phone: None,
            address: None,
            email: None,
            birthday: None,
            signature: AttrValue::default(),
            region: value.region,
        }
    }
}

impl From<Friend> for User {
    fn from(value: Friend) -> Self {
        Self {
            id: value.friend_id,
            name: value.name,
            account: value.account,
            avatar: value.avatar,
            gender: value.gender,
            age: value.age,
            phone: None,
            email: None,
            address: None,
            birthday: None,
            signature: value.signature,
            region: value.region,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserRegister {
    pub avatar: AttrValue,
    pub name: AttrValue,
    pub password: AttrValue,
    pub email: AttrValue,
    pub code: AttrValue,
}
