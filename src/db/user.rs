use super::repository::Repository;
use crate::model::user::User;
use futures_channel::oneshot;
use std::ops::Deref;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{console, Event, IdbRequest};
use yew::AttrValue;

const USER_STORE_NAME: &str = "users";
// 用户仓库，增删改查
pub struct UserRepo(Repository);

impl Deref for UserRepo {
    type Target = Repository;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// 增删查改
impl UserRepo {
    pub async fn new() -> Self {
        UserRepo(Repository::new().await)
    }

    pub async fn add(&self, user: &User) /* -> Result<i32, String> */
    {
        let store = self.store(&String::from(USER_STORE_NAME)).await.unwrap();
        let add_request = store
            .put(&serde_wasm_bindgen::to_value(&user).unwrap())
            .unwrap();

        let on_add_error = Closure::once(move |event: &Event| {
            console::log_1(&String::from("写入数据失败").into());
            console::log_1(&event.into());
        });
        add_request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        on_add_error.forget();

        let on_add_success = Closure::once(move |_event: &Event| {
            console::log_1(&String::from("写入数据成功").into());
        });
        add_request.set_onsuccess(Some(on_add_success.as_ref().unchecked_ref()));
        on_add_success.forget();
    }

    // 需要优化
    pub async fn get(&self, id: AttrValue) -> Result<User, JsValue> {
        let (tx, rx) = oneshot::channel::<User>();

        let store = self.store(&String::from(USER_STORE_NAME)).await.unwrap();
        let request = store
            .get(&JsValue::from(id.as_str()))
            .expect("get all error");
        let on_add_error = Closure::once(move |event: &Event| {
            console::log_1(&String::from("读取数据失败").into());
            console::log_1(&event.into());
        });
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        on_add_error.forget();

        let on_success = Closure::once(move |event: &Event| {
            let target = event.target().expect("msg");
            let req = target
                .dyn_ref::<IdbRequest>()
                .expect("Event target is IdbRequest; qed");
            let result = req.result().expect("read result error");
            let mut user = User::default();
            if !result.is_undefined() && !result.is_null() {
                console::log_1(&String::from("读取数据成功").into());
                user = serde_wasm_bindgen::from_value(result).expect("反序列化出错");
            }
            let _ = tx.send(user);
        });
        request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        on_success.forget();

        let user = rx.await.unwrap();
        Ok(user)
    }

    // pub fn delete(&self, id: i32) -> Result<i32, String> {}
}
