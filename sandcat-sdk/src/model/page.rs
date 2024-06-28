use std::{fmt::Display, str::FromStr};

use yew::AttrValue;
use yew_router::Routable;

// 定义路由
#[derive(Clone, PartialEq, Routable)]
pub enum Page {
    #[at("/:id")]
    Home { id: AttrValue },
    #[at("/login")]
    Login,
    #[at("/register")]
    Register,
    #[at("/")]
    Redirect,
    #[at("/third_login_callback/:tp")]
    ThirdLoginCallback { tp: ThirdLoginType },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ThirdLoginType {
    GitHub,
    Google,
}

impl Display for ThirdLoginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThirdLoginType::GitHub => write!(f, "github"),
            ThirdLoginType::Google => write!(f, "google"),
        }
    }
}

impl FromStr for ThirdLoginType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "github" => Ok(ThirdLoginType::GitHub),
            "google" => Ok(ThirdLoginType::Google),
            _ => Err(format!("Invalid third login type: {}", s)),
        }
    }
}
