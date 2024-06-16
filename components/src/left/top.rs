use std::rc::Rc;

use fluent::{FluentBundle, FluentResource};
use i18n::{en_us, zh_cn, LanguageType};
use utils::tr;
use web_sys::HtmlDivElement;
use yew::prelude::*;
use yewdux::Dispatch;

use icons::{
    ConnectedIcon, ContactsIcon, DisconnectIcon, HangUpLoadingIcon, MessagesIcon, SettingIcon,
};
use sandcat_sdk::{
    model::{user::User, ComponentType},
    state::{
        AppState, ComponentTypeState, ConnectState, ConvState, FriendListState, I18nState,
        MobileState, ShowRight, UnreadState,
    },
};

use crate::self_info::SelfInfo;

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
        let _conn_dis =
            Dispatch::global().subscribe(ctx.link().callback(TopMsg::ConnectionStateChanged));
        let unread_dis =
            Dispatch::global().subscribe(ctx.link().callback(TopMsg::UnreadStateChanged));
        let res = match Dispatch::<I18nState>::global().get().lang {
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
            is_mobile: Dispatch::<MobileState>::global().get().is_mobile(),
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
            TopMsg::ConnectionStateChanged(state) => self.connect_state = state,
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut msg_class = "top-icon-selected";
        let msg_onclick = if self.com_state.component_type != ComponentType::Messages {
            msg_class = "hover top-icon";
            self.com_s_dis
                .reduce_mut_callback(|s| s.component_type = ComponentType::Messages)
        } else {
            ctx.link().callback(move |_| TopMsg::EmptyCallback)
        };
        let mut contact_class = "top-icon-selected";
        let contact_onclick = if self.com_state.component_type != ComponentType::Contacts {
            contact_class = "hover top-icon";
            self.com_s_dis
                .reduce_mut_callback(|s| s.component_type = ComponentType::Contacts)
        } else {
            ctx.link().callback(move |_| TopMsg::EmptyCallback)
        };
        let mut setting_class = "top-icon-selected";
        let setting_onclick = if self.com_state.component_type != ComponentType::Setting {
            setting_class = "hover";
            self.com_s_dis.reduce_mut_callback(|s| {
                Dispatch::<ShowRight>::global().reduce_mut(|s| *s = ShowRight::Show);
                Dispatch::<ConvState>::global().set(ConvState::default());
                Dispatch::<FriendListState>::global().set(FriendListState::default());
                s.component_type = ComponentType::Setting;
            })
        } else {
            ctx.link().callback(move |_| TopMsg::EmptyCallback)
        };
        let mut msg_count = html!();
        if self.unread_state.msg_count > 0 {
            msg_count = html! {
                <span class="unread-count">
                    {self.unread_state.msg_count}
                </span>
            };
        }

        let mut contact_count = html!();
        if self.unread_state.contacts_count > 0 {
            contact_count = html! {
                <span class="unread-count">
                    {self.unread_state.contacts_count}
                </span>
            };
        }

        let mut info_panel = html!();
        if self.show_info {
            let close = ctx.link().callback(|_| TopMsg::ShowInfoPanel);
            let submit = ctx.link().callback(TopMsg::SubmitInfo);
            info_panel =
                html!(<SelfInfo user={self.app_state.login_user.clone()} {close} {submit} />)
        }
        let onclick = ctx.link().callback(|_| TopMsg::ShowInfoPanel);
        let mut top_right = html!();
        let mut top_down = html!();
        if !self.is_mobile {
            top_right = html! {
                <div class="top-right">
                    <span class={msg_class} onclick={msg_onclick}>
                        <MessagesIcon fill={"var(--color-text)"}/>
                        {msg_count}
                    </span>
                    <span class={contact_class} onclick={contact_onclick}>
                        <ContactsIcon fill={"var(--color-text)"}/>
                        {contact_count}
                    </span>
                    <span class={setting_class} onclick={setting_onclick}>
                        <SettingIcon fill={"var(--color-text)"}/>
                    </span>
                </div>
            };
        } else {
            let mut msg_class = classes!(msg_class);
            msg_class.push("top-down-style");
            let mut contact_class = classes!(contact_class);
            contact_class.push("top-down-style");
            top_down = html! {
                <div class="top-down">
                    <div class={msg_class} onclick={msg_onclick}>
                        {tr!(self.i18n, "msg")}
                        {msg_count}
                    </div>
                    <div class={contact_class} onclick={contact_onclick}>
                        {tr!(self.i18n, "contact")}
                    </div>
                </div>
            };
        }

        // connection state
        let connection_state = match *self.connect_state {
            ConnectState::DisConnect => {
                html!(
                    <div class="connection-state">
                        <DisconnectIcon />
                    </div>
                )
            }
            ConnectState::Connecting => {
                html!(
                    <div class="connection-state">
                        <HangUpLoadingIcon fill={AttrValue::from("var(--color-text)")} />
                    </div>
                )
            }
            ConnectState::Connected => {
                html!(
                    <div class="connection-state">
                        <ConnectedIcon />
                    </div>
                )
            }
        };
        html! {
            <div class="top" ref={self.node.clone()}>
                <div class="top-up">
                    {info_panel}
                    <div class="top-left pointer" {onclick}>
                        <img
                            class="avatar"
                            title={&self.app_state.login_user.name}
                            src={utils::get_avatar_url(&self.app_state.login_user.avatar)} />
                        <div class="top-left-name">
                            <span><b>{&self.app_state.login_user.name}</b></span>
                            {connection_state}
                        </div>
                    </div>
                    {top_right}
                </div>
                {top_down}
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if let Some(node) = self.node.cast::<HtmlDivElement>() {
            let _ = node.set_attribute("data-tauri-drag-region", "");
        }
    }
}
