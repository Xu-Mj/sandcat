use yew::AttrValue;
use yewdux::{Dispatch, Store};

use crate::error::Error;

#[derive(Default, Debug, Clone, PartialEq, Store)]
pub struct Notification {
    pub id: i64,
    pub content: AttrValue,
    pub delay: u32,
    pub type_: NotificationType,
    pub error: Option<Error>,
}

impl Notification {
    pub fn info(content: impl ToString) -> Self {
        let id = chrono::Utc::now().timestamp_millis();
        Self {
            id,
            content: content.to_string().into(),
            type_: NotificationType::Info,
            delay: 3000,
            error: None,
        }
    }

    pub fn warn(content: impl ToString) -> Self {
        let id = chrono::Utc::now().timestamp_millis();
        Self {
            id,
            content: content.to_string().into(),
            type_: NotificationType::Warn,
            delay: 3000,
            error: None,
        }
    }

    pub fn error(err: Error) -> Self {
        let id = chrono::Utc::now().timestamp_millis();
        Self {
            id,
            content: AttrValue::default(),
            type_: NotificationType::Error,
            delay: 5000,
            error: Some(err),
        }
    }

    pub fn notify(self) {
        Dispatch::<Notification>::global().set(self);
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
