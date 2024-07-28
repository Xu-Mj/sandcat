use std::{error::Error as StdError, fmt, ops::Deref};

use js_sys::{Error as JsError, JsString};
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize,
};
// use thiserror::Error as ThisError;
use wasm_bindgen::{JsCast, JsValue};

pub type Result<T> = std::result::Result<T, Error>;

// #[derive(Debug, Clone, ThisError, PartialEq)]
// pub enum Error {
//     /// convert server message to local error
//     #[error("Convert message error {0}")]
//     Convert(Reason),
//     /// database query not found
//     #[error("Not found{0}")]
//     NotFound(Reason),
//     /// database error
//     #[error("Database error {0}")]
//     Database(Reason),
//     /// request server error
//     #[error("Network error {0}")]
//     Network(api_err::Error),
//     #[error("Network connect error {0}")]
//     NetworkConn(Reason),
//     /// js related error
//     #[error("{0}")]
//     JavaScript(Reason),
//     #[error("Unknown error")]
//     Unknown,
//     #[error("No window object")]
//     NoWindow,
//     #[error("JsValue to string error")]
//     JsToStr,
//     #[error("JsValue to string error")]
//     WebSocket(WebSocketError),
//     #[error("BinCode error {0}")]
//     BinCode(Reason),
// }

// The ERROR ON THE SERVER SIDE, no need to catch
// is ServerError
// ParseError,
// ServiceNotFound,
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

// BadRequest
// BodyParsing,
// PathParsing,
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum ErrorKind {
    UnknownError,
    Network,
    LocalNotFound,
    NotFound,
    ServerError,
    Internal,
    UnAuthorized,
    BadRequest,
    AccountOrPassword,
    CodeIsExpired,
    CodeIsInvalid,
    MsgSendError,
    WsConnError,
    WsClosed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.details {
            Some(details) => write!(f, "{:?}: {}", self.kind, details),
            None => write!(f, "{:?}", self.kind),
        }
    }
}
impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_deref()
            .map(|e| e as &(dyn StdError + 'static))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Error {
    kind: ErrorKind,
    details: Option<String>,
    #[serde(skip)]
    source: Option<Box<dyn StdError>>,
}

impl Clone for Error {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind.clone(),
            details: self.details.clone(),
            source: None,
        }
    }
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.details == other.details
    }
}

impl Error {
    #[inline]
    pub fn new(
        kind: ErrorKind,
        details: impl Into<String>,
        source: impl StdError + 'static,
    ) -> Self {
        Self {
            kind,
            source: Some(Box::new(source)),
            details: Some(details.into()),
        }
    }

    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    #[inline]
    pub fn with_kind(kind: ErrorKind) -> Self {
        Self {
            kind,
            source: None,
            details: None,
        }
    }

    #[inline]
    pub fn with_details(kind: ErrorKind, details: impl Into<String>) -> Self {
        Self {
            kind,
            source: None,
            details: Some(details.into()),
        }
    }

    pub fn js_error(err: JavaScriptError) -> Self {
        Self {
            kind: ErrorKind::Internal,
            details: Some(err.message().into()),
            source: Some(Box::new(err)),
        }
    }

    pub fn js_err(err: JsValue) -> Self {
        let err = JavaScriptError::from(err);
        Self {
            kind: ErrorKind::Internal,
            details: Some(err.message().into()),
            source: Some(Box::new(err)),
        }
    }

    pub fn send_err(err: JsValue) -> Self {
        let err = JavaScriptError::from(err);
        Self {
            kind: ErrorKind::MsgSendError,
            details: Some(err.message().into()),
            source: Some(Box::new(err)),
        }
    }

    pub fn ws_conn(err: JsValue) -> Self {
        let err = JavaScriptError::from(err);
        Self {
            kind: ErrorKind::WsConnError,
            details: Some(err.message().into()),
            source: Some(Box::new(err)),
        }
    }

    pub fn ws_closed() -> Self {
        Self::with_kind(ErrorKind::WsClosed)
    }

    pub fn internal(err: impl StdError + 'static) -> Self {
        Self {
            kind: ErrorKind::Internal,
            details: Some(err.to_string()),
            source: Some(Box::new(err)),
        }
    }

    pub fn internal_with_details(details: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::Internal,
            details: Some(details.into()),
            source: None,
        }
    }

    pub fn network(err: impl StdError + 'static) -> Self {
        Self {
            kind: ErrorKind::Internal,
            details: Some(err.to_string()),
            source: Some(Box::new(err)),
        }
    }
    pub fn local_not_found(details: impl Into<String>) -> Self {
        Self::with_details(ErrorKind::LocalNotFound, details)
    }

    pub fn unknown() -> Self {
        Self::with_kind(ErrorKind::UnknownError)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum WebSocketError {
    Closed,
    Connect(JsValue),
    Send(JsValue),
}

impl From<bincode::Error> for Error {
    fn from(value: bincode::Error) -> Self {
        Self::internal(value)
    }
}

impl From<serde_wasm_bindgen::Error> for Error {
    fn from(value: serde_wasm_bindgen::Error) -> Self {
        Self::internal(value)
    }
}

impl From<gloo_net::Error> for Error {
    fn from(err: gloo_net::Error) -> Self {
        Self::network(err)
    }
}

#[derive(Debug, Clone)]
pub struct JavaScriptError(JsError);

impl fmt::Display for JavaScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Deref for JavaScriptError {
    type Target = JsError;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl StdError for JavaScriptError {}

impl From<JsValue> for JavaScriptError {
    fn from(value: JsValue) -> Self {
        if JsError::instanceof(&value) {
            JavaScriptError(value.unchecked_into::<JsError>())
        } else {
            let msg = if JsString::instanceof(&value) {
                value.as_string().unwrap_or_default()
            } else {
                "Unknown JavaScript error".to_string()
            };
            JavaScriptError(JsError::new(&msg))
        }
    }
}

impl From<JsValue> for Error {
    fn from(err: JsValue) -> Self {
        if JsError::instanceof(&err) {
            let e: JsError = err.into();
            Self::js_error(JavaScriptError(e))
        } else if JsString::instanceof(&err) {
            let e: JsString = err.into();
            Self::with_details(ErrorKind::Internal, e)
        } else {
            Self::unknown()
        }
    }
}

/// deserialize error from server
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
                    "UnAuthorized" => Ok(ErrorKind::UnAuthorized),
                    "BodyParsing" | "PathParsing" | "BadRequest" => Ok(ErrorKind::BadRequest),
                    "AccountOrPassword" => Ok(ErrorKind::AccountOrPassword),
                    "CodeIsExpired" => Ok(ErrorKind::CodeIsExpired),
                    "CodeIsInvalid" => Ok(ErrorKind::CodeIsInvalid),
                    "Internal" => Ok(ErrorKind::Internal),
                    "ParseError"
                    | "ServiceNotFound"
                    | "OSSError"
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
                    | "InternalServer" => Ok(ErrorKind::ServerError),
                    _ => Ok(ErrorKind::UnknownError),
                }
            }
        }

        deserializer.deserialize_str(ErrorKindVisitor)
    }
}
