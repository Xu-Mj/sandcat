use fluent::{FluentBundle, FluentResource};
use gloo::timers::callback::Interval;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use zxcvbn::zxcvbn;

use i18n::{en_us, zh_cn, LanguageType};
use sandcat_sdk::{api, error::Error, model::notification::Notification};
use utils::tr;

use crate::constant::CANCEL;

pub struct ChangePwd {
    code_node: NodeRef,
    new_pwd: AttrValue,
    confirm_pwd: AttrValue,
    re_pwd_is_modify: bool,
    pwd_strength: u8,
    pwd_is_same: bool,
    is_code_send: bool,
    is_send: bool,
    time: i64,
    i18n: FluentBundle<FluentResource>,
    code_timer: Option<Interval>,
}

#[derive(Properties, Clone, PartialEq)]
pub struct Props {
    pub email: AttrValue,
    pub close: Callback<()>,
    pub lang: LanguageType,
}

pub enum Msg {
    OnPwdInput(InputEvent),
    OnRePwdInput(InputEvent),
    SendCode,
    SendCodeSuccess,
    SendCodeFailed(Error),
    UpdateTime,
    Submit(SubmitEvent),
}

impl Component for ChangePwd {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &yew::Context<Self>) -> Self {
        let res = match ctx.props().lang {
            LanguageType::ZhCN => zh_cn::CHANGE_PWD,
            LanguageType::EnUS => en_us::CHANGE_PWD,
        };
        let i18n = utils::create_bundle(res);

        Self {
            code_node: NodeRef::default(),
            new_pwd: AttrValue::default(),
            confirm_pwd: AttrValue::default(),
            is_send: false,
            is_code_send: false,
            re_pwd_is_modify: false,
            pwd_is_same: false,
            pwd_strength: 0,
            time: 0,
            code_timer: None,
            i18n,
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::OnPwdInput(event) => {
                if !self.new_pwd.is_empty() {
                    let estimate = zxcvbn(self.new_pwd.as_str(), &[]).unwrap();
                    self.pwd_strength = estimate.score() * 25;
                    log::debug!("strength: {}", estimate.score());
                } else {
                    self.pwd_strength = 0;
                }
                self.new_pwd = event
                    .target_dyn_into::<HtmlInputElement>()
                    .unwrap()
                    .value()
                    .into();
            }
            Msg::OnRePwdInput(event) => {
                self.confirm_pwd = event
                    .target_dyn_into::<HtmlInputElement>()
                    .unwrap()
                    .value()
                    .into();
                self.re_pwd_is_modify = true;
                self.pwd_is_same = self.confirm_pwd == self.new_pwd;
            }
            Msg::SendCode => {
                log::debug!("send code");
                self.is_send = true;
                // 获取邮件
                let email = ctx.props().email.to_string();

                ctx.link().send_future(async move {
                    // 发送邮件
                    match api::users().send_mail(email).await {
                        Ok(_) => Msg::SendCodeSuccess,
                        Err(e) => Msg::SendCodeFailed(e),
                    }
                });
            }
            Msg::Submit(event) => {
                event.prevent_default();
            }
            Msg::SendCodeSuccess => {
                log::debug!("send code success");
                // 初始化计时器
                let ctx = ctx.link().clone();
                self.code_timer = Some(Interval::new(1000, move || {
                    ctx.send_message(Msg::UpdateTime);
                }));
                self.time = 60;
                self.is_code_send = true;

                self.is_send = false;
            }
            Msg::SendCodeFailed(e) => {
                log::error!("send code failed: {:?}", e);
                Notification::error("code send failed").notify();
                self.is_send = false;
            }
            Msg::UpdateTime => {
                self.time -= 1;
                if self.time == 0 {
                    self.code_timer.take().unwrap().cancel();
                    self.code_timer = None;
                    self.is_code_send = false;
                }
            }
        }
        true
    }

    fn view(&self, ctx: &yew::Context<Self>) -> Html {
        let code_button = if self.is_send {
            format!("{}s {}", self.time, tr!(self.i18n, "re_send_code"))
        } else {
            tr!(self.i18n, "send_code")
        };

        let pwd_is_same = if self.re_pwd_is_modify {
            if self.pwd_is_same {
                html!(<span style="color: green;">{"√"}</span>)
            } else {
                html!(<span style="color: red;">{"×"}</span>)
            }
        } else {
            html!()
        };

        let pwd_strength = if self.pwd_strength > 0 {
            html! {
                <meter
                    max="100"
                    low="33"
                    high="66"
                    optimum="75"
                    value={self.pwd_strength.to_string()}>
                </meter>
            }
        } else {
            html!()
        };

        let onsubmit = ctx.link().callback(Msg::Submit);
        let on_cancel = ctx.props().close.reform(|_| ());

        html! {
            <form {onsubmit} class="change-pwd box-shadow">
                <div class="pwd">
                    <label for="pwd">
                        {tr!(self.i18n, "new_pwd")}
                    </label>
                    <input
                        type="password"
                        id="pwd"
                        required={true}
                        autocomplete="password"
                        placeholder={tr!(self.i18n, "pwd_hint")}
                        oninput={ctx.link().callback(Msg::OnPwdInput)}
                        minlength="6"
                        />
                    {pwd_strength}
                </div>
                <div class="re-pwd">
                    <label for="re-pwd">
                        {tr!(self.i18n, "confirm_pwd")}
                    </label>
                    <input
                        type="password"
                        id="re-pwd"
                        required={true}
                        autocomplete="current-password"
                        placeholder={tr!(self.i18n, "confirm_pwd_hint")}
                        minlength="6"
                        oninput={ctx.link().callback(Msg::OnRePwdInput)}
                        />
                    {pwd_is_same}
                </div>
                <div class="code">
                    <label for="code">
                        {tr!(self.i18n, "code")}
                    </label>
                    <input
                        ref={self.code_node.clone()}
                        type="text"
                        id="code"
                        required={true}
                        autocomplete="current-password"
                        placeholder={tr!(self.i18n, "code")}/>
                    <button
                        class="register-code-btn"
                        disabled={self.time != 0 && self.is_send}
                        onclick={ctx.link().callback(|_| Msg::SendCode)}
                        >
                        {code_button}
                    </button>
                </div>
                <p>
                    <input type="submit" class="register-button"  value={tr!(self.i18n, "submit")}/>
                    <input type="button" value={tr!(self.i18n, CANCEL)} onclick={on_cancel}/>
                </p>
            </form>
        }
    }
}
