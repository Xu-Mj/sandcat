use std::{collections::HashMap, rc::Rc};

use fluent::{FluentBundle, FluentResource};
use gloo::timers::callback::Timeout;
use i18n::{en_us, zh_cn, LanguageType};
use utils::tr;
use web_sys::HtmlDivElement;
use yew::{classes, html, Component, Context, Html, NodeRef, Properties};
use yewdux::Dispatch;

use icons::CloseIcon;
use sandcat_sdk::{
    model::notification::{Notification, NotificationType},
    state::{I18nState, Notify},
};

type NotificationList = HashMap<i64, (Rc<Notification>, Timeout)>;

pub struct NotificationCom {
    noti_ref: NodeRef,
    i18n: FluentBundle<FluentResource>,
    _i18n_dis: Dispatch<I18nState>,
    notifications: NotificationList,
    _noti_dis: Dispatch<Notification>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {}

pub enum Msg {
    Notification(Rc<Notification>),
    Remove(i64),
    I18nStateChanged(Rc<I18nState>),
}

impl Component for NotificationCom {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let _noti_dis = Dispatch::global().subscribe_silent(ctx.link().callback(Msg::Notification));
        let _i18n_dis =
            Dispatch::global().subscribe_silent(ctx.link().callback(Msg::I18nStateChanged));

        let res = match I18nState::get().lang {
            LanguageType::ZhCN => zh_cn::NOTIFICATION,
            LanguageType::EnUS => en_us::NOTIFICATION,
        };
        let i18n = utils::create_bundle(res);

        Self {
            noti_ref: NodeRef::default(),
            i18n,
            _i18n_dis,
            notifications: HashMap::new(),
            _noti_dis,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Notification(noti) => {
                let id = noti.id;
                let ctx = ctx.link().clone();
                let timeout = Timeout::new(noti.delay, move || ctx.send_message(Msg::Remove(id)));
                self.notifications.insert(id, (noti.clone(), timeout));
                true
            }
            Msg::Remove(id) => {
                self.notifications.remove(&id);
                true
            }
            Msg::I18nStateChanged(state) => {
                let res = match state.lang {
                    LanguageType::ZhCN => zh_cn::NOTIFICATION,
                    LanguageType::EnUS => en_us::NOTIFICATION,
                };
                self.i18n = utils::create_bundle(res);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let notifications = self
            .notifications
            .iter()
            .map(|(id, (item, _))| {
                let mut class = classes!("notification-item");
                let mut close_btn = html!();
                let mut content = html!({ &item.content });
                match item.type_ {
                    NotificationType::Info => class.push("info"),
                    // NotificationType::Success => class.push("success"),
                    NotificationType::Warn => class.push("warn"),
                    NotificationType::Error => {
                        class.push("error");
                        if let Some(err) = item.error.as_ref() {
                            content = html! {
                                <>
                                    <div class="notification-title"><b>{tr!(self.i18n, &err.kind().to_string())}</b></div>
                                    <div class="notification-err">{err.details()}</div>
                                </>
                            };
                        }
                        let id = *id;
                        let onclick = ctx.link().callback(move |_| Msg::Remove(id));
                        close_btn =
                            html! { <span class="notification-close" {onclick}><CloseIcon/></span>};
                    }
                }

                html! {
                    <div {class} key={*id}>
                        {close_btn}
                        {content}
                    </div>
                }
            })
            .collect::<Html>();
        html! {
            <div class="notify" ref={self.noti_ref.clone()}>
                {notifications}
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if let Some(node) = self.noti_ref.cast::<HtmlDivElement>() {
            node.set_scroll_top(node.scroll_height());
        }
    }
}
