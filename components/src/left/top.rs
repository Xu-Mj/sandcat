use std::rc::Rc;

use fluent::{FluentBundle, FluentResource};
use web_sys::HtmlDivElement;
use yew::prelude::*;
use yewdux::Dispatch;

use i18n::{en_us, zh_cn, LanguageType};
use icons::{
    ConnectedIcon, ContactsIcon, DisconnectIcon, HangUpLoadingIcon, MessagesIcon, SettingIcon,
};
use sandcat_sdk::{
    model::{user::User, ComponentType, OFFLINE_TIME},
    state::{
        AppState, ComponentTypeState, ConnectState, I18nState, MobileState, Notify, UnreadState,
    },
};
use utils::tr;

use crate::{
    constant::{CONTACTS, MSG},
    self_info::SelfInfo,
};

/// 增加双击切换置顶未读消息
pub struct Top {
    node: NodeRef,
    show_info: bool,
    app_state: Rc<AppState>,
    app_s_dis: Dispatch<AppState>,
    com_state: Rc<ComponentTypeState>,
    com_s_dis: Dispatch<ComponentTypeState>,
    unread_state: Rc<UnreadState>,
    _unread_dis: Dispatch<UnreadState>,
    connect_state: Rc<ConnectState>,
    _conn_dis: Dispatch<ConnectState>,
    i18n: FluentBundle<FluentResource>,
    is_mobile: bool,
}

#[derive(Properties, PartialEq)]
pub struct TopProps {}

pub enum TopMsg {
    UnreadStateChanged(Rc<UnreadState>),
    EmptyCallback,
    ShowInfoPanel,
    SubmitInfo(Box<User>),
    AppStateChanged(Rc<AppState>),
    ComStateChanged(Rc<ComponentTypeState>),
    // listen the connection state
    ConnectionStateChanged(Rc<ConnectState>),
}

impl Component for Top {
    type Message = TopMsg;

    type Properties = TopProps;

    fn create(ctx: &Context<Self>) -> Self {
        let dispatch = Dispatch::global().subscribe(ctx.link().callback(TopMsg::AppStateChanged));
        let com_s_dis = Dispatch::global().subscribe(ctx.link().callback(TopMsg::ComStateChanged));
        let _conn_dis = Dispatch::global()
            .subscribe_silent(ctx.link().callback(TopMsg::ConnectionStateChanged));
        let unread_dis =
            Dispatch::global().subscribe(ctx.link().callback(TopMsg::UnreadStateChanged));
        let res = match I18nState::get().lang {
            LanguageType::ZhCN => zh_cn::TOP,
            LanguageType::EnUS => en_us::TOP,
        };
        let i18n = utils::create_bundle(res);
        Self {
            node: NodeRef::default(),
            show_info: false,
            app_state: dispatch.get(),
            app_s_dis: dispatch,
            unread_state: unread_dis.get(),
            _unread_dis: unread_dis,
            com_state: com_s_dis.get(),
            com_s_dis,
            connect_state: _conn_dis.get(),
            _conn_dis,
            i18n,
            is_mobile: MobileState::is_mobile(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TopMsg::EmptyCallback => return false,
            TopMsg::UnreadStateChanged(state) => self.unread_state = state,
            TopMsg::ShowInfoPanel => self.show_info = !self.show_info,
            TopMsg::AppStateChanged(state) => self.app_state = state,
            TopMsg::ComStateChanged(state) => self.com_state = state,
            TopMsg::SubmitInfo(user) => {
                self.show_info = !self.show_info;
                self.app_s_dis.reduce_mut(|s| s.login_user = *user);
            }
            TopMsg::ConnectionStateChanged(state) => {
                self.connect_state = state;

                if *self.connect_state == ConnectState::DisConnect {
                    let now = chrono::Utc::now().timestamp_millis();
                    if let Err(err) = utils::set_local_storage(OFFLINE_TIME, &now.to_string()) {
                        log::error!("record offline time to local storage error: {:?}", err);
                    }
                }
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let (msg_class, msg_onclick) = self.get_top_icon_class_and_callback(
            ctx,
            ComponentType::Messages,
            "top-icon-selected",
            "hover top-icon",
        );
        let (contact_class, contact_onclick) = self.get_top_icon_class_and_callback(
            ctx,
            ComponentType::Contacts,
            "top-icon-selected",
            "hover top-icon",
        );
        let (setting_class, setting_onclick) = self.get_top_icon_class_and_callback(
            ctx,
            ComponentType::Setting,
            "top-icon-selected",
            "hover",
        );

        let msg_count = self.get_unread_count(self.unread_state.msg_count);
        let contact_count = self.get_unread_count(self.unread_state.contacts_count);
        let info_panel = if self.show_info {
            let close = ctx.link().callback(|_| TopMsg::ShowInfoPanel);
            let submit = ctx.link().callback(TopMsg::SubmitInfo);
            html!(<SelfInfo user={self.app_state.login_user.clone()} {close} {submit} />)
        } else {
            html!()
        };

        let onclick = ctx.link().callback(|_| TopMsg::ShowInfoPanel);

        let top_right = if !self.is_mobile {
            html! {
                <div class="top-right">
                    <span class={msg_class.clone()} onclick={msg_onclick.clone()}>
                        <MessagesIcon fill={"var(--color-text)"}/>
                        { msg_count.clone() }
                    </span>
                    <span class={contact_class.clone()} onclick={contact_onclick.clone()}>
                        <ContactsIcon fill={"var(--color-text)"}/>
                        { contact_count.clone() }
                    </span>
                    <span class={setting_class.clone()} onclick={setting_onclick.clone()}>
                        <SettingIcon fill={"var(--color-text)"}/>
                    </span>
                </div>
            }
        } else {
            html! {
                <div class="top-down">
                    <div class={classes!(msg_class.clone(), "top-down-style")} onclick={msg_onclick.clone()}>
                        { tr!(self.i18n, MSG) }
                        { msg_count.clone() }
                    </div>
                    <div class={classes!(contact_class.clone(), "top-down-style")} onclick={contact_onclick.clone()}>
                        { tr!(self.i18n, CONTACTS) }
                        { contact_count.clone() }
                    </div>
                </div>
            }
        };

        let connection_state = self.render_connection_state();

        html! {
            <div class="top" ref={self.node.clone()}>
                <div class="top-up">
                    { info_panel }
                    <div class="top-left pointer" {onclick}>
                        <img
                            class="avatar"
                            alt="avatar"
                            title={&self.app_state.login_user.name}
                            src={utils::get_avatar_url(&self.app_state.login_user.avatar)} />
                        <div class="top-left-name">
                            <span><b>{&self.app_state.login_user.name}</b></span>
                            { connection_state }
                        </div>
                    </div>
                    { top_right }
                </div>
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if let Some(node) = self.node.cast::<HtmlDivElement>() {
            let _ = node.set_attribute("data-tauri-drag-region", "");
        }
    }
}

impl Top {
    fn render_connection_state(&self) -> Html {
        match *self.connect_state {
            ConnectState::DisConnect => {
                html!(<div class="connection-state"><DisconnectIcon /></div>)
            }
            ConnectState::Connecting => {
                html!(<div class="connection-state"><HangUpLoadingIcon fill={AttrValue::from("var(--color-text)")} /></div>)
            }
            ConnectState::Connected => html!(<div class="connection-state"><ConnectedIcon /></div>),
        }
    }

    fn get_top_icon_class_and_callback(
        &self,
        ctx: &Context<Self>,
        component_type: ComponentType,
        default_class: &str,
        active_class: &str,
    ) -> (String, Callback<MouseEvent>) {
        if self.com_state.component_type != component_type {
            let callback = self
                .com_s_dis
                .reduce_mut_callback(move |s| s.component_type = component_type);
            (active_class.to_string(), callback)
        } else {
            let callback = ctx.link().callback(move |_| TopMsg::EmptyCallback);
            (default_class.to_string(), callback)
        }
    }

    fn get_unread_count(&self, count: usize) -> Html {
        if count > 0 {
            html! {
                <span class="unread-count">{ count }</span>
            }
        } else {
            html!()
        }
    }
}
