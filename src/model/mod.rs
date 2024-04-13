pub mod conversation;
pub mod friend;
pub mod group;
pub mod message;
pub mod notification;
pub mod seq;
pub mod user;

use std::fmt::{Debug, Display};

use serde::{Deserialize, Serialize};
use yew::AttrValue;

use self::friend::FriendStatus;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum ContentType {
    #[default]
    Default = 0,
    Text = 1,
    Image = 2,
    Video = 3,
    Audio = 4,
    File = 5,
    Emoji = 6,
    VideoCall = 7,
    AudioCall = 8,
    Error = 9,
}

impl From<i32> for ContentType {
    fn from(value: i32) -> Self {
        match value {
            1 => ContentType::Text,
            2 => ContentType::Image,
            3 => ContentType::Video,
            4 => ContentType::Audio,
            5 => ContentType::File,
            6 => ContentType::Emoji,
            7 => ContentType::VideoCall,
            8 => ContentType::AudioCall,
            9 => ContentType::Error,
            _ => ContentType::Default,
        }
    }
}
pub trait ItemInfo: Debug {
    fn name(&self) -> AttrValue;

    fn id(&self) -> AttrValue;

    fn get_type(&self) -> RightContentType;

    fn avatar(&self) -> AttrValue;

    fn time(&self) -> i64;

    fn remark(&self) -> Option<AttrValue>;

    fn signature(&self) -> AttrValue;

    fn region(&self) -> Option<AttrValue>;

    fn owner(&self) -> AttrValue;

    fn status(&self) -> FriendStatus;
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

#[derive(Default, Clone, PartialEq, Debug)]
pub enum FriendShipStateType {
    #[default]
    Req,
    Res,
    RecResp,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentType {
    Contacts,
    #[default]
    Messages,
    Setting,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CurrentItem {
    pub item_id: AttrValue,
    pub content_type: RightContentType,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UnreadItem {
    pub unread_msg: usize,
    pub unread_contact: usize,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CommonProps {
    pub id: AttrValue,
    pub name: AttrValue,
    pub avatar: AttrValue,
    pub time: i64,
    pub remark: AttrValue,
}
