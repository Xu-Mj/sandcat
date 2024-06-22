use yew::AttrValue;
use yewdux::Store;

#[derive(Default, Debug, Clone, PartialEq, Store)]
pub struct Notification {
    pub id: i64,
    pub content: AttrValue,
    pub delay: u32,
    pub type_: NotificationType,
}

impl Notification {
    pub fn info(content: impl ToString) -> Self {
        let id = chrono::Utc::now().timestamp_millis();
        Self {
            id,
            content: content.to_string().into(),
            type_: NotificationType::Info,
            delay: 3000,
        }
    }

    pub fn warn(content: impl ToString) -> Self {
        let id = chrono::Utc::now().timestamp_millis();
        Self {
            id,
            content: content.to_string().into(),
            type_: NotificationType::Warn,
            delay: 3000,
        }
    }

    pub fn error(content: impl ToString) -> Self {
        let id = chrono::Utc::now().timestamp_millis();
        Self {
            id,
            content: content.to_string().into(),
            type_: NotificationType::Error,
            delay: 3000,
        }
    }

    pub fn with_delay(mut self, delay: u32) -> Self {
        self.delay = delay;
        self
    }
}

#[derive(Default, Clone, Debug, PartialEq)]
pub enum NotificationType {
    #[default]
    Info,
    // Success,
    Warn,
    Error,
}
