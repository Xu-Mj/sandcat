use std::rc::Rc;

use fluent::{FluentBundle, FluentResource};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::{html, Callback, Component, ContextHandle, NodeRef, Properties};

use crate::{
    api, db,
    i18n::{en_us, zh_cn, LanguageType},
    model::user::{User, UserUpdate},
    pages::I18nState,
    tr, utils,
};

pub struct SelfInfo {
    i18n: FluentBundle<FluentResource>,
    phone_node: NodeRef,
    name_node: NodeRef,
    email_node: NodeRef,
    addr_node: NodeRef,
    signature_node: NodeRef,
    gender: String,
    _i18n_state: Rc<I18nState>,
    _i18n_handler: ContextHandle<Rc<I18nState>>,
}

pub enum SelfInfoMsg {
    Submit,
    I18nStateChanged(Rc<I18nState>),
}

#[derive(Properties, PartialEq, Clone)]
pub struct SelfInfoProps {
    pub user: User,
    pub close: Callback<()>,
    pub submit: Callback<Box<User>>,
}

impl Component for SelfInfo {
    type Message = SelfInfoMsg;

    type Properties = SelfInfoProps;

    fn create(ctx: &yew::prelude::Context<Self>) -> Self {
        let (i18n_state, _i18n_handler) = ctx
            .link()
            .context::<Rc<I18nState>>(ctx.link().callback(SelfInfoMsg::I18nStateChanged))
            .expect("need state");

        let res = match i18n_state.lang {
            LanguageType::ZhCN => zh_cn::USER_INFO,
            LanguageType::EnUS => en_us::USER_INFO,
        };
        let i18n = utils::create_bundle(res);
        Self {
            i18n,
            phone_node: NodeRef::default(),
            email_node: NodeRef::default(),
            addr_node: NodeRef::default(),
            name_node: NodeRef::default(),
            signature_node: NodeRef::default(),
            gender: String::new(),
            _i18n_state: i18n_state,
            _i18n_handler,
        }
    }

    fn update(&mut self, ctx: &yew::prelude::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SelfInfoMsg::Submit => {
                let name = self.name_node.cast::<HtmlInputElement>().unwrap().value();
                let email = self.email_node.cast::<HtmlInputElement>().unwrap().value();
                let phone = self.phone_node.cast::<HtmlInputElement>().unwrap().value();
                let address = self.addr_node.cast::<HtmlInputElement>().unwrap().value();
                let signature = self
                    .signature_node
                    .cast::<HtmlInputElement>()
                    .unwrap()
                    .value();
                let user = UserUpdate {
                    name,
                    gender: self.gender.clone(),
                    email: email.is_empty().then_some(email),
                    phone: phone.is_empty().then_some(phone),
                    address: address.is_empty().then_some(address),
                    signature: signature.is_empty().then_some(signature),
                };
                let close = ctx.props().close.clone();
                let submit = ctx.props().submit.clone();
                spawn_local(async move {
                    match api::users().update(user).await {
                        Ok(user) => {
                            db::users().await.add(&user).await;
                            submit.emit(Box::new(user));
                        }
                        Err(e) => {
                            log::error!("{:?}", e);
                            close.emit(());
                        }
                    }
                });
                false
            }
            SelfInfoMsg::I18nStateChanged(state) => {
                let res = match state.lang {
                    LanguageType::ZhCN => zh_cn::ADD_FRIEND,
                    LanguageType::EnUS => en_us::ADD_FRIEND,
                };
                let i18n = utils::create_bundle(res);
                self.i18n = i18n;
                true
            }
        }
    }
    fn view(&self, ctx: &yew::prelude::Context<Self>) -> yew::prelude::Html {
        let on_submit = ctx.link().callback(|_| SelfInfoMsg::Submit);
        let on_cancel = ctx.props().close.reform(|_| ());
        let user = ctx.props().user.clone();
        html! {
            <div class="info-panel box-shadow">
                <div class="info-panel-item-avatar">
                    <img src={user.avatar} class="info-panel-avatar" />
                </div>
                <div class="info-panel-item">
                    <label for="nickname">
                        {tr!(self.i18n, "name")}
                    </label>
                    <input
                        ref={self.name_node.clone()}
                        type="text"
                        id="nickname"
                        placeholder="nickname"
                        required={true}
                        autocomplete="nickname"
                        value={user.name}
                        />
                </div>
                <div class="info-panel-item">
                    <label>{tr!(self.i18n, "account")}</label>
                    <span>{user.account}</span>
                </div>
                <div class="info-panel-item">
                    <label for="gender">
                        {tr!(self.i18n, "gender")}
                    </label>

                </div>
                <div class="info-panel-item">
                    <label for="phone">
                        {tr!(self.i18n, "phone")}
                    </label>
                    <input ref={self.phone_node.clone()}
                        type="text"
                        id="phone"
                        name="phone"
                        placeholder={tr!(self.i18n, "phone")}
                            />
                </div>
                <div class="info-panel-item">
                    <label for="email">
                        {tr!(self.i18n, "email")}
                    </label>
                    <input ref={self.email_node.clone()}
                        type="text"
                        id="email"
                        name="email"
                        placeholder={tr!(self.i18n, "email")}
                        required={true}
                        autocomplete="current-password"
                        /* onchange={ctx.link().callback(|_|RegisterMsg::OnEmailChange)} */ />
                </div>
                <div class="info-panel-item">
                    <label for="address">
                        {tr!(self.i18n, "address")}
                    </label>
                    <input
                        ref={self.addr_node.clone()}
                        type="text"
                        id="address"
                        required={true}
                        autocomplete="address"
                        placeholder={tr!(self.i18n, "address")}
                        value={user.address}
                        />
                </div>
                <div class="info-panel-item">
                    <label for="signature">
                        {tr!(self.i18n, "signature")}
                    </label>
                    <input
                        ref={self.signature_node.clone()}
                        type="text"
                        id="signature"
                        required={true}
                        autocomplete="signature"
                        placeholder={tr!(self.i18n, "signature")}
                        value={user.signature}
                        />
                </div>


                <div class="info-panel-item">
                    <label>{tr!(self.i18n, "region")}</label>
                    {user.region}
                </div>
                <div class="info-panel-btn">
                    <button type="submit" onclick={on_submit}>{tr!(self.i18n, "submit")}</button>
                    <button type="button" onclick={on_cancel}>{tr!(self.i18n, "cancel")}</button>
                </div>
            </div>
        }
    }
}
