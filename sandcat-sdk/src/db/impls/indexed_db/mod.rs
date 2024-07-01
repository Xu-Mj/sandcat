pub mod conv;
pub mod friend;
pub mod friend_ship;
pub mod group;
pub mod group_members;
pub mod group_msg;
pub mod message;
pub mod repository;
pub mod seq;
pub mod user;
pub mod voice;

use std::{cell::RefCell, fmt::Debug, rc::Rc, sync::OnceLock};

use wasm_bindgen::closure::Closure;
use yew::Event;

use crate::error::Error;

pub type SuccessCallback = Rc<RefCell<Option<Closure<dyn FnMut(&Event)>>>>;

// 不同用户创建不同的数据库，方便查询，提升性能
// 用户登录时检查对应的数据库是否存在，不存在则创建

pub static DB_NAME: OnceLock<String> = OnceLock::new();
// 定义表名常量
pub const FRIEND_TABLE_NAME: &str = "friends";
pub const FRIENDSHIP_TABLE_NAME: &str = "friendships";
pub const CONVERSATION_TABLE_NAME: &str = "conversations";
pub const MESSAGE_TABLE_NAME: &str = "messages";
pub const USER_TABLE_NAME: &str = "users";
pub const GROUP_TABLE_NAME: &str = "groups";
pub const GROUP_MSG_TABLE_NAME: &str = "group_messages";
pub const GROUP_MEMBERS_TABLE_NAME: &str = "group_members";
pub const SEQ_TABLE_NAME: &str = "seq";
pub const VOICE_TABLE_NAME: &str = "voices";

pub const FRIENDSHIP_UNREAD_INDEX: &str = "read";
pub const FRIENDSHIP_ID_INDEX: &str = "fs_id";
pub const GROUP_ID_INDEX: &str = "group_id";
// 定义索引常量
pub const GROUP_ID_AND_USER_ID: &str = "group_id_and_friend_id";
pub const FRIEND_USER_ID_INDEX: &str = "friend_id";
// pub const FRIEND_FRIEND_ID_INDEX: &str = "friend_id";
pub const FRIEND_NAME_INDEX: &str = "name";
pub const FRIEND_GENDER_INDEX: &str = "gender";
pub const FRIEND_REMARK_INDEX: &str = "remark";
pub const FRIEND_PHONE_INDEX: &str = "phone";
pub const FRIEND_ADDRESS_INDEX: &str = "address";
pub const FRIEND_TIME_INDEX: &str = "time";

// pub const CONVERSATION_FRIEND_ID_INDEX: &str = "friend_id";
// pub const CONVERSATION_LAST_MSG_TIME_INDEX: &str = "last_msg_time";
pub const CONVERSATION_IS_PINED_WITH_TIME_INDEX: &str = "is_pined_with_time";

// pub const MESSAGE_USER_ID_INDEX: &str = "user_id";
pub const MESSAGE_FRIEND_AND_SEND_TIME_INDEX: &str = "friend_id_and_send_time";
pub const MESSAGE_FRIEND_ID_INDEX: &str = "friend_id";
pub const MESSAGE_ID_INDEX: &str = "local_id";
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
    QueryFail(Error),
}
