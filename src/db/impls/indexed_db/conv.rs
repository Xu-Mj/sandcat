use std::ops::Deref;

use crate::model::{conversation::Conversation, ContentType};

use futures_channel::oneshot;
use indexmap::IndexMap;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Event, IdbRequest};
use yew::AttrValue;

use super::{
    repository::Repository, CONVERSATION_FRIEND_ID_INDEX, CONVERSATION_LAST_MSG_TIME_INDEX,
    CONVERSATION_TABLE_NAME,
};

pub struct ConvRepo(Repository);

impl Deref for ConvRepo {
    type Target = Repository;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl ConvRepo {
    pub async fn new() -> Self {
        ConvRepo(Repository::new().await)
    }

    pub async fn mute(&self, conv: &Conversation) -> Result<(), JsValue> {
        let store = self.store(&String::from(CONVERSATION_TABLE_NAME)).await?;
        store.put(&serde_wasm_bindgen::to_value(conv).unwrap())?;
        Ok(())
    }
    // 使用put方法，不存在创建，存在则直接更新
    pub async fn put_conv(
        &self,
        conv: &Conversation,
        is_clean_unread_count: bool,
    ) -> Result<Conversation, JsValue> {
        let store = self
            .store(&String::from(CONVERSATION_TABLE_NAME))
            .await
            .unwrap();
        let index = store.index(CONVERSATION_FRIEND_ID_INDEX).unwrap();
        let (tx, rx) = oneshot::channel::<Conversation>();
        let req = index.get(&JsValue::from(conv.friend_id.as_str())).unwrap();
        let onsuccess = Closure::once(move |event: &Event| {
            let value = event
                .target()
                .unwrap()
                .dyn_ref::<IdbRequest>()
                .unwrap()
                .result()
                .unwrap();
            let mut result = Conversation::default();
            if !value.is_undefined() && !value.is_null() {
                result = serde_wasm_bindgen::from_value(value).unwrap();
            }
            tx.send(result).unwrap();
        });
        req.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        let mut result = rx.await.unwrap();
        if !conv.avatar.is_empty() {
            result.avatar = conv.avatar.clone();
        }
        if !conv.name.is_empty() {
            result.name = conv.name.clone();
        }
        if !conv.last_msg.is_empty() {
            result.last_msg = conv.last_msg.clone();
        }
        if conv.last_msg_time != 0 {
            result.last_msg_time = conv.last_msg_time;
        }
        if conv.last_msg_type != ContentType::Default {
            result.last_msg_type = conv.last_msg_type;
        }
        if is_clean_unread_count {
            result.unread_count = 0;
        } else {
            result.unread_count += 1;
        }
        result.conv_type = conv.conv_type.clone();
        result.friend_id = conv.friend_id.clone();

        let value = serde_wasm_bindgen::to_value(&result).unwrap();
        // 添加成功失败回调
        let request = store.put(&value).unwrap();
        let on_add_error = Closure::once(move |event: &Event| {
            web_sys::console::log_1(&String::from("put conv失败").into());
            web_sys::console::log_1(&event.into());
        });
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        on_add_error.forget();
        Ok(result)
    }

    pub async fn get_convs2(&self) -> Result<IndexMap<AttrValue, Conversation>, JsValue> {
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

    pub async fn get_by_frined_id(&self, friend_id: AttrValue) -> Conversation {
        // 声明一个channel，接收查询结果
        let (tx, rx) = oneshot::channel::<Conversation>();
        let store = self.store(CONVERSATION_TABLE_NAME).await.unwrap();
        let index = store
            .index(CONVERSATION_FRIEND_ID_INDEX)
            .expect("friend select index error");
        // let key = IdbKeyRange::only(&JsValue::from(friend_id.as_str())).unwrap();
        let request = index
            // .open_cursor_with_range(&key)
            .get(&JsValue::from(friend_id.as_str()))
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

    pub async fn delete(&self, friend_id: AttrValue) -> Result<(), JsValue> {
        let conv = self.get_by_frined_id(friend_id).await;
        let store = self.store(&String::from(CONVERSATION_TABLE_NAME)).await?;
        if conv.id > 0 {
            store.delete(&JsValue::from(conv.id))?;
        }
        Ok(())
    }
}
