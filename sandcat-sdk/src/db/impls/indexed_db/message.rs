use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use futures_channel::oneshot;
use indexmap::IndexMap;
use log::error;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Event, IdbKeyRange, IdbObjectStore, IdbRequest};
use yew::AttrValue;

use crate::db::messages::Messages;
use crate::error::Result;
use crate::model::message::{Message, ServerResponse};

use super::{
    repository::Repository, SuccessCallback, MESSAGE_FRIEND_AND_IS_READ_INDEX,
    MESSAGE_FRIEND_AND_SEND_TIME_INDEX, MESSAGE_FRIEND_ID_INDEX, MESSAGE_IS_READ_INDEX,
    MESSAGE_TABLE_NAME,
};

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
    async fn get_last_msg(&self, friend_id: &str) -> Result<Option<Message>> {
        let store = self.store(MESSAGE_TABLE_NAME).await?;
        get_last_msg(store, friend_id, &self.on_err_callback).await
    }

    async fn get(&self, local_id: &str) -> Result<Option<Message>> {
        let store = self.store(MESSAGE_TABLE_NAME).await?;
        get(store, local_id).await
    }

    async fn get_messages(
        &self,
        friend_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<IndexMap<AttrValue, Message>> {
        let store = self.store(MESSAGE_TABLE_NAME).await?;
        let (result, onsuccess) =
            get_messages(store, friend_id, page, page_size, &self.on_err_callback).await?;
        *self.on_get_list_success.borrow_mut() = Some(onsuccess);
        Ok(result)
    }

    async fn add_message(&self, msg: &Message) -> Result<()> {
        let store = self.store(MESSAGE_TABLE_NAME).await?;
        add(store, msg, &self.on_err_callback).await
    }

    async fn update_msg_status(&self, msg: &ServerResponse) -> Result<()> {
        let store = self.store(MESSAGE_TABLE_NAME).await?;

        let onsuccess = update_msg_status(store, msg, &self.on_err_callback).await?;
        *self.on_update_state_success.borrow_mut() = Some(onsuccess);
        Ok(())
    }

    async fn update_read_status(&self, friend_id: &str) -> Result<Vec<i64>> {
        let store = self.store(MESSAGE_TABLE_NAME).await?;

        let (seq, success) = update_read_status(store, friend_id, &self.on_err_callback).await?;
        *self.on_update_success.borrow_mut() = Some(success);

        Ok(seq)
    }

    async fn unread_count(&self) -> usize {
        let store = self.store(MESSAGE_TABLE_NAME).await.unwrap();
        unread_count(store).await.unwrap_or_default()
    }

    async fn delete_batch(&self, friend_id: &str) -> Result<()> {
        let store = self.store(MESSAGE_TABLE_NAME).await?;
        let onsuccess = delete_batch(store, friend_id).await?;
        *self.on_del_msg_success.borrow_mut() = Some(onsuccess);

        Ok(())
    }

    async fn delete(&self, local_id: &AttrValue) -> Result<()> {
        let store = self.store(MESSAGE_TABLE_NAME).await?;
        store.delete(&JsValue::from(local_id.as_str()))?;
        Ok(())
    }
}

pub(super) async fn get(store: IdbObjectStore, local_id: &str) -> Result<Option<Message>> {
    let request = store.get(&JsValue::from(local_id))?;

    let (tx, rx) = oneshot::channel::<Option<Message>>();

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
pub(super) async fn get_last_msg(
    store: IdbObjectStore,
    friend_id: &str,
    on_err_callback: &Closure<dyn FnMut(&Event)>,
) -> Result<Option<Message>> {
    // use channel to get the result
    let (tx, rx) = oneshot::channel::<Option<Message>>();

    let rang = IdbKeyRange::only(&JsValue::from(friend_id))?;
    let index = store.index(MESSAGE_FRIEND_ID_INDEX)?;

    let request = index.open_cursor_with_range_and_direction(
        &JsValue::from(&rang),
        web_sys::IdbCursorDirection::Prev,
    )?;

    let mut tx = Some(tx);
    request.set_onerror(Some(on_err_callback.as_ref().unchecked_ref()));

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

            let msg: Message = serde_wasm_bindgen::from_value(value).unwrap();

            let _ = tx.take().unwrap().send(Some(msg));
        } else {
            let _ = tx.take().unwrap().send(None);
        }
    }) as Box<dyn FnMut(&Event)>);

    request.set_onsuccess(Some(success.as_ref().unchecked_ref()));

    Ok(rx.await.unwrap())
}

pub(super) async fn get_messages(
    store: IdbObjectStore,
    friend_id: &str,
    page: u32,
    page_size: u32,
    on_err_callback: &Closure<dyn FnMut(&Event)>,
) -> Result<(IndexMap<AttrValue, Message>, Closure<dyn FnMut(&Event)>)> {
    let mut counter = 0;
    let mut advanced = true;

    // use channel to get the result
    let (tx, rx) = oneshot::channel::<IndexMap<AttrValue, Message>>();

    let index = store.index(MESSAGE_FRIEND_AND_SEND_TIME_INDEX)?;

    let start_key = js_sys::Array::new();
    start_key.push(&JsValue::from(friend_id));
    start_key.push(&JsValue::from_f64(f64::NEG_INFINITY));

    let end_key = js_sys::Array::new();
    end_key.push(&JsValue::from(friend_id));
    end_key.push(&JsValue::from_f64(f64::INFINITY));

    let range = IdbKeyRange::bound(&JsValue::from(start_key), &JsValue::from(end_key))?;
    let request =
        index.open_cursor_with_range_and_direction(&range, web_sys::IdbCursorDirection::Prev)?;

    let messages = Rc::new(RefCell::new(IndexMap::new()));
    let messages = messages.clone();
    let mut tx = Some(tx);
    request.set_onerror(Some(on_err_callback.as_ref().unchecked_ref()));

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
            // loop is complete if result is null
            // use the channel to send the messages
            let _ = tx.take().unwrap().send(messages.borrow().clone());
        }
    }) as Box<dyn FnMut(&Event)>);

    request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
    Ok((rx.await.unwrap(), success))
}

pub(super) async fn add(
    store: IdbObjectStore,
    msg: &Message,
    on_err_callback: &Closure<dyn FnMut(&Event)>,
) -> Result<()> {
    let request = store.put(&serde_wasm_bindgen::to_value(&msg)?)?;

    let (tx, rx) = oneshot::channel::<u8>();

    let onsuccess = Closure::once(move |_event: &Event| {
        tx.send(1).unwrap();
    });
    request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));

    request.set_onerror(Some(on_err_callback.as_ref().unchecked_ref()));

    // wait for the result
    rx.await.unwrap();

    Ok(())
}

pub(super) async fn update_msg_status(
    store: IdbObjectStore,
    msg: &ServerResponse,
    on_err_callback: &Closure<dyn FnMut(&Event)>,
) -> Result<Closure<dyn FnMut(&Event)>> {
    let req = store.get(&JsValue::from(msg.local_id.as_str()))?;

    let store = store.clone();
    let send_status = msg.send_status.clone();
    let server_id = msg.server_id.clone();
    let send_time = msg.send_time;
    let send_seq = msg.send_seq;

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
            result.send_seq = send_seq;

            store
                .put(&serde_wasm_bindgen::to_value(&result).unwrap())
                .unwrap();
        }
    });

    req.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
    req.set_onerror(Some(on_err_callback.as_ref().unchecked_ref()));

    Ok(onsuccess)
}

pub(super) async fn update_read_status(
    store: IdbObjectStore,
    friend_id: &str,
    on_err_callback: &Closure<dyn FnMut(&Event)>,
) -> Result<(Vec<i64>, Closure<dyn FnMut(&Event)>)> {
    let index = store.index(MESSAGE_FRIEND_AND_IS_READ_INDEX)?;

    let friend_unread = js_sys::Array::new();
    friend_unread.push(&JsValue::from(friend_id));
    friend_unread.push(&JsValue::from(0));

    let range = IdbKeyRange::only(&JsValue::from(friend_unread))?;
    let request = index.open_cursor_with_range(&range)?;

    request.set_onerror(Some(on_err_callback.as_ref().unchecked_ref()));

    let sequences = Rc::new(RefCell::new(Vec::new()));
    let sequences = sequences.clone();

    let (tx, rx) = oneshot::channel::<Vec<i64>>();
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
                if let Ok(mut msg) = serde_wasm_bindgen::from_value::<Message>(value) {
                    if !msg.is_self {
                        sequences.borrow_mut().push(msg.seq);
                    }
                    msg.is_read = 1;
                    store
                        .put(&serde_wasm_bindgen::to_value(&msg).unwrap())
                        .unwrap();
                }
            }
            let _ = cursor.continue_();
        } else {
            tx.take().unwrap().send(sequences.borrow().clone()).unwrap();
        }
    }) as Box<dyn FnMut(&Event)>);

    request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
    let result = rx.await.unwrap();
    Ok((result, success))
}

pub(super) async fn unread_count(store: IdbObjectStore) -> Result<usize> {
    let index = store.index(MESSAGE_IS_READ_INDEX)?;

    let request = index.count_with_key(&JsValue::from(0))?;

    let (tx, rx) = oneshot::channel();

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

    Ok(rx.await.unwrap_or_default())
}

pub(super) async fn delete_batch(
    store: IdbObjectStore,
    friend_id: &str,
) -> Result<Closure<dyn FnMut(&Event)>> {
    let index = store.index(MESSAGE_FRIEND_ID_INDEX)?;
    let range = IdbKeyRange::only(&JsValue::from(friend_id))?;
    let request = index.open_cursor_with_range(&range)?;

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

            cursor.delete().unwrap();

            let _ = cursor.continue_();
        }
    }) as Box<dyn FnMut(&Event)>);
    request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
    Ok(onsuccess)
}
