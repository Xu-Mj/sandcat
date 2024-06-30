use std::rc::Rc;

use base64::prelude::BASE64_STANDARD_NO_PAD;
use base64::Engine;
use components::dialog::Dialog;
use fluent::{FluentBundle, FluentResource};
use gloo::utils::window;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::scope_ext::RouterScopeExt;
use yewdux::Dispatch;

use components::constant::ERROR;
use i18n::{en_us, zh_cn, LanguageType};
use icons::{GitHubIcon, MoonIcon, SunIcon};
use sandcat_sdk::api;
use sandcat_sdk::db::{self, DB_NAME};
use sandcat_sdk::model::notification::Notification;
use sandcat_sdk::model::page::{Page, ThirdLoginType};
use sandcat_sdk::model::user::LoginRequest;
use sandcat_sdk::model::{REFRESH_TOKEN, TOKEN, WS_ADDR};
use sandcat_sdk::state::{I18nState, Notify, ThemeState};
use utils::tr;

pub struct Login {
    account_ref: NodeRef,
    pwd_ref: NodeRef,
    login_state: LoginState,
    show_error: bool,
    i18n: FluentBundle<FluentResource>,
    lang: LanguageType,
    theme: Dispatch<ThemeState>,
}

pub enum LoginMsg {
    Login,
    ThirdLogin(ThirdLoginType),
    Success(AttrValue),
    Failed,
    OnEnterKeyDown(SubmitEvent),
    SwitchLanguage(Event),
    SwitchTheme(Rc<ThemeState>),
}

pub enum LoginState {
    Logining,
    Nothing,
}

impl Component for Login {
    type Message = LoginMsg;

    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        // load theme
        let theme =
            Dispatch::<ThemeState>::global().subscribe(ctx.link().callback(LoginMsg::SwitchTheme));
        // load the i18n bundle
        let lang = I18nState::get().lang;
        let res = match lang {
            LanguageType::ZhCN => zh_cn::LOGIN,
            LanguageType::EnUS => en_us::LOGIN,
        };
        let i18n = utils::create_bundle(res);

        Self {
            account_ref: NodeRef::default(),
            pwd_ref: NodeRef::default(),
            login_state: LoginState::Nothing,
            show_error: false,
            i18n,
            lang,
            theme,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            LoginMsg::Login => {
                Dialog::loading("login..");
                // use ref to get the account and password
                let account = self.account_ref.cast::<HtmlInputElement>().unwrap().value();
                let pwd = self.pwd_ref.cast::<HtmlInputElement>().unwrap().value();

                ctx.link().send_future(async move {
                    let password = BASE64_STANDARD_NO_PAD.encode(pwd);
                    let res = match api::users()
                        .sign_in(LoginRequest { account, password })
                        .await
                    {
                        Ok(resp) => resp,
                        Err(err) => {
                            Notification::error(err).notify();
                            return LoginMsg::Failed;
                        }
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

                    LoginMsg::Success(id)
                });
                self.login_state = LoginState::Logining;
                true
            }
            LoginMsg::Success(id) => {
                ctx.link().navigator().unwrap().push(&Page::Home { id });
                true
            }
            LoginMsg::Failed => {
                Dialog::close_loading();
                Dialog::error("login failed, please check your account and password");
                self.show_error = true;
                true
            }
            LoginMsg::OnEnterKeyDown(event) => {
                event.prevent_default();
                false
            }
            LoginMsg::SwitchLanguage(event) => {
                log::debug!("switch language: {:?}", event);
                let input = event
                    .target()
                    .unwrap()
                    .dyn_into::<HtmlInputElement>()
                    .unwrap();
                let value = input.value();
                if value == LanguageType::ZhCN.to_string() {
                    self.i18n = utils::create_bundle(zh_cn::LOGIN);
                    // save language type with yewdux
                    Dispatch::<I18nState>::global().reduce_mut(|s| s.lang = LanguageType::ZhCN);
                    self.lang = LanguageType::ZhCN;
                } else if value == LanguageType::EnUS.to_string() {
                    self.i18n = utils::create_bundle(en_us::LOGIN);
                    Dispatch::<I18nState>::global().reduce_mut(|s| s.lang = LanguageType::EnUS);
                    self.lang = LanguageType::EnUS;
                }
                true
            }
            LoginMsg::SwitchTheme(state) => {
                utils::set_theme(&state.to_string());
                true
            }
            LoginMsg::ThirdLogin(tp) => {
                let url = match tp {
                    ThirdLoginType::GitHub => "http://127.0.0.1:50001/user/auth/github",
                    ThirdLoginType::Google => "http://127.0.0.1:50001/user/auth/wechat",
                };
                window().location().set_href(url).unwrap();
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut info = html!();
        if self.show_error {
            info = html!(
                <div class="error">
                    {tr!(self.i18n, ERROR )}
                </div>)
        }

        // i18n
        let login_title = tr!(self.i18n, "login_text");
        let email = tr!(self.i18n, "email");
        let onchange = ctx.link().callback(LoginMsg::SwitchLanguage);

        let (theme_icon, class) = if *self.theme.get() == ThemeState::Dark {
            (html!(<SunIcon/>), "sun")
        } else {
            (html!(<MoonIcon/>), "moon")
        };

        let theme = html! {
            <span {class} onclick={self.theme.reduce_mut_callback(|s| {if *s == ThemeState::Dark{
                *s = ThemeState::Light;
            } else {
                *s = ThemeState::Dark;
            }})}>
                {theme_icon}
            </span>
        };
        html! {
            <div class="login-container">
                {info}
                <form class="login-wrapper" onsubmit={ctx.link().callback(LoginMsg::OnEnterKeyDown)}>
                    {theme}
                    <div class="sign">
                        {login_title.clone()}
                    </div>

                    <div class="email">
                        <input type="text" ref={self.account_ref.clone()} required={true} autocomplete="current-password"  placeholder={email}/>
                    </div>
                    <div class="pwd">
                        <input type="password" ref={self.pwd_ref.clone()} required={true} autocomplete="current-password"   placeholder={tr!(self.i18n, "password")}/>
                    </div>
                    <div class="language">
                        <label for="en_us">
                            <input type="radio" name="language" id="en_us" value="en_us" onchange={onchange.clone()} checked={self.lang==LanguageType::EnUS}/>{"\tENG"}
                        </label>
                        <label for="zh_cn">
                            <input type="radio" name="language" id="zh_cn" value="zh_cn" {onchange} checked={self.lang==LanguageType::ZhCN}/>{"\t中文"}
                        </label>
                    </div>
                    <input type="submit" class="submit" onclick={ctx.link().callback(|_| LoginMsg::Login)} value={login_title.clone()}/>
                    <div class="third-login">
                        <span onclick={ctx.link().callback(|_| LoginMsg::ThirdLogin(ThirdLoginType::GitHub))}><GitHubIcon /></span>
                        // <span onclick={ctx.link().callback(|_| LoginMsg::ThirdLogin(ThirdLoginType::Google))}><GMailIcon /></span>
                    </div>
                    <div class="login-register">
                        {tr!(self.i18n, "to_register_prefix")}
                        <a href="/register">{tr!(self.i18n, "to_register")}</a>
                    </div>
                </form>
            </div>
        }
    }
}
