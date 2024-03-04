use serde::{Deserialize, Serialize};
use yew::{AttrValue, Callback};
use yew_router::Routable;

use crate::db::RightContentType;
use crate::model::friend::{Friend, FriendShipWithUser};
use crate::model::message::{InviteMsg, Msg};
use crate::model::user::User;

pub mod home;
pub mod login;
pub mod register;

// 1. 对话卡片切换
// 2. 朋友卡片切换
// 3. 消息收发
// 4. 全局组件切换

#[derive(Default, Clone, PartialEq)]
pub struct AppState {
    pub component_type: ComponentType,
    pub switch_com_event: Callback<ComponentType>,
    pub unread_count: i32,
    pub login_user: User,
}

#[derive(Default, Clone, PartialEq)]
pub struct WaitState {
    pub wait_count: usize,
    pub ready: Callback<()>,
}

#[derive(Default, Clone, PartialEq)]
pub struct UnreadMsgCountState {
    pub count: usize,
    pub add: Callback<usize>,
    pub sub: Callback<usize>,
}

/// 收发消息状态，收到消息触发receive_msg_event回调，发送消息通过send_msg_event回调来发送
/// msg保存当前收到的消息或者正在发送的消息内容
#[derive(Default, Clone, PartialEq)]
pub struct RecSendMessageState {
    pub msg: Msg,
    // pub receive_msg_event: Callback<Msg>,
    pub send_msg_event: Callback<Msg>,
    pub call_event: Callback<InviteMsg>,
}
#[derive(Default, Clone, PartialEq)]
pub struct RecSendCallState {
    pub msg: InviteMsg,
    pub send_msg_event: Callback<Msg>,
    pub rec_msg_event: Callback<Msg>,
    pub call_event: Callback<InviteMsg>,
}

/// 记录当前会话状态
#[derive(Default, Debug, Clone, PartialEq)]
pub struct ConvState {
    pub conv: CurrentItem,
    pub state_change_event: Callback<CurrentItem>,
}

/// 记录当前朋友列表状态
#[derive(Default, Clone, PartialEq)]
pub struct FriendListState {
    pub friend: CurrentItem,
    pub state_change_event: Callback<CurrentItem>,
}

/// 好友请求状态，当收到好友请求时触发状态改变的钩子
#[derive(Default, Clone, PartialEq, Debug)]
pub struct FriendShipState {
    // Request(FriendShipReq),
    // Response(FriendShipRes)
    pub ship: Option<FriendShipWithUser>,
    pub friend: Option<Friend>,
    pub state_type: FriendShipStateType,
    pub req_change_event: Callback<FriendShipWithUser>,
    pub res_change_event: Callback<(AttrValue, Friend)>,
}

#[derive(Default, Clone, PartialEq, Debug)]
pub enum FriendShipStateType {
    #[default]
    Req,
    Res,
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
    pub unread_count: usize,
    pub item_id: AttrValue,
    pub content_type: RightContentType,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct CommonProps {
    pub id: AttrValue,
    pub name: AttrValue,
    pub avatar: AttrValue,
    pub time: i64,
    pub remark: AttrValue,
}

// 定义路由
#[derive(Clone, PartialEq, Routable)]
pub enum Page {
    #[at("/:id")]
    Home { id: AttrValue },
    #[at("/login")]
    Login,
    #[at("/register")]
    Register,
    #[at("/")]
    Redirect,
}
