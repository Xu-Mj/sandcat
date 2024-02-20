// 请求模块
#![allow(dead_code)]
#![allow(unused_variables)]

pub(crate) mod user;

use crate::db::TOKEN;
use gloo::utils::window;
use std::sync::OnceLock;

pub const AUTHORIZE_HEADER: &str = "Authorization";

pub const TOKEN_VALUE: OnceLock<String> = OnceLock::new();

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
