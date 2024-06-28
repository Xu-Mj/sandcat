use web_sys::UrlSearchParams;
use yew::{html, AttrValue, Component, Html, Properties};

use sandcat_sdk::{
    api,
    db::{self, DB_NAME},
    model::{
        page::{Page, ThirdLoginType},
        REFRESH_TOKEN, TOKEN, WS_ADDR,
    },
};
use yew_router::scope_ext::RouterScopeExt;

#[derive(Debug, Properties, Clone, PartialEq)]
pub struct Props {
    pub tp: ThirdLoginType,
}

pub struct OAuth2 {
    err_msg: AttrValue,
}

pub enum Msg {
    Success(AttrValue),
    Failed(AttrValue),
}

impl Component for OAuth2 {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &yew::Context<Self>) -> Self {
        let location = gloo::utils::window().location();
        let search = location.search().unwrap_or_default();
        let search_params = UrlSearchParams::new_with_str(&search).unwrap();

        let code = search_params.get("code").unwrap_or_default();
        let state = search_params.get("state").unwrap_or_default();
        let tp = ctx.props().tp.clone();
        // send code to server
        ctx.link().send_future(async move {
            let resp = match tp {
                ThirdLoginType::GitHub => api::oauth2().github(&code, &state).await,
                ThirdLoginType::WeChat => api::oauth2().wechat(&code, &state).await,
            };
            let res = match resp {
                Ok(resp) => resp,
                Err(err) => return Msg::Failed(err.to_string().into()),
            };
            let user = res.user;
            // user.login = true;

            let id = user.id.clone();
            // 初始化数据库名称,
            // 这里一定要将所有权传过去，否则会提示将当前域变量转移的问题，因为当前函数结束会将该变量销毁
            DB_NAME.get_or_init(|| format!("im-{}", id.clone()));
            // 将用户id写入本地文件
            //登录成功，初始化数据库

            utils::set_local_storage(WS_ADDR, &res.ws_addr).unwrap();
            utils::set_local_storage(TOKEN, &res.token).unwrap();
            utils::set_local_storage(REFRESH_TOKEN, &res.refresh_token).unwrap();

            // 初始化数据库
            db::init_db().await;
            // 将用户信息存入数据库
            // 先查询是否登录过
            // let user_former = user_repo.get(id.clone()).await;
            db::db_ins().users.add(&user).await;
            Msg::Success(id)
        });
        Self {
            err_msg: AttrValue::default(),
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Success(id) => {
                ctx.link().navigator().unwrap().push(&Page::Home { id });
                false
            }
            Msg::Failed(err) => {
                self.err_msg = err;
                true
            }
        }
    }

    fn view(&self, _ctx: &yew::Context<Self>) -> Html {
        let content = if self.err_msg.is_empty() {
            html!(<p>{ "正在登录..." }</p>)
        } else {
            html!(
                <div>
                    <p>{ "登录失败" }</p>
                    <p>{ self.err_msg.clone() }</p>
                </div>
            )
        };
        html! {
            <div class="oauth2-logining">
                { content }
            </div>
        }
    }
}
