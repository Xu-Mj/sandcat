use crate::error::Error;

pub mod api;
pub mod db;
pub mod error;
pub mod model;
pub mod pb;
pub mod state;

pub type Result<T> = std::result::Result<T, Error>;
