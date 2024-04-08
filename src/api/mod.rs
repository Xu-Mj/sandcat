use gloo::utils::window;
use std::sync::OnceLock;

use crate::db::TOKEN;

use self::{
    friend::FriendApi,
    group::GroupApi,
    http::{FriendHttp, GroupHttp, MsgHttp, UserHttp},
    message::MsgApi,
    user::UserApi,
};

pub mod friend;
pub mod group;
pub mod http;
pub mod message;
pub mod user;

pub const AUTHORIZE_HEADER: &str = "Authorization";
pub static TOKEN_VALUE: OnceLock<String> = OnceLock::new();

pub fn token() -> String {
    let token = TOKEN_VALUE
        .get_or_init(|| {
            window()
                .local_storage()
                .unwrap()
                .unwrap()
                .get(TOKEN)
                .unwrap()
                .unwrap()
        })
        .to_string();
    format!("Bearer {}", token)
}

pub fn users() -> Box<dyn UserApi> {
    Box::new(UserHttp::new(token, AUTHORIZE_HEADER.to_string()))
}

pub fn groups() -> Box<dyn GroupApi> {
    Box::new(GroupHttp::new(token(), AUTHORIZE_HEADER.to_string()))
}

pub fn friends() -> Box<dyn FriendApi> {
    Box::new(FriendHttp::new(token(), AUTHORIZE_HEADER.to_string()))
}

pub fn messages() -> Box<dyn MsgApi> {
    Box::new(MsgHttp::new(token(), AUTHORIZE_HEADER.to_string()))
}
