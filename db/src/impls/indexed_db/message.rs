use std::ops::Deref;

use futures_channel::oneshot;
use indexmap::IndexMap;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Event, IdbKeyRange, IdbRequest};
use yew::AttrValue;

use abi::model::message::{Message, ServerResponse};

use crate::messages::Messages;

use super::{
    repository::Repository, MESSAGE_FRIEND_ID_INDEX, MESSAGE_ID_INDEX, MESSAGE_IS_READ_INDEX,
    MESSAGE_TABLE_NAME,
};

#[derive(Debug, Clone)]
pub struct MessageRepo(Repository);

impl Deref for MessageRepo {
    type Target = Repository;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MessageRepo {
    pub fn new(repo: Repository) -> Self {
        MessageRepo(repo)
    }
}

#[async_trait::async_trait(?Send)]
impl Messages for MessageRepo {
    async fn get_last_msg(&self, friend_id: &str) -> Result<Message, JsValue> {
        // 使用channel异步获取数据
        let (tx, rx) = oneshot::channel::<Message>();
        let store = self.store(MESSAGE_TABLE_NAME).await.unwrap();

        // let rang = IdbKeyRange::bound(&JsValue::from(0), &JsValue::from(100));
        let rang = IdbKeyRange::only(&JsValue::from(friend_id));
        let index = store.index(MESSAGE_FRIEND_ID_INDEX).unwrap();

        let request = index
            .open_cursor_with_range_and_direction(
                &JsValue::from(&rang.unwrap()),
                web_sys::IdbCursorDirection::Prev,
            )
            .unwrap();
        let on_add_error = Closure::once(move |event: &Event| {
            web_sys::console::log_1(&String::from("读取数据失败").into());
            web_sys::console::log_1(&event.into());
        });

        let mut tx = Some(tx);
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
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

                let value = cursor.value().unwrap();
                // 反序列化
                let msg: Message = serde_wasm_bindgen::from_value(value).unwrap();

                let _ = tx.take().unwrap().send(msg);
            } else {
                let _ = tx.take().unwrap().send(Message::default());
            }
        }) as Box<dyn FnMut(&Event)>);

        request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
        success.forget();
        Ok(rx.await.unwrap())
    }

    async fn get_messages(
        &self,
        friend_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<IndexMap<AttrValue, Message>, JsValue> {
        let mut counter = 0;
        let mut advanced = true;
        // 使用channel异步获取数据
        let (tx, rx) = oneshot::channel::<IndexMap<AttrValue, Message>>();
        // let (tx, rx) = oneshot::channel::<Vec<Message>>();
        let store = self.store(MESSAGE_TABLE_NAME).await.unwrap();
        let rang = IdbKeyRange::only(&JsValue::from(friend_id));
        let index = store.index(MESSAGE_FRIEND_ID_INDEX).unwrap();
        let request = index
            .open_cursor_with_range_and_direction(
                &JsValue::from(&rang.unwrap()),
                web_sys::IdbCursorDirection::Prev,
            )
            .unwrap();
        let on_add_error = Closure::once(move |event: &Event| {
            web_sys::console::log_1(&String::from("读取数据失败").into());
            web_sys::console::log_1(&event.into());
        });

        let messages = std::rc::Rc::new(std::cell::RefCell::new(IndexMap::new()));
        let messages = messages.clone();
        let mut tx = Some(tx);
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
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
        success.forget();
        Ok(rx.await.unwrap())
    }

    async fn add_message(&self, msg: &mut Message) -> Result<(), JsValue> {
        let store = self.store(MESSAGE_TABLE_NAME).await.unwrap();
        let index = store.index(MESSAGE_ID_INDEX).unwrap();
        let (tx, rx) = oneshot::channel::<Option<Message>>();
        let req = index.get(&JsValue::from(msg.local_id.as_str())).unwrap();
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
        // todo添加错误处理
        let request = store
            .put(&serde_wasm_bindgen::to_value(&msg).unwrap())
            .unwrap();
        let on_add_error = Closure::once(move |event: &Event| {
            web_sys::console::log_1(&String::from("消息存储失败").into());
            web_sys::console::log_1(&event.into());
        });
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        on_add_error.forget();
        Ok(())
    }

    async fn update_msg_status(&self, msg: &ServerResponse) -> Result<(), JsValue> {
        let store = self.store(MESSAGE_TABLE_NAME).await.unwrap();
        let index = store.index(MESSAGE_ID_INDEX).unwrap();
        let req = index.get(&JsValue::from(msg.local_id.as_str()))?;
        let store = store.clone();
        let send_status = msg.send_status.clone();
        let server_id = msg.server_id.clone();
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
                store
                    .put(&serde_wasm_bindgen::to_value(&result).unwrap())
                    .unwrap();
            }
        });
        req.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        onsuccess.forget();
        let on_add_error = Closure::once(move |event: &Event| {
            web_sys::console::log_1(&String::from("消息存储失败").into());
            web_sys::console::log_1(&event.into());
        });
        req.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        on_add_error.forget();
        Ok(())
    }

    async fn update_read_status(&self, friend_id: &str) -> Result<(), JsValue> {
        let store = self.store(MESSAGE_TABLE_NAME).await.unwrap();
        let rang = IdbKeyRange::only(&JsValue::from(friend_id));
        let index = store.index(MESSAGE_FRIEND_ID_INDEX).unwrap();
        let request = index
            .open_cursor_with_range_and_direction(
                &JsValue::from(&rang.unwrap()),
                web_sys::IdbCursorDirection::Prev,
            )
            .unwrap();
        let on_add_error = Closure::once(move |event: &Event| {
            web_sys::console::log_1(&String::from("读取数据失败").into());
            web_sys::console::log_1(&event.into());
        });

        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));

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
        success.forget();
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
        onsuccess.forget();
        rx.await.unwrap_or_default()
    }

    async fn batch_delete(&self, friend_id: &str) -> Result<(), JsValue> {
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
        onsuccess.forget();
        Ok(())
    }
}
