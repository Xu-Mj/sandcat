pub mod api_err;

use js_sys::{Error as JsError, JsString};
use thiserror::Error as ThisError;
use wasm_bindgen::{JsCast, JsValue};

pub type Result<T> = std::result::Result<T, Error>;

type Reason = String;

#[derive(Debug, Clone, ThisError, PartialEq)]
pub enum Error {
    /// convert server message to local error
    #[error("Convert message error {0}")]
    Convert(Reason),
    /// database query not found
    #[error("Not found{0}")]
    NotFound(Reason),
    /// database error
    #[error("Database error {0}")]
    Database(Reason),
    /// request server error
    #[error("Network error {0}")]
    Network(api_err::Error),
    #[error("Network connect error {0}")]
    NetworkConn(Reason),
    /// js related error
    #[error("{0}")]
    JavaScript(Reason),
    #[error("Unknown error")]
    Unknown,
    #[error("No window object")]
    NoWindow,
    #[error("JsValue to string error")]
    JsToStr,
    #[error("JsValue to string error")]
    WebSocket(WebSocketError),
    #[error("BinCode error {0}")]
    BinCode(Reason),
}

#[derive(Debug, Clone, PartialEq)]
pub enum WebSocketError {
    Closed,
    Connect(JsValue),
    Send(JsValue),
}

impl From<bincode::Error> for Error {
    fn from(value: bincode::Error) -> Self {
        Self::BinCode(value.to_string())
    }
}

impl From<serde_wasm_bindgen::Error> for Error {
    fn from(value: serde_wasm_bindgen::Error) -> Self {
        Self::Convert(value.to_string())
    }
}

impl From<gloo_net::Error> for Error {
    fn from(err: gloo_net::Error) -> Self {
        Self::NetworkConn(err.to_string())
    }
}

impl From<JsValue> for Error {
    fn from(err: JsValue) -> Self {
        match get_js_error_msg(err) {
            None => Self::Unknown,
            Some(reason) => Self::JavaScript(reason),
        }
    }
}

fn get_js_error_msg(err: JsValue) -> Option<String> {
    if JsError::instanceof(&err) {
        let e: JsError = err.into();
        Some(e.message().into())
    } else if JsString::instanceof(&err) {
        let e: JsString = err.into();
        Some(e.into())
    } else {
        None
    }
}
