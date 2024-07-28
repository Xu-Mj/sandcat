use async_trait::async_trait;
use gloo_net::http::Response;

pub use file::*;
pub use friend::*;
pub use group::*;
pub use msg::*;
pub use oauth2::*;
pub use seq::*;
pub use user::*;

use crate::error::{Error, Result};

mod file;
mod friend;
mod group;
mod msg;
mod oauth2;
mod seq;
mod user;

#[async_trait(?Send)]
pub trait RespStatus: Sized {
    async fn success(self) -> Result<Self>;
}

#[async_trait(?Send)]
impl RespStatus for Response {
    async fn success(self) -> Result<Self> {
        if (200..=299).contains(&self.status()) {
            Ok(self)
        } else {
            // deserialize error
            let err = self
                .json::<Error>()
                .await
                .unwrap_or(Error::internal_with_details("Response deserialize error"));
            Err(err)
        }
    }
}
