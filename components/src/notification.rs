use std::{collections::HashMap, rc::Rc};

use gloo::timers::callback::Timeout;
use yew::{classes, html, Component, Context, Html, Properties};
use yewdux::Dispatch;

use sandcat_sdk::model::notification::{Notification, NotificationType};

type NotificationList = HashMap<i64, (Rc<Notification>, Timeout)>;

pub struct NotificationCom {
    notifications: NotificationList,
    _noti_dis: Dispatch<Notification>,
}

#[derive(Clone, PartialEq, Properties)]
pub struct Props {}

pub enum Msg {
    Notification(Rc<Notification>),
    Remove(i64),
}

impl Component for NotificationCom {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let _noti_dis = Dispatch::global().subscribe_silent(ctx.link().callback(Msg::Notification));
        Self {
            notifications: HashMap::new(),
            _noti_dis,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Notification(noti) => {
                let id = chrono::Utc::now().timestamp_millis();
                let ctx = ctx.link().clone();
                let timeout = Timeout::new(noti.delay, move || ctx.send_message(Msg::Remove(id)));
                self.notifications.insert(id, (noti.clone(), timeout));
                true
            }
            Msg::Remove(id) => {
                self.notifications.remove(&id);
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let notifications = self
            .notifications
            .iter()
            .map(|(id, (item, _))| {
                let mut class = classes!("notification-item");
                match item.type_ {
                    NotificationType::Info => class.push("info"),
                    // NotificationType::Success => class.push("success"),
                    NotificationType::Warn => class.push("warn"),
                    NotificationType::Error => class.push("error"),
                }
                html! {
                    <div {class} key={*id}>
                        {item.content.clone()}
                    </div>
                }
            })
            .collect::<Html>();
        html! {
            <div class="notify">
                {notifications}
            </div>
        }
    }
}
