use std::{cell::RefCell, ops::Deref, rc::Rc};

use futures_channel::oneshot;
use indexmap::IndexMap;
use log::error;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Event, IdbKeyRange, IdbRequest};
use yew::AttrValue;

use crate::db::conversations::Conversations;
use crate::error::Result;
use crate::model::conversation::Conversation;

use super::{
    repository::Repository, SuccessCallback, CONVERSATION_IS_PINED_WITH_TIME_INDEX,
    CONVERSATION_TABLE_NAME,
};

#[derive(Debug)]
pub struct ConvRepo {
    repo: Repository,
    on_err_callback: Closure<dyn FnMut(&Event)>,
    /// use `RefCell` that we can modify this attr through the `&self`
    on_update_success: SuccessCallback,
    get_conv_success: SuccessCallback,
}

impl Deref for ConvRepo {
    type Target = Repository;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl ConvRepo {
    pub fn new(repo: Repository) -> Self {
        let on_err_callback =
            Closure::once(move |event: &Event| error!("conversation operate error: {:?}", event));

        Self {
            repo,
            on_err_callback,
            on_update_success: Rc::new(RefCell::new(None)),
            get_conv_success: Rc::new(RefCell::new(None)),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl Conversations for ConvRepo {
    // 使用put方法，不存在创建，存在则直接更新
    async fn put_conv(&self, conv: &Conversation) -> Result<()> {
        let store = self.store(&String::from(CONVERSATION_TABLE_NAME)).await?;
        let value = serde_wasm_bindgen::to_value(&conv)?;
        let request = store.put(&value)?;

        request.set_onerror(Some(self.on_err_callback.as_ref().unchecked_ref()));
        Ok(())
    }

    async fn self_update_conv(&self, mut conv: Conversation) -> Result<Conversation> {
        let store = self.store(&String::from(CONVERSATION_TABLE_NAME)).await?;
        let request = store.get(&JsValue::from(conv.friend_id.as_str()))?;
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
        *self.on_update_success.borrow_mut() = Some(onsuccess);

        request.set_onerror(Some(self.on_err_callback.as_ref().unchecked_ref()));
        Ok(rx.await.unwrap())
    }

    async fn get_pined_convs(&self) -> Result<IndexMap<AttrValue, Conversation>> {
        let (tx, rx) = oneshot::channel::<IndexMap<AttrValue, Conversation>>();
        let store = self.store(&String::from(CONVERSATION_TABLE_NAME)).await?;
        let index = store.index(CONVERSATION_IS_PINED_WITH_TIME_INDEX)?;
        // let request = index.open_cursor_with_range_and_direction(
        //     &JsValue::default(),
        //     web_sys::IdbCursorDirection::Prev,
        // )?;

        let start_key = js_sys::Array::new();
        start_key.push(&JsValue::from(1));
        start_key.push(&JsValue::from_f64(f64::NEG_INFINITY));

        let end_key = js_sys::Array::new();
        end_key.push(&JsValue::from(1));
        end_key.push(&JsValue::from_f64(f64::INFINITY));

        let range = IdbKeyRange::bound(&JsValue::from(start_key), &JsValue::from(end_key))?;
        let request = index
            .open_cursor_with_range_and_direction(&range, web_sys::IdbCursorDirection::Prev)?;

        let convs = std::rc::Rc::new(std::cell::RefCell::new(IndexMap::new()));
        let convs = convs.clone();
        let mut tx = Some(tx);

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
        *self.get_conv_success.borrow_mut() = Some(success);

        request.set_onerror(Some(self.on_err_callback.as_ref().unchecked_ref()));

        Ok(rx.await.unwrap())
    }

    async fn get_convs(&self) -> Result<IndexMap<AttrValue, Conversation>> {
        let (tx, rx) = oneshot::channel::<IndexMap<AttrValue, Conversation>>();
        let store = self.store(&String::from(CONVERSATION_TABLE_NAME)).await?;
        let index = store.index(CONVERSATION_IS_PINED_WITH_TIME_INDEX)?;

        let start_key = js_sys::Array::new();
        start_key.push(&JsValue::from(0));
        start_key.push(&JsValue::from_f64(f64::NEG_INFINITY));

        let end_key = js_sys::Array::new();
        end_key.push(&JsValue::from(0));
        end_key.push(&JsValue::from_f64(f64::INFINITY));

        let range = IdbKeyRange::bound(&JsValue::from(start_key), &JsValue::from(end_key))?;
        let request = index
            .open_cursor_with_range_and_direction(&range, web_sys::IdbCursorDirection::Prev)?;

        let convs = std::rc::Rc::new(std::cell::RefCell::new(IndexMap::new()));
        let convs = convs.clone();
        let mut tx = Some(tx);

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
        *self.get_conv_success.borrow_mut() = Some(success);

        request.set_onerror(Some(self.on_err_callback.as_ref().unchecked_ref()));

        Ok(rx.await.unwrap())
    }

    async fn get_by_frined_id(&self, friend_id: &str) -> Result<Option<Conversation>> {
        // 声明一个channel，接收查询结果
        let (tx, rx) = oneshot::channel::<Option<Conversation>>();
        let store = self.store(CONVERSATION_TABLE_NAME).await?;

        let request = store.get(&JsValue::from(friend_id))?;

        let onsuccess = Closure::once(move |event: &Event| {
            let result = event
                .target()
                .unwrap()
                .dyn_ref::<IdbRequest>()
                .unwrap()
                .result()
                .unwrap();
            let mut conv = None;
            if !result.is_undefined() && !result.is_null() {
                conv = Some(serde_wasm_bindgen::from_value(result).unwrap());
            }
            tx.send(conv).unwrap();
        });

        request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        request.set_onerror(Some(self.on_err_callback.as_ref().unchecked_ref()));

        Ok(rx.await.unwrap())
    }

    async fn delete(&self, friend_id: &str) -> Result<()> {
        let store = self.store(&String::from(CONVERSATION_TABLE_NAME)).await?;
        store.delete(&JsValue::from(friend_id))?;
        Ok(())
    }
}
