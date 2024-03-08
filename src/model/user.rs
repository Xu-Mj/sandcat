use serde::{Deserialize, Serialize};
use yew::AttrValue;

use crate::model::friend::Friend;

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
    pub address: Option<AttrValue>,
    pub birthday: Option<chrono::NaiveDateTime>,
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
            phone: value.phone,
            email: value.email,
            address: value.address,
            birthday: value.birthday,
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
