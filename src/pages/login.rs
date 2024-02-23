#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use gloo::utils::window;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::scope_ext::RouterScopeExt;

use crate::components::right::sender::SenderMsg;
use crate::db::friend_ship::FriendShipRepo;
use crate::db::{TOKEN, WS_ADDR};
use crate::model::user::User;
use crate::{
    api, db,
    db::{friend::FriendRepo, user::UserRepo, DB_NAME},
};

use super::Page;

pub struct Login {
    account_ref: NodeRef,
    pwd_ref: NodeRef,
    login_state: LoginState,
    show_error: bool,
}

pub enum LoginMsg {
    Login,
    Success(AttrValue),
    Failed,
    OnEnterKeyDown(SubmitEvent),
}

pub enum LoginState {
    Logining,
    Success(i32),
    Failed,
    Nothing,
}

async fn login_simulate(account: String, password: String) -> Result<Response, JsValue> {
    let body = serde_json::to_string(&LoginRequest { account, password }).unwrap();
    let res = reqwasm::http::Request::post("/api/user/login")
        .header("Content-Type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|err| {
            log::error!("send request error: {:?}", err);
            JsValue::from(err.to_string())
        })?
        .json()
        .await
        .map_err(|err| {
            log::error!("send request error: {:?}", err);
            JsValue::from(err.to_string())
        })?;
    Ok(res)
}

#[derive(Deserialize)]
pub struct Response {
    id: String,
    user: User,
    token: String,
    ws_addr: String,
}

#[derive(Serialize)]
pub struct LoginRequest {
    pub account: String,
    pub password: String,
}

// 模拟输入写入数据库
async fn init_db(id: AttrValue, token: &str) {
    // 拉取联系人
    // 查询是否需要更新联系人
    match api::user::get_friend_list_by_id(id.to_string()).await {
        Ok(res) => {
            // 写入数据库
            FriendRepo::new().await.put_friend_list(&res).await;
        }
        Err(e) => {
            log::error!("获取联系人列表错误: {:?}", e)
        }
    }

    // // 拉取好友请求列表
    // match api::user::get_friend_apply_list_by_id(id.to_string()).await {
    //     Ok(res) => {
    //         // 写入数据库
    //         FriendShipRepo::new().await.put_friendships(&res).await;
    //         log::debug!("get friend ship list: {:?}", &res)
    //     }
    //     Err(e) => {
    //         log::error!("获取好友请求列表错误: {:?}", e);
    //     }
    // }
}

impl Component for Login {
    type Message = LoginMsg;

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            account_ref: NodeRef::default(),
            pwd_ref: NodeRef::default(),
            login_state: LoginState::Nothing,
            show_error: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            LoginMsg::Login => {
                // 这里可以写登录逻辑
                // 假设登录成功
                // ctx.link().send_message(LoginState::Success);
                // 使用ref获取用户名和密码
                let account = self.account_ref.cast::<HtmlInputElement>().unwrap().value();
                let pwd = self.pwd_ref.cast::<HtmlInputElement>().unwrap().value();
                ctx.link().send_future(async move {
                    let res = login_simulate(account, pwd).await;
                    if res.is_err() {
                        return LoginMsg::Failed;
                    }
                    let res = res.unwrap();
                    let user = res.user;
                    // user.login = true;

                    let id = user.id.clone();
                    // 初始化数据库名称,
                    // 这里一定要将所有权传过去，否则会提示将当前域变量转移的问题，因为当前函数结束会将该变量销毁
                    DB_NAME.get_or_init(|| format!("im-{}", id.clone()));
                    // 将用户id写入本地文件
                    //登录成功，初始化数据库

                    window()
                        .local_storage()
                        .unwrap()
                        .unwrap()
                        .set(WS_ADDR, res.ws_addr.as_str())
                        .unwrap();
                    window()
                        .local_storage()
                        .unwrap()
                        .unwrap()
                        .set(TOKEN, res.token.as_str())
                        .unwrap();
                    // 初始化数据库
                    init_db(id.clone(), &res.token).await;
                    // 将用户信息存入数据库
                    // 先查询是否登录过
                    let user_repo = UserRepo::new().await;
                    let user_former = user_repo.get(id.clone()).await;
                    user_repo.add(&user).await;
                    // if user_former.is_ok() && user_former.unwrap().id != AttrValue::default() {
                    //     // 已经存在，更新数据库
                    // } else {
                    //     user_repo.add(&user).await;
                    // }

                    LoginMsg::Success(id)
                });
                self.login_state = LoginState::Logining;
                true
            }
            LoginMsg::Success(id) => {
                // 路由跳转
                ctx.link().navigator().unwrap().push(&Page::Home { id });
                true
            }
            LoginMsg::Failed => {
                self.show_error = true;
                true
            }
            LoginMsg::OnEnterKeyDown(event) => {
                event.prevent_default();
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut info = html!();
        if self.show_error {
            info = html!(
                <div class="error">
                    {"用户名或密码错误"}
                </div>)
        }
        html! {
            <div class="login-container">
                {info}
                <form class="login-wrapper" onsubmit={ctx.link().callback(LoginMsg::OnEnterKeyDown)}>
                    <div class="sign">
                        {"登录"}
                    </div>

                    <div class="email">
                        <input type="text" ref={self.account_ref.clone()} required={true} autocomplete="current-password"  placeholder="e-mail"/>
                    </div>
                    <div class="pwd">
                        <input type="password" ref={self.pwd_ref.clone()} required={true} autocomplete="current-password"   placeholder="密码"/>
                    </div>
                        <input type="submit" class="submit" onclick={ctx.link().callback(|_| LoginMsg::Login)} value={"登录"}/>
                </form>
                <div class="login-register">
                    {"还没有账号？"}
                    <a href="/register">{"去注册"}</a>
                </div>
            </div>
        }
    }
}
