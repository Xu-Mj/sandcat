use std::rc::Rc;

use gloo::timers::callback::Interval;
use gloo::utils::window;
use web_sys::HtmlDivElement;
use yew::{classes, html, AttrValue, Component, Context, Html, NodeRef, Properties};
use yewdux::Dispatch;

use components::left::Left;
use components::right::Right;
use icons::CloseIcon;
use sandcat_sdk::db::{self, QueryError, QueryStatus, DB_NAME};
use sandcat_sdk::model::notification::{Notification, NotificationType};
use sandcat_sdk::model::user::User;
use sandcat_sdk::state::{
    AppState, FontSizeState, MobileState, NotificationState, ShowRight, ThemeState,
};

pub struct Home {
    notification_node: NodeRef,
    notification_interval: Option<Interval>,
    notifications: Vec<Notification>,
    _noti_dis: Dispatch<NotificationState>,
    _theme_dis: Dispatch<ThemeState>,
    _right_dis: Dispatch<ShowRight>,
    _font_size_dis: Dispatch<FontSizeState>,
    db_inited: bool,
}

#[derive(Debug)]
pub enum HomeMsg {
    // 查询数据库
    Query(Box<QueryStatus<User>>),
    NotificationStateChanged(Rc<NotificationState>),
    SwitchTheme(Rc<ThemeState>),
    ShowRight,
    SwitchFontSize(Rc<FontSizeState>),
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
                        self.db_inited = true;
                    }
                    QueryStatus::QueryFail(_) => {
                        gloo::console::log!("query fail")
                    }
                    _ => {}
                }
                true
            }
            HomeMsg::CleanNotification => {
                log::debug!("clean notification, {:?}", self.notifications);
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
            HomeMsg::SwitchTheme(state) => {
                utils::set_theme(&state.to_string());
                false
            }
            HomeMsg::SwitchFontSize(state) => {
                log::debug!("switch font size: {:?}", state);
                utils::set_font_size(&state.to_string());
                false
            }
            HomeMsg::ShowRight => true,
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
        if !self.db_inited {
            return html! {};
        }
        log::debug!("home view: {:?}", Dispatch::<ShowRight>::global().get());
        let (right, class) = match *Dispatch::<MobileState>::global().get() {
            MobileState::Desktop => (html!(<Right />), "home"),
            MobileState::Mobile => match *Dispatch::<ShowRight>::global().get() {
                ShowRight::None => (html!(), "home-mobile"),
                ShowRight::Show => (html!(<Right />), "home-mobile"),
            },
        };
        html! {
                <div {class} id="app">
                    <Left user_id={ctx.props().id.clone()}/>
                    {right}
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
}

async fn query(id: &str) -> Result<User, QueryError> {
    let user = db::db_ins().users.get(id).await.unwrap();

    Ok(user)
}

impl Home {
    pub fn new(ctx: &Context<Self>) -> Self {
        // 测试数据库
        // 查询当前登录用户放到登录中
        let id = ctx.props().id.clone();
        // 每次创建Home组件时，检查一下数据库名是否存在，不存在则创建
        // 这样就能保证每次创建Home组件时，数据库名都是当前登录用户的id
        DB_NAME.get_or_init(|| format!("im-{}", id));
        let clone_id = id.clone();
        ctx.link().send_future(async move {
            db::init_db().await;
            match query(clone_id.as_str()).await {
                Ok(data) => HomeMsg::Query(Box::new(QueryStatus::QuerySuccess(data))),
                Err(err) => HomeMsg::Query(Box::new(QueryStatus::QueryFail(err))),
            }
        });

        // query device info
        if let Ok(platform) = window().navigator().user_agent() {
            log::debug!("platform: {:?}", platform);

            if platform.contains("Mobile")
                || platform.contains("Android")
                || platform.contains("iPhone")
            {
                Dispatch::<MobileState>::global().set(MobileState::Mobile);
            } else {
                Dispatch::<MobileState>::global().set(MobileState::Desktop);
            }
        }
        // 使用ctx发送一个正在查询的状态
        ctx.link()
            .send_message(HomeMsg::Query(Box::new(QueryStatus::Querying)));

        let noti_dis = Dispatch::global()
            .subscribe_silent(ctx.link().callback(HomeMsg::NotificationStateChanged));
        let _theme_dis = Dispatch::global().subscribe(ctx.link().callback(HomeMsg::SwitchTheme));
        let _right_dis = Dispatch::global().subscribe(ctx.link().callback(|_| HomeMsg::ShowRight));
        let _font_size_dis =
            Dispatch::global().subscribe(ctx.link().callback(HomeMsg::SwitchFontSize));
        Self {
            notifications: vec![],
            _noti_dis: noti_dis,
            notification_node: NodeRef::default(),
            notification_interval: None,
            _theme_dis,
            _font_size_dis,
            db_inited: false,
            _right_dis,
        }
    }

    pub fn info(&mut self, value: AttrValue) {
        self.notifications.push(Notification {
            type_: NotificationType::Info,
            title: AttrValue::from("INFO"),
            content: value,
        });
    }

    pub fn warn(&mut self, value: AttrValue) {
        self.notifications.push(Notification {
            type_: NotificationType::Info,
            title: AttrValue::from("WARN"),
            content: value,
        });
    }

    pub fn error(&mut self, value: AttrValue) {
        self.notifications.push(Notification {
            type_: NotificationType::Error,
            title: AttrValue::from("ERROR"),
            content: value,
        });
    }

    pub fn notify(&mut self, notify: Notification) {
        match notify.type_ {
            NotificationType::Info => self.info(notify.content),
            // NotificationType::Success => {}
            NotificationType::Warn => self.warn(notify.content),
            NotificationType::Error => self.error(notify.content),
        }
    }
}
