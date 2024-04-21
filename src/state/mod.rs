use yewdux::Store;

use crate::{
    i18n::LanguageType,
    model::{user::User, ComponentType},
};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct OfflineMsgState {
    pub complete: Option<()>,
}

#[derive(Debug, Default, Clone, PartialEq, Store)]
pub struct I18nState {
    pub lang: LanguageType,
}
#[derive(Default, Clone, PartialEq, Store)]
pub struct AppState {
    pub component_type: ComponentType,
    pub login_user: User,
}

#[derive(Store, Default, Debug, Clone, PartialEq)]
pub struct UnreadState {
    pub msg_count: usize,
    pub contacts_count: usize,
}
