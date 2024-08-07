use std::{
    fmt::{Display, Formatter},
    rc::Rc,
};

use serde::{Deserialize, Serialize};
use yew::AttrValue;
use yewdux::{Dispatch, Store};

use i18n::LanguageType;

use crate::model::{
    conversation::Conversation,
    friend::{Friend, FriendShipWithUser},
    group::Group,
    message::{InviteMsg, Message, Msg, ServerResponse},
    user::User,
    ComponentType, CurrentItem, FriendShipStateType, RightContentType,
};

pub trait Notify: Sized + Store {
    fn notify(self) {
        Dispatch::<Self>::global().set(self);
    }

    fn get() -> Rc<Self> {
        Dispatch::<Self>::global().get()
    }
}

/// offline message. notify other components after offline handled complete
#[derive(Store, Debug, Default, Clone, PartialEq)]
pub struct RefreshMsgListState {
    pub refresh: bool,
}

/// language type
#[derive(Debug, Default, Clone, PartialEq, Store, Serialize, Deserialize)]
#[store(storage = "local")]
pub struct I18nState {
    pub lang: LanguageType,
}

impl Notify for I18nState {}

/// component type,
#[derive(Default, Debug, Clone, PartialEq, Store)]
pub struct AppState {
    // pub component_type: ComponentType,
    pub login_user: User,
}

impl Notify for AppState {}

/// component type,
#[derive(Default, Debug, Clone, PartialEq, Store, Serialize, Deserialize)]
#[store(storage = "local")]
pub struct ComponentTypeState {
    pub component_type: ComponentType,
}

impl Notify for ComponentTypeState {}

impl From<ComponentType> for ComponentTypeState {
    fn from(value: ComponentType) -> Self {
        Self {
            component_type: value,
        }
    }
}

/// global unread count and contacts count(add friends)
/// there is an issue that I've encountered which is difficult to understand.
/// If the state not stored, and it's not at default value,
/// subscribe do not receive the first change notification following a browser refresh.
#[derive(Store, Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[store(storage = "local")]
pub struct UnreadState {
    pub msg_count: usize,
    pub contacts_count: usize,
}

impl UnreadState {
    pub fn incr_msg(count: usize) {
        Dispatch::<Self>::global().reduce_mut(|s| s.msg_count = s.msg_count.saturating_add(count));
    }

    pub fn incr_contact(count: usize) {
        Dispatch::<Self>::global()
            .reduce_mut(|s| s.contacts_count = s.contacts_count.saturating_add(count));
    }
    pub fn decr_msg(count: usize) {
        Dispatch::<Self>::global().reduce_mut(|s| s.msg_count = s.msg_count.saturating_sub(count));
    }

    pub fn decr_contact(count: usize) {
        Dispatch::<Self>::global()
            .reduce_mut(|s| s.contacts_count = s.contacts_count.saturating_sub(count));
    }
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

impl Notify for RemoveConvState {}

impl RemoveConvState {
    pub fn remove(id: AttrValue) {
        Dispatch::<Self>::global().reduce_mut(|s| s.id = id);
    }
}

/// to notify contacts component to remove friend item by id
#[derive(Default, Debug, Clone, PartialEq, Store)]
pub struct RemoveFriendState {
    pub id: AttrValue,
    pub type_: ItemType,
}

impl RemoveFriendState {
    pub fn remove(id: AttrValue, type_: ItemType) {
        Dispatch::<Self>::global().reduce_mut(|s| {
            s.id = id;
            s.type_ = type_;
        });
    }
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

impl SendMessageState {
    pub fn send(msg: Msg) {
        Dispatch::<Self>::global().reduce_mut(|s| s.msg = msg);
    }
}

/// send audio message
#[derive(Default, Clone, PartialEq, Debug, Store)]
pub struct SendAudioMsgState {
    pub msg: Message,
}

/// send audio message
#[derive(Default, Clone, PartialEq, Debug, Store)]
pub struct AudioDownloadedState {
    pub local_id: AttrValue,
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
    pub conv: Conversation,
}

impl CreateConvState {
    pub fn update(conv: Conversation) {
        Dispatch::<Self>::global().reduce_mut(|s| s.conv = conv);
    }
}

#[derive(Default, Clone, PartialEq, Debug, Store)]
pub struct CreateGroupConvState {
    // 创建群聊，接收一个NodeList，在chats中会生成群聊
    pub group: Option<Vec<String>>,
}

impl CreateGroupConvState {
    pub fn create_group(&mut self, group: Vec<String>) {
        self.group = Some(group);
    }
}

#[derive(Default, Clone, PartialEq, Store)]
pub struct AddFriendState {
    pub item: AddFriendStateItem,
}

#[derive(Debug, Default, Clone, PartialEq, Store)]
pub struct UpdateFriendState {
    pub id: AttrValue,
    pub name: Option<AttrValue>,
    pub remark: Option<AttrValue>,
    pub avatar: Option<AttrValue>,
    pub type_: ItemType,
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
#[derive(Default, Debug, Clone, PartialEq, Store, Serialize, Deserialize)]
#[store(storage = "local")]
pub struct ConvState {
    pub conv: CurrentItem,
}

/// current friend id and type
#[derive(Default, Debug, Clone, PartialEq, Store, Serialize, Deserialize)]
#[store(storage = "local")]
pub struct FriendListState {
    pub friend: CurrentItem,
}

#[derive(Default, Clone, PartialEq, Debug, Store)]
pub struct FriendShipState {
    pub ship: Option<FriendShipWithUser>,
    pub friend: Option<Friend>,
    pub state_type: FriendShipStateType,
}

#[derive(Default, Clone, PartialEq, Debug, Store, Serialize, Deserialize)]
#[store(storage = "local")]
pub enum FontSizeState {
    Small,
    #[default]
    Medium,
    Large,
    Larger,
}

impl Notify for FontSizeState {}

impl From<&str> for FontSizeState {
    fn from(value: &str) -> Self {
        match value {
            "small" => Self::Small,
            "medium" => Self::Medium,
            "large" => Self::Large,
            "larger" => Self::Larger,
            _ => Self::Medium,
        }
    }
}

impl Display for FontSizeState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            FontSizeState::Small => write!(f, "small"),
            FontSizeState::Medium => write!(f, "medium"),
            FontSizeState::Large => write!(f, "large"),
            FontSizeState::Larger => write!(f, "larger"),
        }
    }
}

#[derive(Default, Clone, PartialEq, Debug, Store, Serialize, Deserialize)]
#[store(storage = "local")]
#[serde(rename_all = "lowercase")]
pub enum ThemeState {
    #[default]
    Light,
    Dark,
}

impl Notify for ThemeState {}

impl Display for ThemeState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeState::Light => write!(f, "light"),
            ThemeState::Dark => write!(f, "dark"),
        }
    }
}

impl From<&str> for ThemeState {
    fn from(value: &str) -> Self {
        match value {
            "light" => ThemeState::Light,
            "dark" => ThemeState::Dark,
            _ => ThemeState::Light,
        }
    }
}

#[derive(Default, Clone, PartialEq, Debug, Store, Serialize, Deserialize)]
#[store(storage = "local")]
pub struct TransparentState {
    pub value: f32,
}

impl TransparentState {
    pub fn get() -> f32 {
        Dispatch::<Self>::global().get().value
    }

    pub fn set(value: f32) {
        Dispatch::<Self>::global().reduce_mut(|s| s.value = value);
    }
}

#[derive(Default, Clone, PartialEq, Debug, Store, Serialize, Deserialize)]
#[store(storage = "local")]
#[serde(rename_all = "lowercase")]
#[repr(i32)]
pub enum MobileState {
    #[default]
    Desktop = 0,
    Mobile = 1,
}

impl MobileState {
    pub fn is_mobile() -> bool {
        match *Self::get() {
            MobileState::Desktop => false,
            MobileState::Mobile => true,
        }
    }
}

impl Notify for MobileState {}

impl From<&str> for MobileState {
    fn from(value: &str) -> Self {
        match value {
            "desktop" => MobileState::Desktop,
            "mobile" => MobileState::Mobile,
            _ => MobileState::Desktop,
        }
    }
}

impl Display for MobileState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MobileState::Desktop => write!(f, "desktop"),
            MobileState::Mobile => write!(f, "mobile"),
        }
    }
}

#[derive(Default, Clone, PartialEq, Debug, Store, Serialize, Deserialize)]
#[store(storage = "local")]
#[serde(rename_all = "lowercase")]
pub enum ShowRight {
    #[default]
    None,
    Show,
}

impl Notify for ShowRight {}

impl From<&str> for ShowRight {
    fn from(value: &str) -> Self {
        match value {
            "none" => ShowRight::None,
            "show" => ShowRight::Show,
            _ => ShowRight::None,
        }
    }
}

impl Display for ShowRight {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ShowRight::None => write!(f, "none"),
            ShowRight::Show => write!(f, "show"),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Store)]
pub enum ConnectState {
    #[default]
    DisConnect,
    Connecting,
    Connected,
}

impl Notify for ConnectState {}

#[derive(Default, Debug, Clone, PartialEq, Store)]
pub struct RelatedMsgState {
    pub nickname: Option<AttrValue>,
    pub msg: Option<Message>,
}

impl RelatedMsgState {
    pub fn notify(nickname: AttrValue, msg: Message) {
        Dispatch::<Self>::global().reduce_mut(|s| {
            s.nickname = Some(nickname);
            s.msg = Some(msg);
        });
    }
}
