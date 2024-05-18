use std::rc::Rc;

use fluent::{FluentBundle, FluentResource};
use gloo::utils::window;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::Event;
use web_sys::HtmlInputElement;
use yew::{html, Callback, Component, NodeRef, Properties};
use yew_router::scope_ext::RouterScopeExt;
use yewdux::Dispatch;

use i18n::{en_us, zh_cn, LanguageType};
use sandcat_sdk::db;
use sandcat_sdk::db::repository::Repository;
use sandcat_sdk::model::page::Page;
use sandcat_sdk::model::user::{User, UserUpdate};
use sandcat_sdk::state::I18nState;
use utils::tr;

pub struct SelfInfo {
    i18n: FluentBundle<FluentResource>,
    phone_node: NodeRef,
    name_node: NodeRef,
    email_node: NodeRef,
    addr_node: NodeRef,
    signature_node: NodeRef,
    avatar: String,
    gender: String,
    _dispatch: Dispatch<I18nState>,
}

#[derive(Debug)]
pub enum SelfInfoMsg {
    Submit,
    I18nStateChanged(Rc<I18nState>),
    GenderChange(Event),
    Logout,
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
        let dispatch =
            Dispatch::global().subscribe_silent(ctx.link().callback(SelfInfoMsg::I18nStateChanged));
        let res = match dispatch.get().lang {
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
            gender: ctx.props().user.gender.to_string(),
            avatar: ctx.props().user.avatar.to_string(),
            _dispatch: dispatch,
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
                log::debug!("update user info: name: {:?}; email: {:?}; address: {:?}; signature: {:?}; phone: {:?};",name, email, address, signature, phone );
                let user = UserUpdate {
                    id: ctx.props().user.id.to_string(),
                    name,
                    avatar: self.avatar.clone(),
                    gender: self.gender.clone(),
                    email: (!email.is_empty()).then_some(email),
                    phone: (!phone.is_empty()).then_some(phone),
                    address: (!address.is_empty()).then_some(address),
                    signature: (!signature.is_empty()).then_some(signature),
                };
                let close = ctx.props().close.clone();
                let submit = ctx.props().submit.clone();
                spawn_local(async move {
                    match api::users().update(user).await {
                        Ok(user) => {
                            db::db_ins().users.add(&user).await;
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
            SelfInfoMsg::GenderChange(e) => {
                let gender = e
                    .target()
                    .unwrap()
                    .dyn_into::<HtmlInputElement>()
                    .unwrap()
                    .value();
                self.gender = gender;
                true
            }
            SelfInfoMsg::Logout => {
                log::debug!("user logout ==> delete database");
                // 测试阶段，销毁时删除数据库
                spawn_local(async {
                    let _ = Repository::new().await.delete_db().await;
                });
                window().local_storage().unwrap().unwrap().clear().unwrap();
                ctx.link().navigator().unwrap().push(&Page::Login);
                false
            }
        }
    }
    fn view(&self, ctx: &yew::prelude::Context<Self>) -> yew::prelude::Html {
        let on_submit = ctx.link().callback(|_| SelfInfoMsg::Submit);
        let on_cancel = ctx.props().close.reform(|_| ());
        let onchange = ctx.link().callback(SelfInfoMsg::GenderChange);
        let user = ctx.props().user.clone();
        log::debug!("user: {:?}", user);
        html! {
            <div class="info-panel box-shadow">
                <div class="info-panel-item-avatar">
                    <input type="file" id="avatar" name="avatar" hidden={true} accept="image/*"/>
                    <label for="avatar">
                        <span>
                            {tr!(self.i18n, "set_avatar")}
                        </span>
                        <img src={user.avatar} alt="avatar" class="info-panel-avatar" />
                    </label>
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
                    <label>
                        {tr!(self.i18n, "gender")}
                    </label>
                    <div class="info-panel-item-gender">
                        <label for="male">
                            <input
                                type="radio"
                                name="gender"
                                id="male"
                                value="male"
                                checked={self.gender == "male"}
                                onchange={onchange.clone()}/>
                            {tr!(self.i18n, "male")}
                        </label>
                        <label for="female">
                            <input
                                type="radio"
                                id="female"
                                name="gender"
                                value="female"
                                checked={self.gender == "female"}
                                onchange={onchange.clone()}/>
                            {tr!(self.i18n, "female")}
                        </label>
                        <label for="secret">
                            <input
                                type="radio"
                                id="secret"
                                name="gender"
                                value="secret"
                                checked={self.gender == "secret" || self.gender.is_empty() }
                                {onchange}/>
                            {tr!(self.i18n, "secret")}
                        </label>

                    </div>
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
                        value={user.phone}
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
                        value={user.email}
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
                    <button type="button" onclick={ctx.link().callback(|_| SelfInfoMsg::Logout)}>{tr!(self.i18n, "logout")}</button>
                </div>
            </div>
        }
    }
}
