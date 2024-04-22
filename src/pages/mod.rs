pub mod home_page;
pub mod login;
pub mod register;

use yew::{AttrValue, Callback};
use yew_router::Routable;

use crate::model::friend::{Friend, FriendShipWithUser};
use crate::model::group::Group;
use crate::model::message::{InviteMsg, Msg, ServerResponse};
use crate::model::{CurrentItem, FriendShipStateType, RightContentType};

/// 收发消息状态，收到消息触发receive_msg_event回调，发送消息通过send_msg_event回调来发送
/// msg保存当前收到的消息或者正在发送的消息内容
/// 将收发消息状态切割
#[derive(Default, Clone, PartialEq, Debug)]
pub struct SendMessageState {
    pub msg: Msg,
    pub send_msg_event: Callback<Msg>,
    // dail a single call
    pub call_event: Callback<InviteMsg>,
}

#[derive(Default, Clone, PartialEq, Debug)]
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

#[derive(Default, Debug, Clone, PartialEq)]
pub struct SendResultState {
    pub msg: ServerResponse,
    pub notify: Callback<ServerResponse>,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct RemoveConvState {
    pub id: AttrValue,
    pub remove_event: Callback<AttrValue>,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum ItemType {
    Group,
    #[default]
    Friend,
}
#[derive(Default, Debug, Clone, PartialEq)]
pub struct RemoveFriendState {
    pub id: AttrValue,
    pub type_: ItemType,
    pub remove_event: Callback<(AttrValue, ItemType)>,
}

impl RemoveFriendState {
    pub fn with_event(event: Callback<(AttrValue, ItemType)>) -> Self {
        Self {
            remove_event: event,
            ..Default::default()
        }
    }
}

/// 记录当前朋友列表状态
#[derive(Default, Clone, PartialEq)]
pub struct FriendListState {
    pub friend: CurrentItem,
    pub state_change_event: Callback<CurrentItem>,
}

/// 记录当前朋友列表状态
#[derive(Default, Clone, PartialEq)]
pub struct AddFriendState {
    pub item: AddFriendStateItem,
    pub add: Callback<AddFriendStateItem>,
}

/// 记录当前朋友列表状态
#[derive(Default, Clone, PartialEq)]
pub struct AddFriendStateItem {
    pub friend: Option<Friend>,
    pub group: Option<Group>,
    pub type_: RightContentType,
}

impl From<Group> for AddFriendStateItem {
    fn from(value: Group) -> Self {
        Self {
            friend: None,
            group: Some(value),
            type_: RightContentType::Group,
        }
    }
}

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

#[derive(Default, Clone, PartialEq, Debug)]
pub struct CreateConvState {
    pub type_: RightContentType,
    // 可以是好友，或者其他实现了   ItemInfo的类型
    pub friend: Option<Friend>,
    // 创建群聊，接收一个NodeList，在chats中会生成群聊
    pub group: Option<Vec<String>>,
    pub create_friend: Callback<(RightContentType, Friend)>,
    pub create_group: Callback<(RightContentType, Vec<String>)>,
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
