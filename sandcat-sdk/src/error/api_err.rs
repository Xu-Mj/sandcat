use std::fmt::{Display, Formatter, Result};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorKind {
    UnknownError,
    DbError,
    ConfigReadError,
    ConfigParseError,
    NotFound,
    BroadCastError,
    InternalServer,
    BodyParsing,
    PathParsing,
    UnAuthorized,
    ParseError,
    TonicError,
    MongoDbValueAccessError,
    MongoDbBsonSerError,
    MongoDbOperateError,
    RedisError,
    IOError,
    ReqwestError,
    ServiceNotFound,
    InvalidRegisterCode,
    BadRequest,
    AccountOrPassword,
    OSSError,
    CodeIsExpired,
    CodeIsInvalid,
    BinCode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Error {
    kind: ErrorKind,
    details: Option<String>,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match &self.details {
            Some(details) => write!(f, "{:?}: {}", self.kind, details),
            None => write!(f, "{:?}", self.kind),
        }
    }
}

impl Error {
    pub fn unkonw_error() -> Self {
        Error {
            kind: ErrorKind::UnknownError,
            details: None,
        }
    }
}
