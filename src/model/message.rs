use serde::{Deserialize, Serialize};
use yew::AttrValue;

use crate::model::friend::{FriendShipRequest, FriendShipWithUser};
use crate::model::ContentType;
use crate::{pb, utils};

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
    pub local_id: AttrValue,
    pub server_id: AttrValue,
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
    #[serde(default)]
    pub send_time: i64,
    pub is_success: bool,
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
            local_id: value.local_id,
            server_id: value.server_id,
            send_id: value.send_id,
            friend_id: value.friend_id,
            content_type,
            content: AttrValue::from("已经取消"),
            create_time: value.create_time,
            send_time: value.send_time,
            is_success: value.is_success,
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
            local_id: value.local_id,
            server_id: value.server_id,
            send_id: value.send_id,
            friend_id: value.friend_id,
            content_type,
            content,
            create_time: value.create_time,
            send_time: value.send_time,
            is_success: value.is_success,
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
            local_id: value.local_id,
            server_id: value.server_id,
            send_id: value.send_id,
            friend_id: value.friend_id,
            content_type,
            content,
            create_time: value.create_time,
            send_time: value.send_time,
            is_success: value.is_success,
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
            local_id: msg.local_id,
            server_id: msg.server_id,
            send_id: msg.send_id,
            friend_id: msg.friend_id,
            content_type,
            content: AttrValue::from("未接听"),
            create_time: msg.create_time,
            send_time: msg.send_time,
            is_success: msg.is_success,
            is_read: msg.is_self,
            is_self: msg.is_self,
            file_content: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GroupInvitation {
    pub info: Group,
    pub members: Vec<GroupMember>,
}

pub type MessageID = String;
pub type GroupID = String;
pub type UserID = String;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Msg {
    Single(Message),
    Group(GroupMsg),
    // GroupInvitation(GroupInvitation),
    SendRelationshipReq(FriendShipRequest),
    RecRelationship(FriendShipWithUser),
    RelationshipRes(Friend),
    ReadNotice(ReadNotice),
    SingleDeliveredNotice(MessageID),
    FriendshipDeliveredNotice(MessageID),
    OfflineSync(Message),
    SingleCall(SingleCall),
    ServerRecResp(ServerResponse), // GroupInvitationReceived((UserID, GroupID)),
}

/// server received message and return the result(success/failed)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ServerResponse {
    pub msg_id: AttrValue,
    pub success: bool,
    pub err_msg: Option<AttrValue>,
}

impl Default for Msg {
    fn default() -> Self {
        Self::Single(Message::default())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum GroupMsg {
    Message(Message),
    Invitation(GroupInvitation),
    MemberExit((UserID, GroupID)),
    Dismiss(GroupID),
    DismissOrExitReceived((UserID, GroupID)),
    InvitationReceived((UserID, GroupID)),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SingleCall {
    Offer(Offer),
    Invite(InviteMsg),
    InviteCancel(InviteCancelMsg),
    NotAnswer(InviteNotAnswerMsg),
    InviteAnswer(InviteAnswerMsg),
    Agree(Agree),
    HangUp(Hangup),
    NewIceCandidate(Candidate),
}

impl Default for SingleCall {
    fn default() -> Self {
        Self::Offer(Offer::default())
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
    pub local_id: AttrValue,
    pub server_id: AttrValue,
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    pub create_time: i64,
    pub send_time: i64,
    pub invite_type: InviteType,
    pub is_success: bool,
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
            local_id: self.local_id.clone(),
            server_id: self.server_id.clone(),
            send_id: self.send_id.clone(),
            friend_id: self.friend_id.clone(),
            content_type,
            content: AttrValue::from("未接听"),
            create_time: self.create_time,
            send_time: self.send_time,
            is_success: self.is_success,
            is_read: self.is_self,
            is_self: self.is_self,
            file_content: Default::default(),
        }
    }
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct InviteCancelMsg {
    pub local_id: AttrValue,
    pub server_id: AttrValue,
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    pub create_time: i64,
    pub send_time: i64,
    pub invite_type: InviteType,
    pub is_success: bool,
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
            local_id: self.local_id.clone(),
            server_id: self.server_id.clone(),
            send_id: self.send_id.clone(),
            friend_id: self.friend_id.clone(),
            content_type,
            content: AttrValue::from("已经取消"),
            create_time: self.create_time,
            send_time: self.send_time,
            is_read: self.is_self,
            is_self: self.is_self,
            is_success: self.is_success,
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
    pub local_id: AttrValue,
    pub server_id: AttrValue,
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    pub create_time: i64,
    pub send_time: i64,
    pub agree: bool,
    pub invite_type: InviteType,
    pub is_success: bool,
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
            local_id: self.local_id.clone(),
            server_id: self.server_id.clone(),
            send_id: self.send_id.clone(),
            friend_id: self.friend_id.clone(),
            content_type,
            content,
            create_time: self.create_time,
            send_time: self.send_time,
            is_success: self.is_success,
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
    pub sdp: Option<String>,
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    pub create_time: i64,
}

#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Hangup {
    pub local_id: AttrValue,
    pub server_id: AttrValue,
    pub send_id: AttrValue,
    pub friend_id: AttrValue,
    pub create_time: i64,
    pub send_time: i64,
    pub invite_type: InviteType,
    pub sustain: i64,
    pub is_success: bool,
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
            local_id: self.local_id.clone(),
            server_id: self.server_id.clone(),
            send_id: self.send_id.clone(),
            friend_id: self.friend_id.clone(),
            content_type,
            content,
            create_time: self.create_time,
            send_time: self.send_time,
            is_success: self.is_success,
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

impl TryFrom<pb::message::Msg> for Message {
    type Error = String;

    fn try_from(value: pb::message::Msg) -> Result<Self, Self::Error> {
        let status: ContentType = value.content_type.into();
        Ok(Self {
            id: 0,
            local_id: value.local_id.into(),
            server_id: value.server_id.into(),
            send_id: value.send_id.into(),
            friend_id: value.receiver_id.into(),
            content_type: ContentType::from(value.content_type),
            content: String::from_utf8(value.content)
                .map_err(|e| e.to_string())?
                .into(),
            create_time: value.create_time,
            send_time: value.send_time,
            is_success: status == ContentType::Error,
            is_read: false,
            is_self: false,
            file_content: AttrValue::default(),
        })
    }
}
