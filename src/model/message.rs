use serde::{Deserialize, Serialize};
use yew::AttrValue;

use crate::model::friend::{FriendShipRequest, FriendShipWithUser};
use crate::model::ContentType;
use crate::utils;

use super::friend::Friend;
use super::group::{Group, GroupMember};

pub(crate) const DEFAULT_HELLO_MESSAGE: &str = "I've accepted your friend request. Now let's chat!";

fn is_zero(id: &i32) -> bool {
    *id == 0
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
    // pub msg_type: MessageType,
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

impl From<InviteCancelMsg> for Message {
    fn from(value: InviteCancelMsg) -> Self {
        let content_type = match value.invite_type {
            InviteType::Video => ContentType::VideoCall,
            InviteType::Audio => ContentType::AudioCall,
        };
        Message {
            id: 0,
            msg_id: value.msg_id,
            send_id: value.send_id,
            friend_id: value.friend_id,
            content_type,
            content: AttrValue::from("已经取消"),
            create_time: value.create_time,
            is_read: false,
            is_self: value.is_self,
            file_content: Default::default(),
        }
    }
}

impl From<InviteAnswerMsg> for Message {
    fn from(value: InviteAnswerMsg) -> Self {
        let content_type = match value.invite_type {
            InviteType::Video => ContentType::VideoCall,
            InviteType::Audio => ContentType::AudioCall,
        };
        let content = if value.agree {
            AttrValue::from("一接通")
        } else {
            AttrValue::from("已经拒绝")
        };
        Message {
            id: 0,
            msg_id: value.msg_id,
            send_id: value.send_id,
            friend_id: value.friend_id,
            content_type,
            content,
            create_time: value.create_time,
            is_read: false,
            is_self: value.is_self,
            file_content: Default::default(),
        }
    }
}

impl Message {
    pub fn from_hangup(value: Hangup) -> Self {
        let content_type = match value.invite_type {
            InviteType::Video => ContentType::VideoCall,
            InviteType::Audio => ContentType::AudioCall,
        };
        // 计算时间
        let content = AttrValue::from(utils::format_milliseconds(value.sustain));
        Message {
            id: 0,
            msg_id: value.msg_id,
            send_id: value.send_id,
            friend_id: value.friend_id,
            content_type,
            content,
            create_time: value.create_time,
            is_read: false,
            is_self: value.is_self,
            file_content: Default::default(),
        }
    }
    pub fn from_not_answer(msg: InviteNotAnswerMsg) -> Self {
        let content_type = match msg.invite_type {
            InviteType::Video => ContentType::VideoCall,
            InviteType::Audio => ContentType::AudioCall,
        };

        Self {
            id: 0,
            msg_id: msg.msg_id,
            send_id: msg.send_id,
            friend_id: msg.friend_id,
            content_type,
            content: AttrValue::from("未接听"),
            create_time: msg.create_time,
            is_read: msg.is_self,
            is_self: msg.is_self,
            file_content: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CreateGroup {
    pub info: Group,
    pub members: Vec<GroupMember>,
}

pub type MessageID = String;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Msg {
    Single(Message),
    Group(Message),
    CreateGroup(CreateGroup),
    SendRelationshipReq(FriendShipRequest),
    RecRelationship(FriendShipWithUser),
    RelationshipRes(Friend),
    ReadNotice(ReadNotice),
    SingleDeliveredNotice(MessageID),
    FriendshipDeliveredNotice(MessageID),
    OfflineSync(Message),
    SingleCallOffer(Offer),
    SingleCallInvite(InviteMsg),
    SingleCallInviteCancel(InviteCancelMsg),
    SingleCallNotAnswer(InviteNotAnswerMsg),
    SingleCallInviteAnswer(InviteAnswerMsg),
    SingleCallAgree(Agree),
    SingleCallHangUp(Hangup),
    NewIceCandidate(Candidate),
}

impl Default for Msg {
    fn default() -> Self {
        Self::Single(Message::default())
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Offer {
    pub sdp: AttrValue,
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    pub create_time: i64,
}

#[derive(Debug, Default, Serialize)]
pub struct InviteInfo {
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    pub invite_type: InviteType,
    pub start_time: i64,
    pub end_time: i64,
    pub connected: bool,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct InviteMsg {
    pub msg_id: AttrValue,
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    pub create_time: i64,
    pub invite_type: InviteType,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct InviteNotAnswerMsg {
    pub msg_id: AttrValue,
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    pub create_time: i64,
    pub invite_type: InviteType,
    #[serde(default)]
    pub is_self: bool,
}

impl InviteNotAnswerMsg {
    pub fn clone_as_message(&self) -> Message {
        let content_type = match self.invite_type {
            InviteType::Video => ContentType::VideoCall,
            InviteType::Audio => ContentType::AudioCall,
        };

        Message {
            id: 0,
            msg_id: self.msg_id.clone(),
            send_id: self.send_id.clone(),
            friend_id: self.friend_id.clone(),
            content_type,
            content: AttrValue::from("未接听"),
            create_time: self.create_time,
            is_read: self.is_self,
            is_self: self.is_self,
            file_content: Default::default(),
        }
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct InviteCancelMsg {
    pub msg_id: AttrValue,
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    pub create_time: i64,
    pub invite_type: InviteType,
    #[serde(default)]
    pub is_self: bool,
}

impl InviteCancelMsg {
    pub fn clone_as_message(&self) -> Message {
        let content_type = match self.invite_type {
            InviteType::Video => ContentType::VideoCall,
            InviteType::Audio => ContentType::AudioCall,
        };

        Message {
            id: 0,
            msg_id: self.msg_id.clone(),
            send_id: self.send_id.clone(),
            friend_id: self.friend_id.clone(),
            content_type,
            content: AttrValue::from("已经取消"),
            create_time: self.create_time,
            is_read: self.is_self,
            is_self: self.is_self,
            file_content: Default::default(),
        }
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub enum InviteType {
    Video,
    #[default]
    Audio,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct InviteAnswerMsg {
    pub msg_id: AttrValue,
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    pub create_time: i64,
    pub agree: bool,
    pub invite_type: InviteType,
    // 主要区分发起端，因为接收端永远都是false不需要处理
    #[serde(default)]
    pub is_self: bool,
}

impl InviteAnswerMsg {
    pub fn clone_as_message(&self) -> Message {
        let content_type = match self.invite_type {
            InviteType::Video => ContentType::VideoCall,
            InviteType::Audio => ContentType::AudioCall,
        };
        let content = if self.agree {
            AttrValue::from("一接通")
        } else {
            AttrValue::from("已经拒绝")
        };
        Message {
            id: 0,
            msg_id: self.msg_id.clone(),
            send_id: self.send_id.clone(),
            friend_id: self.friend_id.clone(),
            content_type,
            content,
            create_time: self.create_time,
            is_read: self.is_self,
            is_self: self.is_self,
            file_content: Default::default(),
        }
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Candidate {
    pub candidate: AttrValue,
    pub sdp_mid: Option<String>,
    pub sdp_m_index: Option<u16>,
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    pub create_time: i64,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Agree {
    pub sdp: Option<AttrValue>,
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    pub create_time: i64,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Hangup {
    pub msg_id: AttrValue,
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    pub create_time: i64,
    pub invite_type: InviteType,
    pub sustain: i64,
    #[serde(default)]
    pub is_self: bool,
}

impl Hangup {
    pub fn clone_as_message(&self) -> Message {
        let content_type = match self.invite_type {
            InviteType::Video => ContentType::VideoCall,
            InviteType::Audio => ContentType::AudioCall,
        };
        let content = AttrValue::from(utils::format_milliseconds(self.sustain));

        Message {
            id: 0,
            msg_id: self.msg_id.clone(),
            send_id: self.send_id.clone(),
            friend_id: self.friend_id.clone(),
            content_type,
            content,
            create_time: self.create_time,
            is_read: self.is_self,
            is_self: self.is_self,
            file_content: Default::default(),
        }
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Relation {
    pub send_id: String,
    pub friend_id: String,
    pub status: RelationStatus,
    pub create_time: i64,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub enum RelationStatus {
    #[default]
    Apply,
    Agree,
    Deny,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct ReadNotice {
    pub msg_ids: Vec<String>,
    pub send_id: String,
    pub friend_id: String,
    pub create_time: i64,
}
