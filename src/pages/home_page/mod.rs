pub(crate) mod home;
use std::rc::Rc;

use gloo::timers::callback::Interval;
use gloo::utils::window;
use web_sys::HtmlDivElement;
use yew::platform::spawn_local;
use yew::prelude::*;
use yewdux::Dispatch;

use crate::db::repository::Repository;
use crate::icons::CloseIcon;
use crate::model::notification::{Notification, NotificationType};
use crate::model::user::User;

use crate::state::{AppState, NotificationState};
use crate::{
    components::{left::Left, right::Right},
    db::QueryStatus,
};

pub struct Home {
    notification_node: NodeRef,
    notification_interval: Option<Interval>,
    notifications: Vec<Notification>,
    _noti_dis: Dispatch<NotificationState>,
}

#[derive(Debug)]
pub enum HomeMsg {
    // 查询数据库
    Query(Box<QueryStatus<User>>),
    NotificationStateChanged(Rc<NotificationState>),
    CleanNotification,
    CloseNotificationByIndex(usize),
}

#[derive(Properties, Clone, PartialEq)]
pub struct HomeProps {
    pub id: AttrValue,
}

impl Component for Home {
    type Message = HomeMsg;
    type Properties = HomeProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self::new(ctx)
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::debug!("home update: {:?}", msg);
        match msg {
            HomeMsg::Query(status) => {
                match *status {
                    QueryStatus::QuerySuccess(u) => {
                        Dispatch::<AppState>::global().reduce_mut(|s| {
                            s.login_user = u;
                        });
                    }
                    QueryStatus::QueryFail(_) => {
                        gloo::console::log!("query fail")
                    }
                    _ => {}
                }
                true
            }
            HomeMsg::CleanNotification => {
                if !self.notifications.is_empty() {
                    self.notifications.remove(0);
                } else {
                    self.notification_interval = None;
                }
                true
            }
            HomeMsg::CloseNotificationByIndex(index) => {
                if index < self.notifications.len() {
                    self.notifications.remove(index);
                    return true;
                }
                false
            }
            HomeMsg::NotificationStateChanged(state) => {
                self.notify(state.noti.clone());
                let ctx = ctx.link().clone();
                if self.notification_interval.is_none() {
                    self.notification_interval = Some(Interval::new(3 * 1000, move || {
                        ctx.send_message(HomeMsg::CleanNotification)
                    }));
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let notify = self
            .notifications
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let mut class = classes!("notification-item") ;
                match item.type_ {
                    NotificationType::Info => class.push("info") ,
                    // NotificationType::Success => class.push("success"),
                    NotificationType::Warn => class.push("warn"),
                    NotificationType::Error => class.push("error"),
                }
                html! {
                    <div {class} key={index}>
                        <div class="notification-close" onclick={ctx.link().callback(move |_| HomeMsg::CloseNotificationByIndex(index))}>
                            <CloseIcon />
                        </div>
                        <div class="notification-title">
                            {item.title.clone()}
                        </div>
                        <div class="notification-content">
                            {item.content.clone()}
                        </div>
                    </div>
                }
            })
            .collect::<Html>();

        html! {
                <div class="home" id="app">
                    <Left user_id={ctx.props().id.clone()}/>
                    <Right />
                    // 通知组件

                    <div class="notify" ref={self.notification_node.clone()}>
                        {notify}
                    </div>
                </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        // 将通知区域向上滚动
        if self.notifications.len() > 2 {
            if let Some(div) = self.notification_node.cast::<HtmlDivElement>() {
                div.set_scroll_top(div.scroll_height());
            }
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        // self.ws.borrow_mut().cleanup();
        log::debug!("home destroy==> delete database");
        // 测试阶段，销毁时删除数据库
        spawn_local(async {
            let _ = Repository::new().await.delete_db().await;
        });
        window().local_storage().unwrap().unwrap().clear().unwrap();
    }
}
