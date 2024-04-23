use yew::{AttrValue, Context, NodeRef};
use yewdux::Dispatch;

use crate::db;
use crate::model::user::User;
use crate::{
    db::{QueryError, QueryStatus, DB_NAME},
    model::notification::{Notification, NotificationType},
    pages::home_page::HomeMsg,
};

use super::Home;

async fn query(id: &str) -> Result<User, QueryError> {
    let user_repo = db::users().await;
    let user = user_repo.get(id).await.unwrap();

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
            match query(clone_id.as_str()).await {
                Ok(data) => HomeMsg::Query(Box::new(QueryStatus::QuerySuccess(data))),
                Err(err) => HomeMsg::Query(Box::new(QueryStatus::QueryFail(err))),
            }
        });

        // 使用ctx发送一个正在查询的状态
        ctx.link()
            .send_message(HomeMsg::Query(Box::new(QueryStatus::Querying)));

        let noti_dis =
            Dispatch::global().subscribe(ctx.link().callback(HomeMsg::NotificationStateChanged));
        Self {
            notifications: vec![],
            _noti_dis: noti_dis,
            notification_node: NodeRef::default(),
            notification_interval: None,
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
