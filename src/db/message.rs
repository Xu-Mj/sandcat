#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]

use std::ops::Deref;

use super::{
    repository::Repository, MESSAGE_FRIEND_ID_INDEX, MESSAGE_ID_INDEX, MESSAGE_TABLE_NAME,
};
use crate::model::message::Message;
use futures_channel::oneshot;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Event, IdbKeyRange, IdbRequest};
use yew::AttrValue;

pub struct MessageRepo(Repository);

impl Deref for MessageRepo {
    type Target = Repository;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MessageRepo {
    pub async fn new() -> Self {
        MessageRepo(Repository::new().await)
    }

    // todo 分页
    pub async fn get_last_msg(&self, friend_id: AttrValue) -> Result<Message, JsValue> {
        // 使用channel异步获取数据
        let (tx, rx) = oneshot::channel::<Message>();
        let store = self.store(&String::from(MESSAGE_TABLE_NAME)).await.unwrap();

        // let rang = IdbKeyRange::bound(&JsValue::from(0), &JsValue::from(100));
        let rang = IdbKeyRange::only(&JsValue::from(friend_id.as_str()));
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
    pub async fn get_messages(
        &self,
        friend_id: AttrValue,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<Message>, JsValue> {
        let mut counter = 0;
        let mut advanced = true;
        // 使用channel异步获取数据
        let (tx, rx) = oneshot::channel::<Vec<Message>>();
        let store = self.store(&String::from(MESSAGE_TABLE_NAME)).await.unwrap();
        let rang = IdbKeyRange::only(&JsValue::from(friend_id.as_str()));
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

        let mut messages = Vec::new();
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
                let msg: Message = serde_wasm_bindgen::from_value(value).unwrap();
                messages.push(msg);
                counter += 1;
                if counter >= page_size {
                    let _ = tx.take().unwrap().send(messages.to_owned());
                    return;
                }
                let _ = cursor.continue_();
            } else {
                // 如果为null说明已经遍历完成
                //将总的结果发送出来
                let _ = tx.take().unwrap().send(messages.to_owned());
            }
        }) as Box<dyn FnMut(&Event)>);

        request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
        success.forget();
        Ok(rx.await.unwrap())
    }

    pub async fn add_message(&self, msg: &mut Message) -> Result<(), JsValue> {
        let store = self.store(&String::from(MESSAGE_TABLE_NAME)).await.unwrap();
        let index = store.index(MESSAGE_ID_INDEX).unwrap();
        let (tx, rx) = oneshot::channel::<Option<Message>>();
        let req = index.get(&JsValue::from(msg.msg_id.as_str())).unwrap();
        let onsuccess = Closure::once(move |event: &Event| {
            let value = event
                .target()
                .unwrap()
                .dyn_ref::<IdbRequest>()
                .unwrap()
                .result()
                .unwrap();
            let mut result = Message::default();
            if !value.is_undefined() && !value.is_null() {
                result = serde_wasm_bindgen::from_value(value).unwrap();
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
}
