use serde::{Deserialize, Serialize};
use yew::AttrValue;

use crate::model::ContentType;
use crate::model::RightContentType;

use super::friend::Friend;
use super::group::Group;
use super::message::Hangup;
use super::message::InviteAnswerMsg;
use super::message::InviteCancelMsg;
use super::message::InviteMsg;
use super::message::InviteNotAnswerMsg;
use super::message::InviteType;
use super::message::Message;

pub fn attr_value_is_empty(value: &AttrValue) -> bool {
    value.is_empty()
}

/// 会话表
///
#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct Conversation {
    pub conv_type: RightContentType,
    pub friend_id: AttrValue,
    #[serde(default)]
    pub name: AttrValue,
    #[serde(default)]
    pub remark: Option<AttrValue>,
    #[serde(default)]
    pub avatar: AttrValue,
    pub last_msg: AttrValue,
    // 需要根据时间来排序
    pub last_msg_time: i64,
    pub last_msg_type: ContentType,
    pub unread_count: usize,
    pub mute: bool,
    #[serde(default)]
    pub is_pined: u8,
}

impl From<Message> for Conversation {
    fn from(msg: Message) -> Self {
        Self {
            last_msg: msg.content,
            last_msg_time: msg.send_time,
            last_msg_type: msg.content_type,
            conv_type: RightContentType::Default,
            friend_id: msg.friend_id,
            unread_count: 1,
            avatar: msg.avatar,
            name: msg.nickname,
            remark: None,
            mute: false,
            is_pined: 0,
        }
    }
}

impl From<Hangup> for Conversation {
    fn from(msg: Hangup) -> Self {
        let (last_msg, last_msg_type) = get_invite_type(&msg.invite_type);
        Self {
            friend_id: msg.friend_id,
            last_msg,
            last_msg_time: msg.create_time,
            last_msg_type,
            unread_count: 1,
            ..Default::default()
        }
    }
}

impl From<InviteNotAnswerMsg> for Conversation {
    fn from(msg: InviteNotAnswerMsg) -> Self {
        let (last_msg, last_msg_type) = get_invite_type(&msg.invite_type);
        Self {
            friend_id: msg.friend_id,
            last_msg,
            last_msg_time: msg.create_time,
            last_msg_type,
            unread_count: 1,
            ..Default::default()
        }
    }
}

impl From<InviteCancelMsg> for Conversation {
    fn from(msg: InviteCancelMsg) -> Self {
        let (last_msg, last_msg_type) = get_invite_type(&msg.invite_type);
        Self {
            friend_id: msg.friend_id,
            last_msg,
            last_msg_time: msg.create_time,
            last_msg_type,
            unread_count: 1,
            ..Default::default()
        }
    }
}

impl From<InviteMsg> for Conversation {
    fn from(msg: InviteMsg) -> Self {
        let (last_msg, last_msg_type) = get_invite_type(&msg.invite_type);
        Self {
            friend_id: msg.friend_id,
            last_msg,
            last_msg_time: msg.create_time,
            last_msg_type,
            avatar: msg.avatar,
            unread_count: 1,
            ..Default::default()
        }
    }
}

impl From<InviteAnswerMsg> for Conversation {
    fn from(msg: InviteAnswerMsg) -> Self {
        let (last_msg, last_msg_type) = get_invite_type(&msg.invite_type);
        Self {
            friend_id: msg.friend_id,
            last_msg,
            last_msg_time: msg.create_time,
            last_msg_type,
            unread_count: 1,
            avatar: msg.avatar,
            ..Default::default()
        }
    }
}

impl From<Group> for Conversation {
    fn from(value: Group) -> Self {
        Self {
            name: value.name,
            friend_id: value.id.to_string().into(),
            avatar: value.avatar,
            conv_type: RightContentType::Group,
            last_msg_time: value.create_time,
            ..Default::default()
        }
    }
}

impl From<Friend> for Conversation {
    fn from(value: Friend) -> Self {
        Self {
            conv_type: RightContentType::Friend,
            friend_id: value.friend_id,
            name: value.name,
            remark: value.remark,
            avatar: value.avatar,
            last_msg: AttrValue::from("new friend"),
            ..Default::default()
        }
    }
}
pub fn get_invite_type(t: &InviteType) -> (AttrValue, ContentType) {
    match t {
        InviteType::Video => (AttrValue::from("[视频通话]"), ContentType::VideoCall),
        InviteType::Audio => (AttrValue::from("[语音通话]"), ContentType::AudioCall),
    }
}
