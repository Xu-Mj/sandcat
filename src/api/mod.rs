pub(crate) mod user;

use crate::db::TOKEN;
use gloo::utils::window;
use std::sync::OnceLock;

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
