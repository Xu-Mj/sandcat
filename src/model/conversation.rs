use serde::{Deserialize, Serialize};
use yew::AttrValue;

use crate::model::ContentType;
use crate::model::RightContentType;

use super::group::Group;
use super::message::Hangup;
use super::message::InviteAnswerMsg;
use super::message::InviteCancelMsg;
use super::message::InviteMsg;
use super::message::InviteNotAnswerMsg;
use super::message::InviteType;

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
            friend_id: value.id.to_string().into(),
            avatar: value.avatar,
            conv_type: RightContentType::Group,
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
