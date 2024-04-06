/// / decode message content by content type
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Msg {
    /// must have
    #[prost(string, tag = "1")]
    pub send_id: ::prost::alloc::string::String,
    /// must have
    #[prost(string, tag = "2")]
    pub receiver_id: ::prost::alloc::string::String,
    /// must have
    #[prost(string, tag = "3")]
    pub local_id: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub server_id: ::prost::alloc::string::String,
    /// timestamp
    #[prost(int64, tag = "5")]
    pub send_time: i64,
    #[prost(int64, tag = "6")]
    pub seq: i64,
    #[prost(string, tag = "7")]
    pub group_id: ::prost::alloc::string::String,
    /// is there necessary to cary the user's avatar and nickname?
    #[prost(enumeration = "MsgType", tag = "8")]
    pub msg_type: i32,
    #[prost(enumeration = "ContentType", tag = "9")]
    pub content_type: i32,
    #[prost(bytes = "vec", tag = "10")]
    pub content: ::prost::alloc::vec::Vec<u8>,
    #[prost(bool, tag = "11")]
    pub is_read: bool,
    #[prost(string, optional, tag = "12")]
    pub sdp: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "13")]
    pub sdp_mid: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(int32, optional, tag = "14")]
    pub sdp_m_index: ::core::option::Option<i32>,
    #[prost(bool, tag = "15")]
    pub call_agree: bool,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgRead {
    #[prost(string, tag = "1")]
    pub msg_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub user_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub seq: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Candidate {
    #[prost(string, tag = "1")]
    pub candidate: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "2")]
    pub sdp_mid: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(int32, optional, tag = "3")]
    pub sdp_m_index: ::core::option::Option<i32>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AgreeSingleCall {
    #[prost(string, tag = "1")]
    pub sdp: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SingleCallInvite {
    #[prost(enumeration = "SingleCallInviteType", tag = "1")]
    pub invite_type: i32,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SingleCallInviteAnswer {
    #[prost(bool, tag = "1")]
    pub agree: bool,
    #[prost(enumeration = "SingleCallInviteType", tag = "2")]
    pub invite_type: i32,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SingleCallInviteNotAnswer {
    #[prost(enumeration = "SingleCallInviteType", tag = "1")]
    pub invite_type: i32,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SingleCallInviteCancel {
    #[prost(enumeration = "SingleCallInviteType", tag = "2")]
    pub invite_type: i32,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SingleCallOffer {
    #[prost(string, tag = "1")]
    pub sdp: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Hangup {
    #[prost(enumeration = "SingleCallInviteType", tag = "1")]
    pub invite_type: i32,
    #[prost(int64, tag = "2")]
    pub sustain: i64,
}
/// / use to send single message or group message;
/// / message ws is used to connect the client by websocket;
/// / and it receive message from clients; then send message to mq;
/// / so only provide the send message function for other rpc service;
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Single {
    /// message content
    #[prost(string, tag = "2")]
    pub content: ::prost::alloc::string::String,
    /// message type
    #[prost(enumeration = "ContentType", tag = "3")]
    pub content_type: i32,
}
/// / user and group id
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserAndGroupId {
    #[prost(string, tag = "1")]
    pub user_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub group_id: ::prost::alloc::string::String,
}
/// / group invitation include group information and group member information
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GroupInvitation {
    #[prost(message, optional, tag = "1")]
    pub info: ::core::option::Option<GroupInfo>,
    #[prost(message, repeated, tag = "2")]
    pub members: ::prost::alloc::vec::Vec<GroupMember>,
}
/// / group information also related to database
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GroupInfo {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub owner: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub avatar: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub description: ::prost::alloc::string::String,
    #[prost(string, tag = "6")]
    pub announcement: ::prost::alloc::string::String,
    #[prost(int64, tag = "7")]
    pub create_time: i64,
    #[prost(int64, tag = "8")]
    pub update_time: i64,
}
/// / group member information also related to database table group_members
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GroupMember {
    #[prost(int64, tag = "1")]
    pub id: i64,
    #[prost(int32, tag = "2")]
    pub age: i32,
    #[prost(string, tag = "3")]
    pub group_id: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub user_id: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub group_name: ::prost::alloc::string::String,
    #[prost(string, tag = "6")]
    pub avatar: ::prost::alloc::string::String,
    #[prost(int64, tag = "7")]
    pub joined_at: i64,
    #[prost(string, optional, tag = "8")]
    pub region: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, tag = "9")]
    pub gender: ::prost::alloc::string::String,
    #[prost(bool, tag = "10")]
    pub is_friend: bool,
    #[prost(string, optional, tag = "11")]
    pub remark: ::core::option::Option<::prost::alloc::string::String>,
}
/// / create group object
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GroupCreate {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub owner: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub avatar: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub group_name: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "5")]
    pub members_id: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GroupInviteNew {
    #[prost(string, tag = "1")]
    pub user_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub group_id: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "3")]
    pub members: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GroupUpdate {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub avatar: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub description: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub announcement: ::prost::alloc::string::String,
    #[prost(int64, tag = "6")]
    pub update_time: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct User {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub account: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    #[serde(skip_serializing)]
    pub password: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub avatar: ::prost::alloc::string::String,
    #[prost(string, tag = "6")]
    pub gender: ::prost::alloc::string::String,
    #[prost(int32, tag = "7")]
    pub age: i32,
    #[prost(string, optional, tag = "8")]
    pub phone: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "9")]
    pub email: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "10")]
    pub address: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "11")]
    pub region: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(int64, optional, tag = "12")]
    pub birthday: ::core::option::Option<i64>,
    #[prost(int64, tag = "13")]
    pub create_time: i64,
    #[prost(int64, tag = "14")]
    pub update_time: i64,
    #[prost(string, tag = "15")]
    pub salt: ::prost::alloc::string::String,
    #[prost(string, tag = "16")]
    pub signature: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UserWithMatchType {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub account: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub avatar: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub gender: ::prost::alloc::string::String,
    #[prost(int32, tag = "6")]
    pub age: i32,
    #[prost(string, optional, tag = "7")]
    pub phone: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "8")]
    pub email: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "9")]
    pub address: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "10")]
    pub region: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(int64, optional, tag = "11")]
    pub birthday: ::core::option::Option<i64>,
    #[prost(string, optional, tag = "12")]
    pub match_type: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, tag = "13")]
    pub signature: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Friendship {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub user_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub friend_id: ::prost::alloc::string::String,
    #[prost(enumeration = "FriendshipStatus", tag = "4")]
    pub status: i32,
    #[prost(string, optional, tag = "5")]
    pub apply_msg: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "6")]
    pub req_remark: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "7")]
    pub resp_msg: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "8")]
    pub resp_remark: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, tag = "9")]
    pub source: ::prost::alloc::string::String,
    #[prost(int64, tag = "10")]
    pub create_time: i64,
    #[prost(int64, tag = "11")]
    pub accept_time: i64,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FriendshipWithUser {
    #[prost(string, tag = "1")]
    pub fs_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub user_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub avatar: ::prost::alloc::string::String,
    #[prost(string, tag = "5")]
    pub gender: ::prost::alloc::string::String,
    #[prost(int32, tag = "6")]
    pub age: i32,
    #[prost(string, optional, tag = "7")]
    pub region: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(enumeration = "FriendshipStatus", tag = "8")]
    pub status: i32,
    #[prost(string, optional, tag = "9")]
    pub apply_msg: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, tag = "10")]
    pub source: ::prost::alloc::string::String,
    #[prost(int64, tag = "11")]
    pub create_time: i64,
    #[prost(string, tag = "12")]
    pub account: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Friend {
    /// / friendship related user's id
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub avatar: ::prost::alloc::string::String,
    #[prost(string, tag = "4")]
    pub gender: ::prost::alloc::string::String,
    #[prost(int32, tag = "5")]
    pub age: i32,
    #[prost(string, optional, tag = "6")]
    pub region: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(enumeration = "FriendshipStatus", tag = "7")]
    pub status: i32,
    #[prost(string, optional, tag = "8")]
    pub hello: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "9")]
    pub remark: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, tag = "10")]
    pub source: ::prost::alloc::string::String,
    #[prost(int64, tag = "11")]
    pub accept_time: i64,
    #[prost(string, tag = "12")]
    pub account: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FsCreateRequest {
    #[prost(message, optional, tag = "1")]
    pub fs_create: ::core::option::Option<FsCreate>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FsCreate {
    #[prost(string, tag = "1")]
    pub user_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub friend_id: ::prost::alloc::string::String,
    #[prost(enumeration = "FriendshipStatus", tag = "3")]
    pub status: i32,
    #[prost(string, optional, tag = "4")]
    pub apply_msg: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "5")]
    pub req_remark: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, tag = "6")]
    pub source: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FsCreateResponse {
    #[prost(message, optional, tag = "1")]
    pub fs_req: ::core::option::Option<FriendshipWithUser>,
    #[prost(message, optional, tag = "2")]
    pub fs_send: ::core::option::Option<FriendshipWithUser>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FsAgreeRequest {
    #[prost(message, optional, tag = "1")]
    pub fs_reply: ::core::option::Option<AgreeReply>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FsAgreeResponse {
    #[prost(message, optional, tag = "1")]
    pub req: ::core::option::Option<Friend>,
    #[prost(message, optional, tag = "2")]
    pub send: ::core::option::Option<Friend>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateRemarkRequest {
    #[prost(string, tag = "1")]
    pub user_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub friend_id: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub remark: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateRemarkResponse {}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AgreeReply {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, optional, tag = "2")]
    pub resp_msg: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, optional, tag = "3")]
    pub resp_remark: ::core::option::Option<::prost::alloc::string::String>,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FsListRequest {
    #[prost(string, tag = "1")]
    pub user_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FriendListRequest {
    #[prost(string, tag = "1")]
    pub user_id: ::prost::alloc::string::String,
}
/// / only for update friend apply request
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FsUpdate {
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub apply_msg: ::prost::alloc::string::String,
    #[prost(string, tag = "3")]
    pub req_remark: ::prost::alloc::string::String,
}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FsUpdateRequest {
    #[prost(message, optional, tag = "1")]
    pub fs_update: ::core::option::Option<FsUpdate>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FsListResponse {
    #[prost(message, repeated, tag = "1")]
    pub friendships: ::prost::alloc::vec::Vec<FriendshipWithUser>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FriendListResponse {
    #[prost(message, repeated, tag = "1")]
    pub friends: ::prost::alloc::vec::Vec<Friend>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GroupInviteNewRequest {
    #[prost(message, optional, tag = "1")]
    pub group_invite: ::core::option::Option<GroupInviteNew>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GroupInviteNewResp {
    #[prost(message, repeated, tag = "1")]
    pub members: ::prost::alloc::vec::Vec<GroupMember>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GroupUpdateRequest {
    #[prost(message, optional, tag = "1")]
    pub group: ::core::option::Option<GroupUpdate>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GroupUpdateResponse {
    #[prost(message, optional, tag = "1")]
    pub group: ::core::option::Option<GroupInfo>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GroupDeleteRequest {
    #[prost(string, tag = "1")]
    pub group_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub user_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GroupDeleteResponse {
    ///   repeated string members_id = 1;
    #[prost(message, optional, tag = "1")]
    pub group: ::core::option::Option<GroupInfo>,
}
///   repeated string members_id = 1;
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GroupMemberExitResponse {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GroupMembersIdRequest {
    #[prost(string, tag = "1")]
    pub group_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GroupMembersIdResponse {
    #[prost(string, repeated, tag = "1")]
    pub members_id: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateUserRequest {
    #[prost(message, optional, tag = "1")]
    pub user: ::core::option::Option<User>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateUserResponse {
    #[prost(message, optional, tag = "1")]
    pub user: ::core::option::Option<User>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetUserRequest {
    #[prost(string, tag = "1")]
    pub user_id: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetUserResponse {
    #[prost(message, optional, tag = "1")]
    pub user: ::core::option::Option<User>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateUserRequest {
    #[prost(message, optional, tag = "1")]
    pub user: ::core::option::Option<User>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateUserResponse {
    #[prost(message, optional, tag = "1")]
    pub user: ::core::option::Option<User>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SearchUserRequest {
    #[prost(string, tag = "1")]
    pub user_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub pattern: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SearchUserResponse {
    #[prost(message, repeated, tag = "1")]
    pub users: ::prost::alloc::vec::Vec<UserWithMatchType>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VerifyPwdRequest {
    /// / could be account, email or phone number
    #[prost(string, tag = "1")]
    pub account: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub password: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct VerifyPwdResponse {
    #[prost(message, optional, tag = "1")]
    pub user: ::core::option::Option<User>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SendMsgRequest {
    #[prost(message, optional, tag = "1")]
    pub message: ::core::option::Option<Msg>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SendGroupMsgRequest {
    #[prost(message, optional, tag = "1")]
    pub message: ::core::option::Option<Msg>,
    #[prost(string, repeated, tag = "2")]
    pub members_id: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SendMsgResponse {}
#[derive(serde::Serialize, serde::Deserialize)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MsgResponse {
    #[prost(string, tag = "1")]
    pub local_id: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub server_id: ::prost::alloc::string::String,
    #[prost(int64, tag = "3")]
    pub send_time: i64,
    #[prost(string, tag = "4")]
    pub err: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SaveMessageRequest {
    #[prost(message, optional, tag = "1")]
    pub message: ::core::option::Option<Msg>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SaveMessageResponse {}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetDbMsgRequest {
    #[prost(string, tag = "1")]
    pub user_id: ::prost::alloc::string::String,
    #[prost(int64, tag = "2")]
    pub start: i64,
    #[prost(int64, tag = "3")]
    pub end: i64,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GroupCreateRequest {
    #[prost(message, optional, tag = "1")]
    pub group: ::core::option::Option<GroupCreate>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GroupCreateResponse {
    #[prost(message, optional, tag = "1")]
    pub invitation: ::core::option::Option<GroupInvitation>,
}
/// / message content type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ContentType {
    Default = 0,
    Text = 1,
    Image = 2,
    Video = 3,
    File = 4,
    Emoji = 5,
    Audio = 6,
    VideoCall = 7,
    AudioCall = 8,
}
impl ContentType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ContentType::Default => "Default",
            ContentType::Text => "Text",
            ContentType::Image => "Image",
            ContentType::Video => "Video",
            ContentType::File => "File",
            ContentType::Emoji => "Emoji",
            ContentType::Audio => "Audio",
            ContentType::VideoCall => "VideoCall",
            ContentType::AudioCall => "AudioCall",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Default" => Some(Self::Default),
            "Text" => Some(Self::Text),
            "Image" => Some(Self::Image),
            "Video" => Some(Self::Video),
            "File" => Some(Self::File),
            "Emoji" => Some(Self::Emoji),
            "Audio" => Some(Self::Audio),
            "VideoCall" => Some(Self::VideoCall),
            "AudioCall" => Some(Self::AudioCall),
            _ => None,
        }
    }
}
/// / friendship status
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum FriendshipStatus {
    /// / default status
    Default = 0,
    Pending = 1,
    Accepted = 2,
    Rejected = 3,
    /// / blacklist
    Blacked = 4,
    Canceled = 5,
    Deleted = 6,
}
impl FriendshipStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            FriendshipStatus::Default => "FriendshipStatusDefault",
            FriendshipStatus::Pending => "Pending",
            FriendshipStatus::Accepted => "Accepted",
            FriendshipStatus::Rejected => "Rejected",
            FriendshipStatus::Blacked => "Blacked",
            FriendshipStatus::Canceled => "Canceled",
            FriendshipStatus::Deleted => "Deleted",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "FriendshipStatusDefault" => Some(Self::Default),
            "Pending" => Some(Self::Pending),
            "Accepted" => Some(Self::Accepted),
            "Rejected" => Some(Self::Rejected),
            "Blacked" => Some(Self::Blacked),
            "Canceled" => Some(Self::Canceled),
            "Deleted" => Some(Self::Deleted),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum MsgType {
    SingleMsg = 0,
    GroupMsg = 1,
    GroupInvitation = 2,
    GroupInviteNew = 3,
    GroupMemberExit = 4,
    GroupDismiss = 5,
    GroupDismissOrExitReceived = 6,
    GroupInvitationReceived = 7,
    GroupUpdate = 8,
    FriendApplyReq = 9,
    FriendApplyResp = 10,
    SingleCallInvite = 11,
    SingleCallInviteAnswer = 12,
    SingleCallInviteNotAnswer = 13,
    SingleCallInviteCancel = 14,
    SingleCallOffer = 15,
    Hangup = 16,
    AgreeSingleCall = 17,
    Candidate = 18,
    Read = 19,
    MsgRecResp = 20,
    Notification = 21,
    Service = 22,
}
impl MsgType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            MsgType::SingleMsg => "MsgTypeSingleMsg",
            MsgType::GroupMsg => "MsgTypeGroupMsg",
            MsgType::GroupInvitation => "MsgTypeGroupInvitation",
            MsgType::GroupInviteNew => "MsgTypeGroupInviteNew",
            MsgType::GroupMemberExit => "MsgTypeGroupMemberExit",
            MsgType::GroupDismiss => "MsgTypeGroupDismiss",
            MsgType::GroupDismissOrExitReceived => "MsgTypeGroupDismissOrExitReceived",
            MsgType::GroupInvitationReceived => "MsgTypeGroupInvitationReceived",
            MsgType::GroupUpdate => "MsgTypeGroupUpdate",
            MsgType::FriendApplyReq => "MsgTypeFriendApplyReq",
            MsgType::FriendApplyResp => "MsgTypeFriendApplyResp",
            MsgType::SingleCallInvite => "MsgTypeSingleCallInvite",
            MsgType::SingleCallInviteAnswer => "MsgTypeSingleCallInviteAnswer",
            MsgType::SingleCallInviteNotAnswer => "MsgTypeSingleCallInviteNotAnswer",
            MsgType::SingleCallInviteCancel => "MsgTypeSingleCallInviteCancel",
            MsgType::SingleCallOffer => "MsgTypeSingleCallOffer",
            MsgType::Hangup => "MsgTypeHangup",
            MsgType::AgreeSingleCall => "MsgTypeAgreeSingleCall",
            MsgType::Candidate => "MsgTypeCandidate",
            MsgType::Read => "MsgTypeRead",
            MsgType::MsgRecResp => "MsgTypeMsgRecResp",
            MsgType::Notification => "MsgTypeNotification",
            MsgType::Service => "MsgTypeService",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "MsgTypeSingleMsg" => Some(Self::SingleMsg),
            "MsgTypeGroupMsg" => Some(Self::GroupMsg),
            "MsgTypeGroupInvitation" => Some(Self::GroupInvitation),
            "MsgTypeGroupInviteNew" => Some(Self::GroupInviteNew),
            "MsgTypeGroupMemberExit" => Some(Self::GroupMemberExit),
            "MsgTypeGroupDismiss" => Some(Self::GroupDismiss),
            "MsgTypeGroupDismissOrExitReceived" => Some(Self::GroupDismissOrExitReceived),
            "MsgTypeGroupInvitationReceived" => Some(Self::GroupInvitationReceived),
            "MsgTypeGroupUpdate" => Some(Self::GroupUpdate),
            "MsgTypeFriendApplyReq" => Some(Self::FriendApplyReq),
            "MsgTypeFriendApplyResp" => Some(Self::FriendApplyResp),
            "MsgTypeSingleCallInvite" => Some(Self::SingleCallInvite),
            "MsgTypeSingleCallInviteAnswer" => Some(Self::SingleCallInviteAnswer),
            "MsgTypeSingleCallInviteNotAnswer" => Some(Self::SingleCallInviteNotAnswer),
            "MsgTypeSingleCallInviteCancel" => Some(Self::SingleCallInviteCancel),
            "MsgTypeSingleCallOffer" => Some(Self::SingleCallOffer),
            "MsgTypeHangup" => Some(Self::Hangup),
            "MsgTypeAgreeSingleCall" => Some(Self::AgreeSingleCall),
            "MsgTypeCandidate" => Some(Self::Candidate),
            "MsgTypeRead" => Some(Self::Read),
            "MsgTypeMsgRecResp" => Some(Self::MsgRecResp),
            "MsgTypeNotification" => Some(Self::Notification),
            "MsgTypeService" => Some(Self::Service),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SingleCallInviteType {
    SingleAudio = 0,
    SingleVideo = 1,
}
impl SingleCallInviteType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            SingleCallInviteType::SingleAudio => "SingleAudio",
            SingleCallInviteType::SingleVideo => "SingleVideo",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "SingleAudio" => Some(Self::SingleAudio),
            "SingleVideo" => Some(Self::SingleVideo),
            _ => None,
        }
    }
}
