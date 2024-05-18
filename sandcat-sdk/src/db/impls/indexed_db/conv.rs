use std::ops::Deref;

use futures_channel::oneshot;
use indexmap::IndexMap;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Event, IdbRequest};
use yew::AttrValue;

use crate::model::conversation::Conversation;

use super::{repository::Repository, CONVERSATION_LAST_MSG_TIME_INDEX, CONVERSATION_TABLE_NAME};
use crate::db::conversations::Conversations;
#[derive(Debug, Clone)]
pub struct ConvRepo(Repository);

impl Deref for ConvRepo {
    type Target = Repository;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ConvRepo {
    pub fn new(repo: Repository) -> Self {
        ConvRepo(repo)
    }
}

#[async_trait::async_trait(?Send)]
impl Conversations for ConvRepo {
    async fn mute(&self, conv: &Conversation) -> Result<(), JsValue> {
        let store = self.store(&String::from(CONVERSATION_TABLE_NAME)).await?;
        store.put(&serde_wasm_bindgen::to_value(&conv).unwrap())?;
        Ok(())
    }
    // 使用put方法，不存在创建，存在则直接更新
    async fn put_conv(&self, conv: &Conversation) -> Result<(), JsValue> {
        let store = self.store(&String::from(CONVERSATION_TABLE_NAME)).await?;
        let value = serde_wasm_bindgen::to_value(&conv).unwrap();
        let request = store.put(&value).unwrap();
        let on_add_error = Closure::once(move |event: &Event| {
            web_sys::console::log_1(&String::from("put conv失败").into());
            web_sys::console::log_1(&event.into());
        });
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        on_add_error.forget();
        Ok(())
    }

    async fn self_update_conv(&self, mut conv: Conversation) -> Result<Conversation, JsValue> {
        let store = self.store(&String::from(CONVERSATION_TABLE_NAME)).await?;
        let request = store
            .get(&JsValue::from(conv.friend_id.as_str()))
            .expect("friend select get error");
        let store = store.clone();

        let (tx, rx) = oneshot::channel::<Conversation>();
        let onsuccess = Closure::once(move |event: &Event| {
            let result = event
                .target()
                .unwrap()
                .dyn_ref::<IdbRequest>()
                .unwrap()
                .result()
                .unwrap();
            if !result.is_undefined() && !result.is_null() {
                let conv1: Conversation = serde_wasm_bindgen::from_value(result).unwrap();
                conv.unread_count = conv1.unread_count;
            }
            let value = serde_wasm_bindgen::to_value(&conv).unwrap();
            store.put(&value).unwrap();
            tx.send(conv).unwrap();
        });
        request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        onsuccess.forget();
        let on_add_error = Closure::once(move |event: &Event| {
            web_sys::console::log_1(&String::from("put conv失败").into());
            web_sys::console::log_1(&event.into());
        });
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        on_add_error.forget();
        Ok(rx.await.unwrap())
    }

    async fn get_convs(&self) -> Result<IndexMap<AttrValue, Conversation>, JsValue> {
        let (tx, rx) = oneshot::channel::<IndexMap<AttrValue, Conversation>>();
        let store = self.store(&String::from(CONVERSATION_TABLE_NAME)).await?;
        let index = store.index(CONVERSATION_LAST_MSG_TIME_INDEX).unwrap();
        let request = index.open_cursor_with_range_and_direction(
            &JsValue::default(),
            web_sys::IdbCursorDirection::Prev,
        )?;
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
                    log::error!("query...:{:?}", _err);
                    JsValue::null()
                }
            };
            if !result.is_null() {
                let cursor = result
                    .dyn_ref::<web_sys::IdbCursorWithValue>()
                    .expect("result is IdbCursorWithValue; qed");
                let value = cursor.value().unwrap();
                // 反序列化
                if let Ok(conv) = serde_wasm_bindgen::from_value::<Conversation>(value) {
                    let id = conv.friend_id.clone();
                    convs.borrow_mut().insert(id, conv);
                }
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

    async fn get_by_frined_id(&self, friend_id: &str) -> Conversation {
        // 声明一个channel，接收查询结果
        let (tx, rx) = oneshot::channel::<Conversation>();
        let store = self.store(CONVERSATION_TABLE_NAME).await.unwrap();

        let request = store
            // .open_cursor_with_range(&key)
            .get(&JsValue::from(friend_id))
            .expect("friend select get error");
        let onsuccess = Closure::once(move |event: &Event| {
            let result = event
                .target()
                .unwrap()
                .dyn_ref::<IdbRequest>()
                .unwrap()
                .result()
                .unwrap();
            let mut conv = Conversation::default();
            if !result.is_undefined() && !result.is_null() {
                conv = serde_wasm_bindgen::from_value(result).unwrap();
            }
            tx.send(conv).unwrap();
        });
        request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        let on_add_error = Closure::once(move |event: &Event| {
            web_sys::console::log_1(&String::from("读取数据失败").into());
            web_sys::console::log_1(&event.into());
        });
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        rx.await.unwrap()
    }

    async fn delete(&self, friend_id: &str) -> Result<(), JsValue> {
        let store = self.store(&String::from(CONVERSATION_TABLE_NAME)).await?;
        store.delete(&JsValue::from(friend_id))?;
        Ok(())
    }
}
