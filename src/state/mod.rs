use serde::{Deserialize, Serialize};
use yew::AttrValue;
use yewdux::Store;

use crate::{
    i18n::LanguageType,
    model::{
        message::{InviteMsg, Msg, ServerResponse},
        user::User,
        ComponentType,
    },
    pages::ItemType,
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
#[derive(Default, Clone, PartialEq, Store)]
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
