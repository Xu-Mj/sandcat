pub mod config;
pub mod conv;
pub mod current_item;
pub mod friend;
pub mod friend_ship;
pub mod message;
pub mod repository;
pub mod user;

use std::{
    error::Error,
    fmt::{Debug, Display},
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};
use yew::AttrValue;

use crate::model::{
    group::Group,
    message::{
        Hangup, InviteAnswerMsg, InviteCancelMsg, InviteMsg, InviteNotAnswerMsg, InviteType,
    },
    ContentType, RightContentType,
};
// 不同用户创建不同的数据库，方便查询，提升性能
// 用户登录时检查对应的数据库是否存在，不存在则创建
pub static WS_ADDR: &str = "WS_ADDR";
pub static TOKEN: &str = "ACCESS_TOKEN";
pub static DB_NAME: OnceLock<String> = OnceLock::new();
// 定义表名常量
pub const FRIEND_TABLE_NAME: &str = "friends";
pub const FRIENDSHIP_TABLE_NAME: &str = "friendships";
pub const CONVERSATION_TABLE_NAME: &str = "conversations";
pub const MESSAGE_TABLE_NAME: &str = "messages";
pub const USER_TABLE_NAME: &str = "users";
pub const CONFIG_TABLE_NAME: &str = "configs";
pub const CURRENT_CONV_TABLE_NAME: &str = "conv";

pub const FRIENDSHIP_UNREAD_INDEX: &str = "read";
pub const FRIENDSHIP_ID_INDEX: &str = "friendship_id";
// 定义索引常量
pub const FRIEND_USER_ID_INDEX: &str = "friend_id";
pub const FRIEND_FRIEND_ID_INDEX: &str = "friend_id";
pub const FRIEND_NAME_INDEX: &str = "name";
pub const FRIEND_GENDER_INDEX: &str = "gender";
pub const FRIEND_REMARK_INDEX: &str = "remark";
pub const FRIEND_PHONE_INDEX: &str = "phone";
pub const FRIEND_ADDRESS_INDEX: &str = "address";
pub const FRIEND_TIME_INDEX: &str = "time";

pub const CONVERSATION_FRIEND_ID_INDEX: &str = "friend_id";
pub const CONVERSATION_LAST_MSG_TIME_INDEX: &str = "last_msg_time";

// pub const MESSAGE_USER_ID_INDEX: &str = "user_id";
pub const MESSAGE_FRIEND_ID_INDEX: &str = "friend_id";
pub const MESSAGE_ID_INDEX: &str = "msg_id";
// pub const MESSAGE_SEND_ID_INDEX: &str = "send_id";
pub const MESSAGE_TIME_INDEX: &str = "time";
pub const MESSAGE_CONTENT_INDEX: &str = "content";
pub const MESSAGE_TYPE_INDEX: &str = "type";
pub const MESSAGE_IS_READ_INDEX: &str = "is_read";

fn is_zero(id: &i32) -> bool {
    *id == 0
}

// 数据结构
// 表

pub fn attr_value_string_empty(value: &AttrValue) -> bool {
    value.to_string().is_empty()
}

/// 会话表
///
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct Conversation {
    #[serde(skip_serializing_if = "is_zero")]
    pub id: i32,
    pub conv_type: RightContentType,
    // pub user_id: i32,
    pub friend_id: AttrValue,
    #[serde(skip_serializing_if = "attr_value_string_empty")]
    pub name: AttrValue,
    #[serde(skip_serializing_if = "attr_value_string_empty")]
    pub avatar: AttrValue,
    pub last_msg: AttrValue,
    // 需要根据时间来排序
    pub last_msg_time: i64,
    pub last_msg_type: ContentType,
    pub unread_count: usize,
    // pub file:
}

impl From<Hangup> for Conversation {
    fn from(msg: Hangup) -> Self {
        let (last_msg, last_msg_type) = get_invite_type(msg.invite_type);
        Self {
            friend_id: msg.friend_id,
            last_msg,
            last_msg_time: msg.create_time,
            last_msg_type,
            ..Default::default()
        }
    }
}

impl From<InviteNotAnswerMsg> for Conversation {
    fn from(msg: InviteNotAnswerMsg) -> Self {
        let (last_msg, last_msg_type) = get_invite_type(msg.invite_type);
        Self {
            friend_id: msg.friend_id,
            last_msg,
            last_msg_time: msg.create_time,
            last_msg_type,
            ..Default::default()
        }
    }
}

impl From<InviteCancelMsg> for Conversation {
    fn from(msg: InviteCancelMsg) -> Self {
        let (last_msg, last_msg_type) = get_invite_type(msg.invite_type);
        Self {
            friend_id: msg.friend_id,
            last_msg,
            last_msg_time: msg.create_time,
            last_msg_type,
            ..Default::default()
        }
    }
}

impl From<InviteMsg> for Conversation {
    fn from(msg: InviteMsg) -> Self {
        let (last_msg, last_msg_type) = get_invite_type(msg.invite_type);
        Self {
            friend_id: msg.friend_id,
            last_msg,
            last_msg_time: msg.create_time,
            last_msg_type,
            ..Default::default()
        }
    }
}

impl From<InviteAnswerMsg> for Conversation {
    fn from(msg: InviteAnswerMsg) -> Self {
        let (last_msg, last_msg_type) = get_invite_type(msg.invite_type);
        Self {
            friend_id: msg.friend_id,
            last_msg,
            last_msg_time: msg.create_time,
            last_msg_type,
            ..Default::default()
        }
    }
}

impl From<Group> for Conversation {
    fn from(value: Group) -> Self {
        Self {
            name: value.name,
            friend_id: value.id,
            avatar: value.avatar,
            ..Default::default()
        }
    }
}

fn get_invite_type(t: InviteType) -> (AttrValue, ContentType) {
    match t {
        InviteType::Video => (AttrValue::from("[视频通话]"), ContentType::VideoCall),
        InviteType::Audio => (AttrValue::from("[语音通话]"), ContentType::AudioCall),
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub enum MessageType {
    #[default]
    Default,
    Single,
    Group,
    DeliveredNotice,
    ReadNotice,
}

impl From<RightContentType> for MessageType {
    fn from(conv_type: RightContentType) -> Self {
        match conv_type {
            RightContentType::Friend => MessageType::Single,
            RightContentType::Group => MessageType::Group,
            _ => MessageType::Default,
        }
    }
}

/// 消息表，要不要每个用户对应一个表？
/// 表名由message+user_id组成
///
/// 由于indexeddb只能在onupgrade中建表，不能动态创建，所以消息只能存到一张表中
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct Message {
    #[serde(skip_serializing_if = "is_zero")]
    #[serde(default)]
    pub id: i32,
    pub msg_id: AttrValue,
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    // 是MessageType类型，需要做转换
    pub msg_type: MessageType,
    #[serde(default)]
    pub content_type: ContentType,
    // 如果是文件类型，那么content存储文件的路径
    pub content: AttrValue,
    #[serde(default)]
    pub create_time: i64,
    // pub update_time: String,
    #[serde(default)]
    pub is_read: bool,
    #[serde(default)]
    pub is_self: bool,
    // 是否删除字段可以只存储在服务端
    // pub is_delete: bool,
    #[serde(skip)]
    pub file_content: AttrValue,
}

// 配置文件表
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Default)]
pub struct Config {
    #[serde(skip_serializing_if = "is_zero")]
    // 如果为零那么不进行序列化
    pub id: i32,
    // pub user_id: i32,
    pub name: AttrValue,
    pub value: AttrValue,
    // pub create_time: String,
    // pub update_time: String,
}

// #[derive(Debug, Deserialize, Serialize, Default, PartialEq, Clone)]
// // 当前用户表
// pub struct User {
//     #[serde(skip)]
//     pub login: bool,
//     pub id: AttrValue,
//     pub name: AttrValue,
//     pub avatar: AttrValue,
//     pub gender: AttrValue,
//     pub phone: Option<AttrValue>,
//     pub email: Option<AttrValue>,
//     pub address: Option<AttrValue>,
//     pub birthday: Option<chrono::NaiveDateTime>,
// }

// 定义数据库查询状态
#[derive(Debug, Clone)]
pub enum QueryStatus<T> {
    // 正在查询
    Querying,
    // 查询成功
    QuerySuccess(T),
    // 查询失败
    QueryFail(QueryError),
}

#[derive(Debug, Clone)]
pub struct QueryError {
    // pub code: i32,
    pub err: String,
}

// 为error类型实现Display特征
impl Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.err, f)
    }
}

impl Error for QueryError {}
