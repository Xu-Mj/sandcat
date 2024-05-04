use futures_channel::oneshot;
use indexmap::IndexMap;
use std::ops::Deref;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Event, IdbCursorWithValue, IdbKeyRange, IdbRequest};
use yew::AttrValue;

use abi::model::friend::Friend;
use abi::model::message::Message;

use crate::friends::Friends;

use super::{repository::Repository, FRIEND_TABLE_NAME};
use super::{MESSAGE_FRIEND_ID_INDEX, MESSAGE_TABLE_NAME};

#[derive(Debug, Clone)]
pub struct FriendRepo(Repository);

impl Deref for FriendRepo {
    type Target = Repository;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FriendRepo {
    pub fn new(repo: Repository) -> Self {
        Self(repo)
    }
}
#[async_trait::async_trait(?Send)]
impl Friends for FriendRepo {
    async fn put_friend(&self, friend: &Friend) {
        let store = self.store(FRIEND_TABLE_NAME).await.unwrap();
        let value = serde_wasm_bindgen::to_value(friend).unwrap();
        store.put(&value).unwrap();
    }

    async fn put_friend_list(&self, friends: &[Friend]) {
        let store = self.store(FRIEND_TABLE_NAME).await.unwrap();
        friends.iter().for_each(|item| {
            let value = serde_wasm_bindgen::to_value(item).unwrap();
            store.put(&value).unwrap();
        });
    }

    #[allow(dead_code)]
    async fn get(&self, id: &str) -> Friend {
        // 声明一个channel，接收查询结果
        let (tx, rx) = oneshot::channel::<Friend>();
        let store = self.store(FRIEND_TABLE_NAME).await.unwrap();
        let request = store
            .get(&JsValue::from(id))
            .expect("friend select get error");
        let onsuccess = Closure::once(move |event: &Event| {
            let result = event
                .target()
                .unwrap()
                .dyn_ref::<IdbRequest>()
                .unwrap()
                .result()
                .unwrap();
            let mut friend = Friend::default();
            if !result.is_undefined() && !result.is_null() {
                friend = serde_wasm_bindgen::from_value(result).unwrap();
            }
            tx.send(friend).unwrap();
        });
        request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        let on_add_error = Closure::once(move |event: &Event| {
            web_sys::console::log_1(&String::from("读取数据失败").into());
            web_sys::console::log_1(&event.into());
        });
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        rx.await.unwrap()
    }

    async fn get_list(&self) -> Result<IndexMap<AttrValue, Friend>, JsValue> {
        let (tx, rx) = oneshot::channel::<IndexMap<AttrValue, Friend>>();
        let store = self.store(FRIEND_TABLE_NAME).await.unwrap();
        let request = store.open_cursor().unwrap();
        let on_add_error = Closure::once(move |event: &Event| {
            web_sys::console::log_1(&String::from("读取数据失败").into());
            web_sys::console::log_1(&event.into());
        });

        let convs = std::rc::Rc::new(std::cell::RefCell::new(IndexMap::new()));
        let convs = convs.clone();
        let mut tx = Some(tx);
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        let success = Closure::wrap(Box::new(move |event: &Event| {
            let target = event.target().expect("msg");
            let req = target
                .dyn_ref::<IdbRequest>()
                .expect("Event target is IdbRequest; qed");
            let result = match req.result() {
                Ok(data) => data,
                Err(_err) => {
                    log::error!("query friend list error ...:{:?}", _err);
                    JsValue::null()
                }
            };
            if !result.is_null() {
                let cursor = result
                    .dyn_ref::<web_sys::IdbCursorWithValue>()
                    .expect("result is IdbCursorWithValue; qed");
                let value = cursor.value().unwrap();
                // 反序列化
                let conv: Friend = serde_wasm_bindgen::from_value(value).unwrap();
                let id = conv.friend_id.clone();
                convs.borrow_mut().insert(id, conv);
                let _ = cursor.continue_();
            } else {
                // 如果为null说明已经遍历完成
                //将总的结果发送出来
                let _ = tx.take().unwrap().send(convs.borrow().clone());
            }
        }) as Box<dyn FnMut(&Event)>);

        request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
        success.forget();
        Ok(rx.await.unwrap())
    }

    /// delete friend by id; need to delete message data
    async fn delete_friend(&self, id: &str) -> Result<(), JsValue> {
        let store = self.store(FRIEND_TABLE_NAME).await.unwrap();
        store.delete(&JsValue::from(id))?;

        // delete message data
        let store = self.store(MESSAGE_TABLE_NAME).await.unwrap();
        let index = store.index(MESSAGE_FRIEND_ID_INDEX)?;
        let range = IdbKeyRange::only(&JsValue::from(id))?;
        let req = index.open_cursor_with_range(&range)?;
        let onsuccess = Closure::wrap(Box::new(move |event: &Event| {
            let req = event.target().unwrap().dyn_into::<IdbRequest>().unwrap();
            let result = req.result().unwrap_or_default();
            // default is Undefined
            if !result.is_undefined() && !result.is_null() {
                let cursor = result.dyn_ref::<IdbCursorWithValue>().unwrap();
                let value = cursor.value().unwrap();
                let msg: Message = serde_wasm_bindgen::from_value(value).unwrap();
                store.delete(&JsValue::from(msg.id)).unwrap();
                cursor.continue_().unwrap();
            }
        }) as Box<dyn FnMut(&Event)>);
        req.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        onsuccess.forget();
        Ok(())
    }
}
