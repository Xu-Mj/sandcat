use std::rc::Rc;

use gloo::utils::window;
use yew::{html, AttrValue, Component, Context, Html, Properties};
use yewdux::Dispatch;

use components::left::Left;
use components::notification::NotificationCom;
use components::right::Right;
use sandcat_sdk::db::{self, QueryStatus, DB_NAME};
use sandcat_sdk::model::user::User;
use sandcat_sdk::state::{AppState, FontSizeState, MobileState, Notify, ShowRight, ThemeState};

pub struct Home {
    _theme_dis: Dispatch<ThemeState>,
    _right_dis: Dispatch<ShowRight>,
    _font_size_dis: Dispatch<FontSizeState>,
    db_inited: bool,
}

#[derive(Debug)]
pub enum HomeMsg {
    // 查询数据库
    Query(Box<QueryStatus<User>>),
    SwitchTheme(Rc<ThemeState>),
    ShowRight,
    SwitchFontSize(Rc<FontSizeState>),
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

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::debug!("home update: {:?}", msg);
        match msg {
            HomeMsg::Query(status) => {
                match *status {
                    QueryStatus::QuerySuccess(u) => {
                        AppState { login_user: u }.notify();
                        self.db_inited = true;
                    }
                    QueryStatus::QueryFail(e) => {
                        log::error!("query fail{:?}", e)
                    }
                    _ => {}
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
        if !self.db_inited {
            return html! {};
        }
        let (right, class) = match *MobileState::get() {
            MobileState::Desktop => (html!(<Right />), "home"),
            MobileState::Mobile => match *ShowRight::get() {
                ShowRight::None => (html!(), "home-mobile"),
                ShowRight::Show => (html!(<Right />), "home-mobile"),
            },
        };
        html! {
                <div {class} id="app">
                    <Left user_id={ctx.props().id.clone()}/>
                    {right}
                    // 通知组件
                    <NotificationCom />
                </div>
        }
    }
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
            // 防止页面刷新，导致全局变量重置后，db对象也被重置
            db::init_db().await;

            match db::db_ins().users.get(&clone_id).await {
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
                MobileState::Mobile.notify();
            } else {
                MobileState::Desktop.notify();
            }
        }

        ctx.link()
            .send_message(HomeMsg::Query(Box::new(QueryStatus::Querying)));

        let _theme_dis = Dispatch::global().subscribe(ctx.link().callback(HomeMsg::SwitchTheme));
        let _right_dis = Dispatch::global().subscribe(ctx.link().callback(|_| HomeMsg::ShowRight));
        let _font_size_dis =
            Dispatch::global().subscribe(ctx.link().callback(HomeMsg::SwitchFontSize));

        Self {
            _theme_dis,
            _font_size_dis,
            db_inited: false,
            _right_dis,
        }
    }
}
