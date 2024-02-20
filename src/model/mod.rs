#![allow(dead_code)]

pub mod conversation;
pub mod friend;
pub mod message;
pub mod notification;
pub mod user;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    #[default]
    Default,
    Text,
    Image,
    Video,
    Audio,
    File,
    Emoji,
    VideoCall,
    AudioCall,
}

pub enum RightContentType {}
