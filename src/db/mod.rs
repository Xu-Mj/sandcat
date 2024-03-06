pub mod config;
pub mod conv;
pub mod current_item;
pub mod friend;
pub mod friend_ship;
pub mod group;
pub mod group_members;
pub mod message;
pub mod repository;
pub mod user;

use std::{
    error::Error,
    fmt::{Debug, Display},
    sync::OnceLock,
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
pub const GROUP_TABLE_NAME: &str = "groups";
pub const GROUP_MEMBERS_TABLE_NAME: &str = "group_members";

pub const FRIENDSHIP_UNREAD_INDEX: &str = "read";
pub const FRIENDSHIP_ID_INDEX: &str = "friendship_id";
pub const GROUP_ID_INDEX: &str = "group_id";
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
