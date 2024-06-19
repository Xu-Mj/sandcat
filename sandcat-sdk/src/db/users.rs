use std::fmt::Debug;

use crate::{error::Error, model::user::User};

#[async_trait::async_trait(?Send)]
pub trait Users: Debug {
    async fn add(&self, user: &User);
    async fn get(&self, id: &str) -> Result<User, Error>;
}
