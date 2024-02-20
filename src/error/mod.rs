use gloo::console;
use thiserror::Error;
use yew::html::RenderError;
use yew::suspense::Suspension;

#[derive(Error, Debug)]
pub enum MyError {
    #[error("未知错误")]
    Unknown,
    #[error("HtmlElement转换无效:{0}")]
    ElementCastError(String),
    // #[error("RequestError:{0}")]
    // RequestError(#[from] gloo_net::Error),
    // 如果有其它错误，在这里添加转换
}

impl From<MyError> for RenderError {
    fn from(val: MyError) -> Self {
        console::error!(format!("!!错误!!:{}", val));
        RenderError::Suspended(Suspension::new().0)
    }
}
