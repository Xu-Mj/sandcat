use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use futures_channel::oneshot;
use indexmap::IndexMap;
use log::error;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Event, IdbKeyRange, IdbRequest};
use yew::AttrValue;

use crate::db::messages::Messages;
use crate::error::Result;
use crate::model::message::{Message, ServerResponse};

use super::{
    repository::Repository, MESSAGE_FRIEND_ID_INDEX, MESSAGE_ID_INDEX, MESSAGE_IS_READ_INDEX,
    MESSAGE_TABLE_NAME,
};
use super::{SuccessCallback, MESSAGE_FRIEND_AND_SEND_TIME_INDEX};

#[derive(Debug)]
pub struct MessageRepo {
    repo: Repository,
    on_err_callback: Closure<dyn FnMut(&Event)>,
    on_get_list_success: SuccessCallback,
    on_update_success: SuccessCallback,
    on_del_msg_success: SuccessCallback,
    on_update_state_success: SuccessCallback,
}

impl Deref for MessageRepo {
    type Target = Repository;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl MessageRepo {
    pub fn new(repo: Repository) -> Self {
        let on_err_callback =
            Closure::once(move |event: &Event| error!("group operate error: {:?}", event));

        Self {
            repo,
            on_err_callback,
            on_get_list_success: Rc::new(RefCell::new(None)),
            on_update_success: Rc::new(RefCell::new(None)),
            on_del_msg_success: Rc::new(RefCell::new(None)),
            on_update_state_success: Rc::new(RefCell::new(None)),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl Messages for MessageRepo {
    async fn get_last_msg(&self, friend_id: &str) -> Result<Message> {
        // 使用channel异步获取数据
        let (tx, rx) = oneshot::channel::<Message>();
        let store = self.store(MESSAGE_TABLE_NAME).await?;

        // let rang = IdbKeyRange::bound(&JsValue::from(0), &JsValue::from(100));
        let rang = IdbKeyRange::only(&JsValue::from(friend_id))?;
        let index = store.index(MESSAGE_FRIEND_ID_INDEX)?;

        let request = index.open_cursor_with_range_and_direction(
            &JsValue::from(&rang),
            web_sys::IdbCursorDirection::Prev,
        )?;

        let mut tx = Some(tx);
        request.set_onerror(Some(self.on_err_callback.as_ref().unchecked_ref()));

        let success = Closure::once(Box::new(move |event: &Event| {
            let target = event.target().expect("msg");
            let req = target
                .dyn_ref::<IdbRequest>()
                .expect("Event target is IdbRequest; qed");
            let result = req.result().unwrap_or_else(|_err| JsValue::null());

            if !result.is_null() {
                let cursor = result
                    .dyn_ref::<web_sys::IdbCursorWithValue>()
                    .expect("result is IdbCursorWithValue; qed");

                let value = cursor.value().unwrap();
                // 反序列化
                let msg: Message = serde_wasm_bindgen::from_value(value).unwrap();

                let _ = tx.take().unwrap().send(msg);
            } else {
                let _ = tx.take().unwrap().send(Message::default());
            }
        }) as Box<dyn FnMut(&Event)>);

        request.set_onsuccess(Some(success.as_ref().unchecked_ref()));

        Ok(rx.await.unwrap())
    }

    async fn get_msg_by_local_id(&self, local_id: &str) -> Result<Option<Message>> {
        let (tx, rx) = oneshot::channel::<Option<Message>>();
        let store = self.store(MESSAGE_TABLE_NAME).await?;
        let index = store.index(MESSAGE_ID_INDEX)?;
        let request = index.get(&JsValue::from(local_id))?;
        let onsuccess = Closure::once(move |event: &Event| {
            let result = event
                .target()
                .unwrap()
                .dyn_ref::<IdbRequest>()
                .unwrap()
                .result()
                .unwrap();
            let mut msg = None;
            if !result.is_undefined() && !result.is_null() {
                msg = Some(serde_wasm_bindgen::from_value(result).unwrap());
            }
            tx.send(msg).unwrap();
        });
        request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        let on_add_error = Closure::once(move |event: &Event| error!("query error: {:?}", event));
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        Ok(rx.await.unwrap())
    }

    async fn get_messages(
        &self,
        friend_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<IndexMap<AttrValue, Message>> {
        let mut counter = 0;
        let mut advanced = true;
        // 使用channel异步获取数据
        let (tx, rx) = oneshot::channel::<IndexMap<AttrValue, Message>>();
        // let (tx, rx) = oneshot::channel::<Vec<Message>>();
        let store = self.store(MESSAGE_TABLE_NAME).await?;

        let index = store.index(MESSAGE_FRIEND_AND_SEND_TIME_INDEX)?;

        let start_key = js_sys::Array::new();
        start_key.push(&JsValue::from(friend_id));
        start_key.push(&JsValue::from_f64(f64::NEG_INFINITY));

        let end_key = js_sys::Array::new();
        end_key.push(&JsValue::from(friend_id));
        end_key.push(&JsValue::from_f64(f64::INFINITY));

        let range = IdbKeyRange::bound(&JsValue::from(start_key), &JsValue::from(end_key))?;
        let request = index
            .open_cursor_with_range_and_direction(&range, web_sys::IdbCursorDirection::Prev)?;

        let messages = std::rc::Rc::new(std::cell::RefCell::new(IndexMap::new()));
        let messages = messages.clone();
        let mut tx = Some(tx);
        request.set_onerror(Some(self.on_err_callback.as_ref().unchecked_ref()));

        let success = Closure::wrap(Box::new(move |event: &Event| {
            let target = event.target().expect("msg");
            let req = target
                .dyn_ref::<IdbRequest>()
                .expect("Event target is IdbRequest; qed");
            let result = req.result().unwrap_or(JsValue::null());

            if !result.is_null() {
                let cursor = result
                    .dyn_ref::<web_sys::IdbCursorWithValue>()
                    .expect("result is IdbCursorWithValue; qed");
                if page > 1 && advanced {
                    advanced = false;
                    cursor.advance((page - 1) * page_size).unwrap();
                    return;
                }
                let value = cursor.value().unwrap();
                // 反序列化
                if let Ok(msg) = serde_wasm_bindgen::from_value::<Message>(value) {
                    let id = msg.local_id.clone();
                    messages.borrow_mut().insert(id, msg);
                }
                counter += 1;
                if counter >= page_size {
                    let _ = tx.take().unwrap().send(messages.borrow().clone());
                    return;
                }
                let _ = cursor.continue_();
            } else {
                // 如果为null说明已经遍历完成
                //将总的结果发送出来
                let _ = tx.take().unwrap().send(messages.borrow().clone());
            }
        }) as Box<dyn FnMut(&Event)>);

        request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
        *self.on_get_list_success.borrow_mut() = Some(success);
        Ok(rx.await.unwrap())
    }

    /// todo test if set local id to unique, and put a message with same local id
    async fn add_message(&self, msg: &mut Message) -> Result<()> {
        let store = self.store(MESSAGE_TABLE_NAME).await?;
        let index = store.index(MESSAGE_ID_INDEX)?;
        let (tx, rx) = oneshot::channel::<Option<Message>>();
        let req = index.get(&JsValue::from(msg.local_id.as_str()))?;

        let onsuccess = Closure::once(move |event: &Event| {
            let value = event
                .target()
                .unwrap()
                .dyn_ref::<IdbRequest>()
                .unwrap()
                .result()
                .unwrap();
            if !value.is_undefined() && !value.is_null() {
                let result = serde_wasm_bindgen::from_value(value).unwrap();
                tx.send(Some(result)).unwrap();
            } else {
                tx.send(None).unwrap();
            }
        });
        req.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));

        let result = rx.await.unwrap();
        msg.id = if let Some(msg) = result { msg.id } else { 0 };

        let request = store.put(&serde_wasm_bindgen::to_value(&msg)?)?;
        request.set_onerror(Some(self.on_err_callback.as_ref().unchecked_ref()));
        Ok(())
    }

    async fn update_msg_status(&self, msg: &ServerResponse) -> Result<()> {
        let store = self.store(MESSAGE_TABLE_NAME).await?;
        let index = store.index(MESSAGE_ID_INDEX)?;
        let req = index.get(&JsValue::from(msg.local_id.as_str()))?;

        let store = store.clone();
        let send_status = msg.send_status.clone();
        let server_id = msg.server_id.clone();
        let send_time = msg.send_time;

        let onsuccess = Closure::once(move |event: &Event| {
            let value = event
                .target()
                .unwrap()
                .dyn_ref::<IdbRequest>()
                .unwrap()
                .result()
                .unwrap();
            if !value.is_undefined() && !value.is_null() {
                let mut result: Message = serde_wasm_bindgen::from_value(value).unwrap();
                result.send_status = send_status;
                result.server_id = server_id;
                result.send_time = send_time;

                store
                    .put(&serde_wasm_bindgen::to_value(&result).unwrap())
                    .unwrap();
            }
        });

        req.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        req.set_onerror(Some(self.on_err_callback.as_ref().unchecked_ref()));

        *self.on_update_state_success.borrow_mut() = Some(onsuccess);
        Ok(())
    }

    async fn update_read_status(&self, friend_id: &str) -> Result<()> {
        let store = self.store(MESSAGE_TABLE_NAME).await?;
        let rang = IdbKeyRange::only(&JsValue::from(friend_id))?;
        let index = store.index(MESSAGE_FRIEND_ID_INDEX)?;
        let request = index.open_cursor_with_range_and_direction(
            &JsValue::from(&rang),
            web_sys::IdbCursorDirection::Prev,
        )?;

        request.set_onerror(Some(self.on_err_callback.as_ref().unchecked_ref()));

        let (tx, rx) = oneshot::channel::<()>();
        let mut tx = Some(tx);
        let store = store.clone();

        let success = Closure::wrap(Box::new(move |event: &Event| {
            let target = event.target().expect("msg");
            let req = target
                .dyn_ref::<IdbRequest>()
                .expect("Event target is IdbRequest; qed");
            let result = req.result().unwrap_or_else(|_err| JsValue::null());

            if !result.is_null() {
                let cursor = result
                    .dyn_ref::<web_sys::IdbCursorWithValue>()
                    .expect("result is IdbCursorWithValue; qed");
                if let Ok(value) = cursor.value() {
                    // 反序列化
                    if let Ok(mut msg) = serde_wasm_bindgen::from_value::<Message>(value) {
                        msg.is_read = 1;
                        store
                            .put(&serde_wasm_bindgen::to_value(&msg).unwrap())
                            .unwrap();
                    }
                }
                let _ = cursor.continue_();
            } else {
                tx.take().unwrap().send(()).unwrap();
            }
        }) as Box<dyn FnMut(&Event)>);

        request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
        *self.on_update_success.borrow_mut() = Some(success);

        rx.await.unwrap();
        Ok(())
    }

    async fn unread_count(&self) -> usize {
        let store = self.store(MESSAGE_TABLE_NAME).await.unwrap();
        let index = store.index(MESSAGE_IS_READ_INDEX).unwrap();
        let request = index.count_with_key(&JsValue::from(0)).unwrap();
        let (tx, rx) = futures_channel::oneshot::channel();
        let onsuccess = Closure::once(move |event: &Event| {
            let value = event
                .target()
                .unwrap()
                .dyn_ref::<IdbRequest>()
                .unwrap()
                .result()
                .unwrap();
            if !value.is_undefined() && !value.is_null() {
                let result = value.as_f64().unwrap() as usize;
                tx.send(result).unwrap();
            } else {
                tx.send(0).unwrap();
            }
        });
        request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        rx.await.unwrap_or_default()
    }

    async fn batch_delete(&self, friend_id: &str) -> Result<()> {
        let store = self.store(MESSAGE_TABLE_NAME).await?;
        let index = store.index(MESSAGE_FRIEND_ID_INDEX)?;
        let range = IdbKeyRange::only(&JsValue::from(friend_id))?;
        let request = index.open_cursor_with_range(&range)?;
        let store = store.clone();

        let onsuccess = Closure::wrap(Box::new(move |event: &Event| {
            let target = event.target().expect("msg");
            let req = target
                .dyn_ref::<IdbRequest>()
                .expect("Event target is IdbRequest; qed");
            let result = req.result().unwrap_or_else(|_err| JsValue::null());

            if !result.is_null() {
                let cursor = result
                    .dyn_ref::<web_sys::IdbCursorWithValue>()
                    .expect("result is IdbCursorWithValue; qed");
                let value = cursor.value().unwrap();
                // 反序列化
                if let Ok(msg) = serde_wasm_bindgen::from_value::<Message>(value) {
                    store.delete(&JsValue::from(msg.id)).unwrap();
                }
                let _ = cursor.continue_();
            }
        }) as Box<dyn FnMut(&Event)>);
        request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        *self.on_del_msg_success.borrow_mut() = Some(onsuccess);

        Ok(())
    }

    async fn delete(&self, local_id: i32) -> Result<()> {
        let store = self.store(MESSAGE_TABLE_NAME).await?;
        store.delete(&JsValue::from(local_id))?;
        Ok(())
    }
}
