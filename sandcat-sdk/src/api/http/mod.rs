use gloo_net::http::Response;

pub use file::*;
pub use friend::*;
pub use group::*;
pub use msg::*;
pub use seq::*;
pub use user::*;

use crate::error::Error;

mod file;
mod friend;
mod group;
mod msg;
mod seq;
mod user;

pub trait RespStatus: Sized {
    fn success(self) -> Result<Self, Error>;
}

impl RespStatus for Response {
    fn success(self) -> Result<Self, Error> {
        if (200..=299).contains(&self.status()) {
            Ok(self)
        } else {
            Err(Error::Network(format!(
                "Server responded with error: {}",
                self.status()
            )))
        }
    }
}
