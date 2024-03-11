pub mod conversation;
pub mod friend;
pub mod group;
pub mod message;
pub mod notification;
pub mod user;

use std::fmt::Display;

use serde::{Deserialize, Serialize};
use yew::AttrValue;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    #[default]
    Default,
    Text,
    Image,
    Video,
    Audio,
    File,
    Emoji,
    VideoCall,
    AudioCall,
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

    fn time(&self) -> i64;

    fn remark(&self) -> Option<AttrValue>;
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub enum RightContentType {
    // 啥都没选择时，根据全局组件类型展示一些标语
    #[default]
    Default,
    // 好友，包括会话与群组信息
    Friend,
    // 群组，包括会话与群组信息
    Group,
    // 用户信息，todo考虑查找好友时使用浮窗的方式
    UserInfo,
    // 好友请求列表
    FriendShipList,
    // 其他服务消息
    Service,
}

impl From<MessageType> for RightContentType {
    fn from(msg_type: MessageType) -> Self {
        match msg_type {
            MessageType::Single => Self::Friend,
            MessageType::Group => Self::Group,
            _ => Self::Default,
        }
    }
}

impl From<usize> for RightContentType {
    fn from(id: usize) -> Self {
        match id {
            1 => RightContentType::Friend,
            2 => RightContentType::Group,
            _ => RightContentType::Default,
        }
    }
}

impl Display for RightContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RightContentType::Friend => write!(f, "friend"),
            RightContentType::Group => write!(f, "group"),
            RightContentType::Default => write!(f, "default"),
            RightContentType::UserInfo => write!(f, "user_info"),
            RightContentType::FriendShipList => write!(f, "frienship_list"),
            RightContentType::Service => write!(f, "service"),
        }
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
