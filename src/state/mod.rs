use serde::{Deserialize, Serialize};
use yew::AttrValue;
use yewdux::Store;

use crate::{
    i18n::LanguageType,
    model::{
        friend::Friend,
        group::Group,
        message::{InviteMsg, Msg, ServerResponse},
        user::User,
        ComponentType, CurrentItem, RightContentType,
    },
};

/// offline message. notify other components after offline handled complete
#[derive(Store, Debug, Default, Clone, PartialEq)]
pub struct OfflineMsgState {
    pub complete: (),
}

/// language type
#[derive(Debug, Default, Clone, PartialEq, Store)]
pub struct I18nState {
    pub lang: LanguageType,
}

/// component type,
#[derive(Default, Debug, Clone, PartialEq, Store)]
pub struct AppState {
    pub component_type: ComponentType,
    pub login_user: User,
}

/// global unread count and contacts count(add friends)
#[derive(Store, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnreadState {
    pub msg_count: usize,
    pub contacts_count: usize,
}

/// notify other components after received a message
#[derive(Default, Clone, PartialEq, Debug, Store)]
pub struct RecMessageState {
    pub msg: Msg,
}

/// mute conversation in chats component and set window com
#[derive(Default, Debug, Clone, PartialEq, Store)]
pub struct MuteState {
    pub conv_id: AttrValue,
}

/// to notify chats component to remove conversation by id
#[derive(Default, Debug, Clone, PartialEq, Store)]
pub struct RemoveConvState {
    pub id: AttrValue,
}

/// to notify contacts component to remove friend item by id
#[derive(Default, Debug, Clone, PartialEq, Store)]
pub struct RemoveFriendState {
    pub id: AttrValue,
    pub type_: ItemType,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum ItemType {
    Group,
    #[default]
    Friend,
}

/// send message from send or home component
#[derive(Default, Clone, PartialEq, Debug, Store)]
pub struct SendMessageState {
    pub msg: Msg,
}

#[derive(Default, Clone, PartialEq, Debug, Store)]
pub struct SendCallState {
    pub msg: InviteMsg,
}

/// send message result, success or failed or timeout
#[derive(Default, Debug, Clone, PartialEq, Store)]
pub struct SendResultState {
    pub msg: ServerResponse,
}

#[derive(Default, Clone, PartialEq, Debug, Store)]
pub struct CreateConvState {
    pub type_: RightContentType,
    // 可以是好友，或者其他实现了   ItemInfo的类型
    // pub friend: Option<Friend>,
    // 创建群聊，接收一个NodeList，在chats中会生成群聊
    pub group: Option<Vec<String>>,
}

impl CreateConvState {
    pub fn create_group(&mut self, group: Vec<String>) {
        self.group = Some(group);
        self.type_ = RightContentType::Group;
        // self.friend = None;
    }

    // pub fn create_friend(&mut self, friend: Friend) {
    //     self.friend = Some(friend);
    //     self.type_ = RightContentType::Friend;
    //     self.group = None;
    // }
}

#[derive(Default, Clone, PartialEq, Store)]
pub struct AddFriendState {
    pub item: AddFriendStateItem,
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

/// current conversation id and type
#[derive(Default, Debug, Clone, PartialEq, Store)]
pub struct ConvState {
    pub conv: CurrentItem,
}

/// current friend id and type
#[derive(Default, Debug, Clone, PartialEq, Store)]
pub struct FriendListState {
    pub friend: CurrentItem,
}
