use yew::{AttrValue, Callback};

#[derive(Default, Clone, PartialEq)]
pub struct NotificationState {
    pub notify: Callback<Notification>,
}

#[derive(Default, Debug, Clone)]
pub struct Notification {
    pub type_: NotificationType,
    pub title: AttrValue,
    pub content: AttrValue,
}

impl Notification {
    pub fn from_content(content: AttrValue) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
    pub fn error_from_content(content: AttrValue) -> Self {
        Self {
            content,
            type_: NotificationType::Error,
            ..Default::default()
        }
    }
}

#[derive(Default, Clone, Debug, PartialEq)]
pub enum NotificationType {
    #[default]
    Info,
    Success,
    Warn,
    Error,
}
