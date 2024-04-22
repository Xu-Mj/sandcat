pub mod home_page;
pub mod login;
pub mod register;

use yew::{AttrValue, Callback};
use yew_router::Routable;

use crate::model::friend::{Friend, FriendShipWithUser};
use crate::model::FriendShipStateType;

/// 收发消息状态，收到消息触发receive_msg_event回调，发送消息通过send_msg_event回调来发送
/// msg保存当前收到的消息或者正在发送的消息内容
/// 将收发消息状态切割

#[derive(Default, Debug, Clone, PartialEq)]
pub enum ItemType {
    Group,
    #[default]
    Friend,
}

/// 记录当前朋友列表状态

/// 好友请求状态，当收到好友请求时触发状态改变的钩子
#[derive(Default, Clone, PartialEq, Debug)]
pub struct FriendShipState {
    // Request(FriendShipReq),
    // Response(FriendShipRes)
    pub ship: Option<FriendShipWithUser>,
    pub friend: Option<Friend>,
    pub state_type: FriendShipStateType,
    /// received the friend application request
    pub req_change_event: Callback<FriendShipWithUser>,
    /// received the friend application response
    pub rec_resp: Callback<Friend>,
    /// response the friend application request
    pub res_change_event: Callback<(AttrValue, Friend)>,
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
