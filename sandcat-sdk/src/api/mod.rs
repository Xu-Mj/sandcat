use crate::model::TOKEN;

use self::{
    file::FileApi,
    friend::FriendApi,
    group::GroupApi,
    http::OAuth2Http,
    http::{FileHttp, FriendHttp, GroupHttp, MsgHttp, SeqHttp, UserHttp},
    message::MsgApi,
    oauth2::OAuth2Api,
    seq::SeqApi,
    user::UserApi,
};

mod file;
mod friend;
mod group;
mod http;
mod message;
mod oauth2;
mod seq;
mod user;

pub const AUTHORIZE_HEADER: &str = "Authorization";

pub fn token() -> String {
    let token = utils::get_local_storage(TOKEN).unwrap();
    format!("Bearer {}", token)
}

pub fn users() -> Box<dyn UserApi> {
    Box::new(UserHttp)
}

pub fn oauth2() -> Box<dyn OAuth2Api> {
    Box::new(OAuth2Http)
}

pub fn groups() -> Box<dyn GroupApi> {
    Box::new(GroupHttp)
}

pub fn friends() -> Box<dyn FriendApi> {
    Box::new(FriendHttp)
}

pub fn messages() -> Box<dyn MsgApi> {
    Box::new(MsgHttp)
}

pub fn seq() -> Box<dyn SeqApi> {
    Box::new(SeqHttp)
}

pub fn file() -> Box<dyn FileApi> {
    Box::new(FileHttp)
}
