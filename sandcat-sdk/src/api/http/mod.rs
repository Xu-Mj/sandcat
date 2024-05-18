mod file;
mod friend;
mod group;
mod msg;
mod seq;
mod user;

pub use file::*;
pub use friend::*;
pub use group::*;
pub use msg::*;
pub use seq::*;
pub use user::*;

use gloo_net::http::Response;
use wasm_bindgen::JsValue;

pub trait RespStatus: Sized {
    fn success(self) -> Result<Self, JsValue>;
}

impl RespStatus for Response {
    fn success(self) -> Result<Self, JsValue> {
        if (200..=299).contains(&self.status()) {
            Ok(self)
        } else {
            Err(JsValue::from_str(&format!(
                "Server responded with error: {}",
                self.status()
            )))
        }
    }
}
