use js_sys::{Error as JsError, JsString};
use thiserror::Error as ThisError;
use wasm_bindgen::{JsCast, JsValue};

pub type Reason = String;

#[derive(Debug, ThisError)]
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
    Network(Reason),
    /// js related error
    #[error("{0}")]
    JavaScript(Reason),
    #[error("Unknown error")]
    Unknown,
    #[error("No window object")]
    NoWindow,
    #[error("JsValue to string error")]
    JsToStr,
}

impl From<gloo_net::Error> for Error {
    fn from(err: gloo_net::Error) -> Self {
        Self::Network(err.to_string())
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
