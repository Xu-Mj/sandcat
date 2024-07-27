use std::rc::Rc;

use fluent::{FluentBundle, FluentResource};
use gloo::utils::document;
use gloo::utils::window;
use js_sys::Array;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use wasm_bindgen_futures::JsFuture;
use web_sys::Event;
use web_sys::File;
use web_sys::FilePropertyBag;
use web_sys::HtmlCanvasElement;
use web_sys::HtmlDivElement;
use web_sys::HtmlImageElement;
use web_sys::HtmlInputElement;
use yew::KeyboardEvent;
use yew::{html, Callback, Component, NodeRef, Properties};
use yew_router::scope_ext::RouterScopeExt;
use yewdux::Dispatch;

use i18n::{en_us, zh_cn, LanguageType};
use sandcat_sdk::api;
use sandcat_sdk::db;
use sandcat_sdk::model::notification::Notification;
use sandcat_sdk::model::page::Page;
use sandcat_sdk::model::user::{User, UserUpdate};
use sandcat_sdk::state::I18nState;
use sandcat_sdk::state::MobileState;
use utils::tr;

use crate::avatar::{Avatar, SubmitOption};
use crate::change_pwd::ChangePwd;
use crate::constant::CHANGE_PWD;
use crate::constant::{
    ACCOUNT, ADDRESS, CANCEL, CHOOSE_AVATAR, EMAIL, FEMALE, GENDER, LOGOUT, MALE, NICKNAME, PHONE,
    REGION, SECRET, SET_AVATAR, SIGNATURE, SUBMIT,
};

pub struct SelfInfo {
    node: NodeRef,
    i18n: FluentBundle<FluentResource>,
    phone_node: NodeRef,
    name_node: NodeRef,
    email_node: NodeRef,
    addr_node: NodeRef,
    avatar_node: NodeRef,
    avatar_url: Option<String>,
    signature_node: NodeRef,
    avatar: String,
    gender: String,
    dispatch: Dispatch<I18nState>,
    is_mobile: bool,
    show_avatar_setter: bool,
    show_change_pwd: bool,
}

#[derive(Debug)]
pub enum SelfInfoMsg {
    Submit,
    I18nStateChanged(Rc<I18nState>),
    GenderChange(Event),
    Logout,
    ShowAvatarSetter,
    SetAvatar(String),
    OnEscDown(KeyboardEvent),
    ShowChangePwd,
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
            node: NodeRef::default(),
            i18n,
            phone_node: NodeRef::default(),
            email_node: NodeRef::default(),
            addr_node: NodeRef::default(),
            name_node: NodeRef::default(),
            avatar_node: NodeRef::default(),
            avatar_url: None,
            signature_node: NodeRef::default(),
            gender: ctx.props().user.gender.to_string(),
            avatar: ctx.props().user.avatar.to_string(),
            show_avatar_setter: false,
            show_change_pwd: false,
            dispatch,
            is_mobile: MobileState::is_mobile(),
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
                log::debug!("update user info: name: {:?}; email: {:?}; address: {:?}; signature: {:?}; phone: {:?};"
                    ,name, email, address, signature, phone );
                let mut user = UserUpdate {
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
                let avatar = self.avatar_url.clone();
                spawn_local(async move {
                    // upload avatar image
                    if let Some(avatar) = avatar {
                        // convert to file
                        let file = data_url_to_file(avatar, "avatar.png").await.unwrap();

                        match api::file().upload_avatar(&file).await {
                            Ok(name) => user.avatar = name,
                            Err(e) => {
                                log::error!("upload avatar error: {:?}", e);
                                Dispatch::<Notification>::global()
                                    .set(Notification::error("upload avatar error"));
                                return;
                            }
                        }
                    }
                    match api::users().update(user).await {
                        Ok(user) => {
                            db::db_ins().users.add(&user).await;
                            submit.emit(Box::new(user));
                        }
                        Err(e) => {
                            log::error!("{:?}", e);
                            Dispatch::<Notification>::global()
                                .set(Notification::error("update user info failed"));
                            close.emit(());
                        }
                    }
                });
                false
            }
            SelfInfoMsg::I18nStateChanged(state) => {
                let res = match state.lang {
                    LanguageType::ZhCN => zh_cn::USER_INFO,
                    LanguageType::EnUS => en_us::USER_INFO,
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
                // spawn_local(async {
                //     let _ = Repository::new().await.delete_db().await;
                // });
                window().local_storage().unwrap().unwrap().clear().unwrap();
                ctx.link().navigator().unwrap().push(&Page::Login);
                false
            }
            SelfInfoMsg::ShowAvatarSetter => {
                self.show_avatar_setter = !self.show_avatar_setter;
                true
            }
            SelfInfoMsg::SetAvatar(url) => {
                if let Some(img) = self.avatar_node.cast::<HtmlImageElement>() {
                    img.set_src(&url);
                    self.avatar_url = Some(url);
                }
                self.show_avatar_setter = false;
                true
            }
            SelfInfoMsg::OnEscDown(event) => {
                if event.key() == "Escape" {
                    ctx.props().close.emit(());
                }
                event.stop_propagation();
                false
            }
            SelfInfoMsg::ShowChangePwd => {
                self.show_change_pwd = !self.show_change_pwd;
                true
            }
        }
    }
    fn view(&self, ctx: &yew::prelude::Context<Self>) -> yew::prelude::Html {
        let on_submit = ctx.link().callback(|_| SelfInfoMsg::Submit);
        let on_cancel = ctx.props().close.reform(|_| ());
        let onchange = ctx.link().callback(SelfInfoMsg::GenderChange);
        let user = ctx.props().user.clone();
        let class = if self.is_mobile {
            "info-panel info-panel-size-mobile"
        } else {
            "info-panel info-panel-size box-shadow"
        };
        let onclick = ctx.link().callback(|_| SelfInfoMsg::ShowAvatarSetter);
        let mut avatar_setter = html!();
        if self.show_avatar_setter {
            let submit = ctx.link().callback(SelfInfoMsg::SetAvatar);
            let close = ctx.link().callback(|_| SelfInfoMsg::ShowAvatarSetter);
            avatar_setter = html!(
                <div class="avatar-setter">
                    <Avatar
                        submit={SubmitOption::DataUrl(submit)}
                        {close}
                        avatar_url={self.get_avatar_url()}
                        submit_text={tr!(self.i18n, SUBMIT)}
                        cancel_text={tr!(self.i18n, CANCEL)}
                        choose_text={tr!(self.i18n, CHOOSE_AVATAR)}/>
                </div>
            )
        }

        let set_pwd_click = ctx.link().callback(|_| SelfInfoMsg::ShowChangePwd);

        let change_pwd = if self.show_change_pwd {
            let lang = self.dispatch.get().lang;
            let email = user.email.clone().unwrap();
            let close = ctx.link().callback(|_| SelfInfoMsg::ShowChangePwd);
            let user_id = user.id;
            html!(<ChangePwd {user_id} {close} {lang} {email}/>)
        } else {
            html!()
        };

        html! {
            <>
            {change_pwd}
            <div tabindex="-1" {class} ref={self.node.clone()} onkeydown={ctx.link().callback(SelfInfoMsg::OnEscDown)}>
                {avatar_setter}
                <div class="info-panel-item-avatar">
                    <label for="avatar" {onclick}>
                        <span>
                            {tr!(self.i18n, SET_AVATAR)}
                        </span>
                        <img ref={self.avatar_node.clone()}
                            src={format!("/api/file/avatar/get/{}", user.avatar)}
                            alt="avatar"
                            class="info-panel-avatar" />
                    </label>
                </div>
                <div class="info-panel-item">
                    <label for="nickname">
                        {tr!(self.i18n, NICKNAME)}
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
                    <label>{tr!(self.i18n, ACCOUNT)}</label>
                    <span>{user.account}</span>
                </div>
                <div class="info-panel-item">
                    <label>{tr!(self.i18n, CHANGE_PWD)}</label>
                    <button type="button" class="set-pwd-btn" onclick={set_pwd_click}>
                        {tr!(self.i18n, CHANGE_PWD)}
                    </button>
                </div>
                <div class="info-panel-item">
                    <label>
                        {tr!(self.i18n, GENDER)}
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
                            {tr!(self.i18n, MALE)}
                        </label>
                        <label for="female">
                            <input
                                type="radio"
                                id="female"
                                name="gender"
                                value="female"
                                checked={self.gender == "female"}
                                onchange={onchange.clone()}/>
                            {tr!(self.i18n, FEMALE)}
                        </label>
                        <label for="secret">
                            <input
                                type="radio"
                                id="secret"
                                name="gender"
                                value="secret"
                                checked={self.gender == "secret" || self.gender.is_empty() }
                                {onchange}/>
                            {tr!(self.i18n, SECRET)}
                        </label>

                    </div>
                </div>
                <div class="info-panel-item">
                    <label for="phone">
                        {tr!(self.i18n, PHONE)}
                    </label>
                    <input ref={self.phone_node.clone()}
                        type="text"
                        id="phone"
                        name="phone"
                        placeholder={tr!(self.i18n, PHONE)}
                        value={user.phone}
                            />
                </div>
                <div class="info-panel-item">
                    <label for="email">
                        {tr!(self.i18n, EMAIL)}
                    </label>
                    <input ref={self.email_node.clone()}
                        type="text"
                        id="email"
                        name="email"
                        placeholder={tr!(self.i18n, EMAIL)}
                        required={true}
                        value={user.email}
                        autocomplete="current-password"
                        /* onchange={ctx.link().callback(|_|RegisterMsg::OnEmailChange)} */ />
                </div>
                <div class="info-panel-item">
                    <label for="address">
                        {tr!(self.i18n, ADDRESS)}
                    </label>
                    <input
                        ref={self.addr_node.clone()}
                        type="text"
                        id="address"
                        required={true}
                        autocomplete="address"
                        placeholder={tr!(self.i18n, ADDRESS)}
                        value={user.address}
                        />
                </div>
                <div class="info-panel-item">
                    <label for="signature">
                        {tr!(self.i18n, SIGNATURE)}
                    </label>
                    <input
                        ref={self.signature_node.clone()}
                        type="text"
                        id="signature"
                        required={true}
                        autocomplete="signature"
                        placeholder={tr!(self.i18n, SIGNATURE)}
                        value={user.signature}
                        />
                </div>


                <div class="info-panel-item">
                    <label>{tr!(self.i18n, REGION)}</label>
                    {user.region}
                </div>
                <div class="info-panel-btn">
                    <button type="submit" aria-label={tr!(self.i18n, SUBMIT)} onclick={on_submit}>{tr!(self.i18n, SUBMIT)}</button>
                    <button type="button" aria-label={tr!(self.i18n, CANCEL)} onclick={on_cancel}>{tr!(self.i18n, CANCEL)}</button>
                    <button type="button" aria-label={tr!(self.i18n, LOGOUT)} onclick={ctx.link().callback(|_| SelfInfoMsg::Logout)}>{tr!(self.i18n, LOGOUT)}</button>
                </div>
            </div>
            </>
        }
    }

    fn rendered(&mut self, _ctx: &yew::Context<Self>, _first_render: bool) {
        let _ = self.node.cast::<HtmlDivElement>().unwrap().focus();
    }
}

impl SelfInfo {
    fn get_avatar_url(&self) -> String {
        if let Some(image) = self.avatar_node.cast::<HtmlImageElement>() {
            let canvas = document().create_element("canvas").unwrap();
            let canvas: HtmlCanvasElement = canvas.dyn_into().unwrap();
            canvas.set_width(image.natural_width());
            canvas.set_height(image.natural_height());
            let ctx = canvas
                .get_context("2d")
                .unwrap()
                .unwrap()
                .dyn_into::<web_sys::CanvasRenderingContext2d>()
                .unwrap();
            ctx.draw_image_with_html_image_element_and_dw_and_dh(
                &image,
                0.0,
                0.0,
                image.natural_width() as f64,
                image.natural_height() as f64,
            )
            .unwrap();
            canvas.to_data_url().unwrap()
        } else {
            String::new()
        }
    }
}

async fn data_url_to_file(data_url: String, file_name: &str) -> Result<File, JsValue> {
    // fetch the Data URL
    let response = JsFuture::from(window().fetch_with_str(&data_url)).await?;
    let response: web_sys::Response = response.dyn_into()?;

    // get the Blob from the response
    let blob = JsFuture::from(response.blob()?).await?;
    let blob: web_sys::Blob = blob.dyn_into()?;

    // create a File from the Blob
    let mut property_bag = FilePropertyBag::new();
    property_bag.type_("image/png");

    File::new_with_blob_sequence_and_options(&Array::of1(&blob), file_name, &property_bag)
}
