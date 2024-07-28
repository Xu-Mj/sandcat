use std::fmt::{self, Display, Formatter};

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum ErrorKind {
    UnknownError,
    NotFound,
    InternalServer,
    BodyParsing,
    PathParsing,
    UnAuthorized,
    ParseError,
    ServiceNotFound,
    InvalidRegisterCode,
    BadRequest,
    AccountOrPassword,
    CodeIsExpired,
    CodeIsInvalid,
    // The ERROR ON THE SERVER SIDE, no need to catch
    // DbError,
    // ConfigReadError,
    // ConfigParseError,
    // BroadCastError,
    // TonicError,
    // MongoDbValueAccessError,
    // MongoDbBsonSerError,
    // MongoDbOperateError,
    // RedisError,
    // IOError,
    // ReqwestError,
    // OSSError,
    // BinCode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Error {
    kind: ErrorKind,
    details: Option<String>,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

impl<'de> Deserialize<'de> for ErrorKind {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ErrorKindVisitor;

        impl<'de> Visitor<'de> for ErrorKindVisitor {
            type Value = ErrorKind;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid error kind")
            }

            fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    "UnknownError" => Ok(ErrorKind::UnknownError),
                    "NotFound" => Ok(ErrorKind::NotFound),
                    "BodyParsing" => Ok(ErrorKind::BodyParsing),
                    "PathParsing" => Ok(ErrorKind::PathParsing),
                    "UnAuthorized" => Ok(ErrorKind::UnAuthorized),
                    "ParseError" => Ok(ErrorKind::ParseError),
                    "ServiceNotFound" => Ok(ErrorKind::ServiceNotFound),
                    "InvalidRegisterCode" => Ok(ErrorKind::InvalidRegisterCode),
                    "BadRequest" => Ok(ErrorKind::BadRequest),
                    "AccountOrPassword" => Ok(ErrorKind::AccountOrPassword),
                    "CodeIsExpired" => Ok(ErrorKind::CodeIsExpired),
                    "CodeIsInvalid" => Ok(ErrorKind::CodeIsInvalid),
                    "OSSError"
                    | "BinCode"
                    | "DbError"
                    | "ConfigReadError"
                    | "ConfigParseError"
                    | "BroadCastError"
                    | "TonicError"
                    | "MongoDbValueAccessError"
                    | "MongoDbBsonSerError"
                    | "MongoDbOperateError"
                    | "RedisError"
                    | "IOError"
                    | "ReqwestError"
                    | "InternalServer" => Ok(ErrorKind::InternalServer),
                    _ => Ok(ErrorKind::UnknownError),
                }
            }
        }

        deserializer.deserialize_str(ErrorKindVisitor)
    }
}
