use serde::{Deserialize, Serialize};
use yew::AttrValue;

use crate::model::friend::{FriendShipRequest, FriendShipWithUser, FriendshipWithUser4Response};
use crate::model::ContentType;
use crate::pb::message::{Msg as PbMsg, MsgType};
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

pub fn convert_server_msg(msg: PbMsg) -> Result<Msg, String> {
    // debug!("convert msg: {:?}", msg);
    let msg_type = MsgType::try_from(msg.msg_type).unwrap();
    match msg_type {
        MsgType::SingleMsg => Ok(Msg::Single(Message::try_from(msg)?)),
        MsgType::GroupMsg => Ok(Msg::Group(GroupMsg::Message(Message::try_from(msg)?))),
        MsgType::GroupInvitation => {
            // decode content
            let info = bincode::deserialize(&msg.content).map_err(|e| e.to_string())?;
            Ok(Msg::Group(GroupMsg::Invitation(info)))
        }
        MsgType::GroupInviteNew => todo!(),
        MsgType::GroupMemberExit => Ok(Msg::Group(GroupMsg::MemberExit((
            msg.send_id,
            msg.group_id,
        )))),
        MsgType::GroupDismiss => Ok(Msg::Group(GroupMsg::Dismiss(msg.group_id))),
        MsgType::GroupDismissOrExitReceived => todo!(),
        MsgType::GroupInvitationReceived => todo!(),
        MsgType::GroupUpdate => todo!(),
        MsgType::FriendApplyReq => {
            // decode content
            let info: FriendshipWithUser4Response =
                bincode::deserialize(&msg.content).map_err(|e| e.to_string())?;
            Ok(Msg::RecRelationship(FriendShipWithUser::from(info)))
        }
        MsgType::FriendApplyResp => {
            // decode content
            let info = bincode::deserialize(&msg.content).map_err(|e| e.to_string())?;
            Ok(Msg::RelationshipRes(info))
        }
        MsgType::SingleCallInvite => {
            let invite_type = match ContentType::from(msg.content_type) {
                ContentType::VideoCall => InviteType::Video,
                ContentType::AudioCall => InviteType::Audio,
                _ => return Err("Invalid content type".to_string()),
            };
            Ok(Msg::SingleCall(SingleCall::Invite(InviteMsg {
                msg_id: msg.server_id.into(),
                send_id: msg.send_id.into(),
                friend_id: msg.receiver_id.into(),
                create_time: msg.send_time,
                invite_type,
            })))
        }
        MsgType::SingleCallInviteAnswer => {
            let invite_type = match ContentType::from(msg.content_type) {
                ContentType::VideoCall => InviteType::Video,
                ContentType::AudioCall => InviteType::Audio,
                _ => return Err("Invalid content type".to_string()),
            };
            Ok(Msg::SingleCall(SingleCall::InviteAnswer(InviteAnswerMsg {
                local_id: msg.local_id.into(),
                server_id: msg.server_id.into(),
                send_id: msg.send_id.into(),
                friend_id: msg.receiver_id.into(),
                create_time: msg.send_time,
                invite_type,
                agree: msg.call_agree,
                is_self: false,
                send_time: msg.send_time,
                is_success: true,
            })))
        }
        MsgType::SingleCallInviteNotAnswer => {
            let invite_type = match ContentType::from(msg.content_type) {
                ContentType::VideoCall => InviteType::Video,
                ContentType::AudioCall => InviteType::Audio,
                _ => return Err("Invalid content type".to_string()),
            };
            Ok(Msg::SingleCall(SingleCall::NotAnswer(InviteNotAnswerMsg {
                local_id: msg.local_id.into(),
                server_id: msg.server_id.into(),
                send_id: msg.send_id.into(),
                friend_id: msg.receiver_id.into(),
                create_time: msg.create_time,
                invite_type,
                is_self: false,
                send_time: msg.send_time,
                is_success: true,
            })))
        }
        MsgType::SingleCallInviteCancel => {
            let invite_type = match ContentType::from(msg.content_type) {
                ContentType::VideoCall => InviteType::Video,
                ContentType::AudioCall => InviteType::Audio,
                _ => return Err("Invalid content type".to_string()),
            };
            Ok(Msg::SingleCall(SingleCall::InviteCancel(InviteCancelMsg {
                local_id: msg.local_id.into(),
                server_id: msg.server_id.into(),
                send_id: msg.send_id.into(),
                friend_id: msg.receiver_id.into(),
                create_time: msg.create_time,
                invite_type,
                is_self: false,
                send_time: msg.send_time,
                is_success: true,
            })))
        }
        MsgType::SingleCallOffer => Ok(Msg::SingleCall(SingleCall::Offer(Offer {
            send_id: msg.send_id.into(),
            friend_id: msg.receiver_id.into(),
            create_time: msg.send_time,
            sdp: msg.sdp.ok_or_else(|| String::from("sdp is empty"))?.into(),
        }))),
        MsgType::Hangup => Ok(Msg::SingleCall(SingleCall::HangUp(Hangup {
            local_id: msg.local_id.into(),
            server_id: msg.server_id.into(),
            send_id: msg.send_id.into(),
            friend_id: msg.receiver_id.into(),
            create_time: msg.create_time,
            send_time: msg.send_time,
            invite_type: InviteType::Audio,
            sustain: 0,
            is_self: false,
            is_success: true,
        }))),
        MsgType::AgreeSingleCall => Ok(Msg::SingleCall(SingleCall::Agree(Agree {
            send_id: msg.send_id.into(),
            friend_id: msg.receiver_id.into(),
            create_time: msg.send_time,
            sdp: msg.sdp,
        }))),
        MsgType::Candidate => Ok(Msg::SingleCall(SingleCall::NewIceCandidate(Candidate {
            candidate: String::from_utf8(msg.content)
                .map_err(|e| e.to_string())?
                .into(),
            sdp_mid: msg.sdp_mid,
            sdp_m_index: msg.sdp_m_index.map(|i| i as u16),
            send_id: msg.send_id.into(),
            friend_id: msg.receiver_id.into(),
            create_time: msg.send_time,
        }))),
        MsgType::Read => todo!(),
        MsgType::MsgRecResp => Ok(Msg::ServerRecResp(ServerResponse {
            msg_id: msg.local_id.into(),
            success: msg.content.is_empty(),
            err_msg: None,
        })),
        MsgType::Notification => todo!(),
        MsgType::Service => todo!(),
    }
}

impl From<Msg> for PbMsg {
    fn from(value: Msg) -> Self {
        match value {
            Msg::Single(msg) => PbMsg {
                msg_type: MsgType::SingleMsg as i32,
                local_id: msg.local_id.as_str().into(),
                send_id: msg.send_id.as_str().into(),
                receiver_id: msg.friend_id.as_str().into(),
                create_time: msg.create_time,
                content_type: msg.content_type as i32,
                content: msg.content.as_bytes().to_vec(),
                ..Default::default()
            },
            Msg::Group(group_msg) => {
                let mut pb_msg = PbMsg::default();
                match group_msg {
                    GroupMsg::Message(msg) => {
                        pb_msg.msg_type = MsgType::GroupMsg as i32;
                        pb_msg.local_id = msg.local_id.as_str().into();
                        pb_msg.send_id = msg.send_id.as_str().into();
                        pb_msg.receiver_id = msg.friend_id.to_string();
                        pb_msg.create_time = msg.create_time;
                        pb_msg.content_type = msg.content_type as i32;
                        pb_msg.content = msg.content.as_bytes().to_vec();
                        pb_msg.group_id = msg.friend_id.to_string();
                    }
                    GroupMsg::Invitation(info) => {
                        pb_msg.msg_type = MsgType::GroupInvitation as i32;
                        pb_msg.content = bincode::serialize(&info).unwrap();
                    }
                    GroupMsg::MemberExit((send_id, group_id)) => {
                        pb_msg.msg_type = MsgType::GroupMemberExit as i32;
                        pb_msg.send_id = send_id.to_string();
                        pb_msg.group_id = group_id.to_string();
                    }
                    GroupMsg::Dismiss(group_id) => {
                        pb_msg.msg_type = MsgType::GroupDismiss as i32;
                        pb_msg.group_id = group_id.to_string();
                    }
                    GroupMsg::DismissOrExitReceived(_) => {}
                    GroupMsg::InvitationReceived(_) => {}
                }
                pb_msg
            }
            Msg::SingleCall(call) => {
                let mut pb_msg = PbMsg::default();
                match call {
                    SingleCall::Invite(invite) => {
                        pb_msg.msg_type = MsgType::SingleCallInvite as i32;
                        pb_msg.local_id = invite.msg_id.as_str().into();
                        pb_msg.send_id = invite.send_id.as_str().into();
                        pb_msg.receiver_id = invite.friend_id.as_str().into();
                        pb_msg.create_time = invite.create_time;
                        pb_msg.content_type = match invite.invite_type {
                            InviteType::Video => ContentType::VideoCall as i32,
                            InviteType::Audio => ContentType::AudioCall as i32,
                        };
                    }
                    SingleCall::InviteAnswer(answer) => {
                        pb_msg.msg_type = MsgType::SingleCallInviteAnswer as i32;
                        pb_msg.local_id = answer.local_id.as_str().into();
                        pb_msg.send_id = answer.send_id.as_str().into();
                        pb_msg.receiver_id = answer.friend_id.as_str().into();
                        pb_msg.create_time = answer.create_time;
                        pb_msg.call_agree = answer.agree;
                        pb_msg.content_type = match answer.invite_type {
                            InviteType::Video => ContentType::VideoCall as i32,
                            InviteType::Audio => ContentType::AudioCall as i32,
                        };
                    }
                    SingleCall::NotAnswer(not_answer) => {
                        pb_msg.msg_type = MsgType::SingleCallInviteNotAnswer as i32;
                        pb_msg.local_id = not_answer.local_id.as_str().into();
                        pb_msg.send_id = not_answer.send_id.as_str().into();
                        pb_msg.receiver_id = not_answer.friend_id.as_str().into();
                        pb_msg.create_time = not_answer.create_time;
                        pb_msg.content_type = match not_answer.invite_type {
                            InviteType::Video => ContentType::VideoCall as i32,
                            InviteType::Audio => ContentType::AudioCall as i32,
                        };
                    }
                    SingleCall::InviteCancel(cancel) => {
                        pb_msg.msg_type = MsgType::SingleCallInviteCancel as i32;
                        pb_msg.local_id = cancel.local_id.as_str().into();
                        pb_msg.send_id = cancel.send_id.as_str().into();
                        pb_msg.receiver_id = cancel.friend_id.as_str().into();
                        pb_msg.create_time = cancel.create_time;
                        pb_msg.content_type = match cancel.invite_type {
                            InviteType::Video => ContentType::VideoCall as i32,
                            InviteType::Audio => ContentType::AudioCall as i32,
                        };
                    }
                    SingleCall::Offer(offer) => {
                        pb_msg.msg_type = MsgType::SingleCallOffer as i32;
                        pb_msg.send_id = offer.send_id.as_str().into();
                        pb_msg.receiver_id = offer.friend_id.as_str().into();
                        pb_msg.create_time = offer.create_time;
                        pb_msg.sdp = Some(offer.sdp.to_string());
                    }
                    SingleCall::Agree(agree) => {
                        pb_msg.msg_type = MsgType::AgreeSingleCall as i32;
                        pb_msg.send_id = agree.send_id.as_str().into();
                        pb_msg.receiver_id = agree.friend_id.as_str().into();
                        pb_msg.create_time = agree.create_time;
                        pb_msg.sdp = agree.sdp;
                    }
                    SingleCall::HangUp(hangup) => {
                        pb_msg.msg_type = MsgType::Hangup as i32;
                        pb_msg.send_id = hangup.send_id.as_str().into();
                        pb_msg.receiver_id = hangup.friend_id.as_str().into();
                        pb_msg.create_time = hangup.create_time;
                    }
                    SingleCall::NewIceCandidate(candidate) => {
                        pb_msg.msg_type = MsgType::Candidate as i32;
                        pb_msg.send_id = candidate.send_id.as_str().into();
                        pb_msg.receiver_id = candidate.friend_id.as_str().into();
                        pb_msg.create_time = candidate.create_time;
                        pb_msg.sdp_mid = candidate.sdp_mid;
                        pb_msg.sdp_m_index = candidate.sdp_m_index.map(|c| c as i32);
                        pb_msg.content = candidate.candidate.as_bytes().to_vec();
                    }
                }
                pb_msg
            }
            Msg::SendRelationshipReq(msg) => PbMsg {
                msg_type: MsgType::FriendApplyReq as i32,
                content: bincode::serialize(&msg).unwrap(),
                ..Default::default()
            },
            Msg::RecRelationship(_) => PbMsg {
                msg_type: MsgType::FriendApplyReq as i32,
                ..Default::default()
            },
            Msg::RelationshipRes(_) => PbMsg {
                msg_type: MsgType::FriendApplyResp as i32,
                ..Default::default()
            },
            Msg::ReadNotice(_) => PbMsg {
                msg_type: MsgType::Read as i32,
                ..Default::default()
            },

            Msg::SingleDeliveredNotice(_) => PbMsg::default(),
            Msg::FriendshipDeliveredNotice(_) => PbMsg::default(),
            Msg::OfflineSync(_) => PbMsg::default(),
            Msg::ServerRecResp(_) => PbMsg::default(),
        }
    }
}
