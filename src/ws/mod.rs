mod manager;
use log::debug;
pub use manager::WebSocketManager;

use crate::{
    model::{
        friend::{FriendShipWithUser, FriendshipWithUser4Response},
        message::{
            Agree, Candidate, GroupMsg, Hangup, InviteAnswerMsg, InviteCancelMsg, InviteMsg,
            InviteNotAnswerMsg, InviteType, Message, Msg, Offer, ServerResponse, SingleCall,
        },
        ContentType,
    },
    pb::message::{Msg as PbMsg, MsgType},
};

fn convert(msg: PbMsg) -> Result<Msg, String> {
    debug!("convert msg: {:?}", msg);
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
                create_time: msg.send_time,
                invite_type,
                is_self: false,
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
                create_time: msg.send_time,
                invite_type,
                is_self: false,
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
            create_time: msg.send_time,
            invite_type: InviteType::Audio,
            sustain: 0,
            is_self: false,
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
