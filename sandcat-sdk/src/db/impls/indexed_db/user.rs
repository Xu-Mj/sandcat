use std::ops::Deref;

use futures_channel::oneshot;
use log::error;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Event, IdbRequest};

use crate::{error::Result, model::user::User};

use super::repository::Repository;
use crate::db::users::Users;

const USER_STORE_NAME: &str = "users";
// 用户仓库，增删改查
#[derive(Debug, Clone)]
pub struct UserRepo(Repository);

impl Deref for UserRepo {
    type Target = Repository;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// 增删查改
impl UserRepo {
    pub fn new(repo: Repository) -> Self {
        UserRepo(repo)
    }
}
#[async_trait::async_trait(?Send)]
impl Users for UserRepo {
    async fn add(&self, user: &User) /* -> Result<i32, String> */
    {
        let store = self.store(&String::from(USER_STORE_NAME)).await.unwrap();
        let add_request = store
            .put(&serde_wasm_bindgen::to_value(&user).unwrap())
            .unwrap();

        let on_add_error =
            Closure::once(move |event: &Event| error!("save user info error{:?}", event));
        add_request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
    }

    // 需要优化
    async fn get(&self, id: &str) -> Result<User> {
        let (tx, rx) = oneshot::channel::<User>();

        let store = self.store(&String::from(USER_STORE_NAME)).await?;
        let request = store.get(&JsValue::from(id))?;
        let on_add_error =
            Closure::once(move |event: &Event| error!("read user info error{:?}", event));
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));

        let on_success = Closure::once(move |event: &Event| {
            let target = event.target().expect("msg");
            let req = target
                .dyn_ref::<IdbRequest>()
                .expect("Event target is IdbRequest; qed");
            let result = req.result().expect("read result error");
            let mut user = User::default();
            if !result.is_undefined() && !result.is_null() {
                user = serde_wasm_bindgen::from_value(result).expect("反序列化出错");
            }
            let _ = tx.send(user);
        });
        request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));

        let user = rx.await.unwrap();
        Ok(user)
    }
}
