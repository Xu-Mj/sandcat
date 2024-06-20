use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use futures_channel::oneshot;
use indexmap::IndexMap;
use log::error;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{IdbKeyRange, IdbRequest};
use yew::{AttrValue, Event};

use crate::db::groups::GroupInterface;
use crate::error::Result;
use crate::model::{
    group::{Group, GroupMember},
    message::Message,
};

use super::SuccessCallback;
use super::{
    repository::Repository, GROUP_ID_INDEX, GROUP_MEMBERS_TABLE_NAME, GROUP_MSG_TABLE_NAME,
    GROUP_TABLE_NAME, MESSAGE_FRIEND_ID_INDEX,
};

#[derive(Debug)]
pub struct GroupRepo {
    repo: Repository,
    on_err_callback: Closure<dyn FnMut(&Event)>,
    on_get_list_success: SuccessCallback,
    on_del_mem_success: SuccessCallback,
    on_del_msg_success: SuccessCallback,
}

impl Deref for GroupRepo {
    type Target = Repository;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl GroupRepo {
    pub fn new(repo: Repository) -> Self {
        let on_err_callback =
            Closure::once(move |event: &Event| error!("group operate error: {:?}", event));

        Self {
            repo,
            on_err_callback,
            on_get_list_success: Rc::new(RefCell::new(None)),
            on_del_mem_success: Rc::new(RefCell::new(None)),
            on_del_msg_success: Rc::new(RefCell::new(None)),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl GroupInterface for GroupRepo {
    async fn put(&self, group: &Group) -> Result<()> {
        let store = self.store(GROUP_TABLE_NAME).await?;
        let value = serde_wasm_bindgen::to_value(group)?;
        store.put(&value)?;
        Ok(())
    }

    async fn get(&self, id: &str) -> Result<Option<Group>> {
        let (tx, rx) = oneshot::channel::<Option<Group>>();
        let store = self.store(GROUP_TABLE_NAME).await?;
        let request = store.get(&JsValue::from(id))?;
        let onsuccess = Closure::once(move |event: &Event| {
            let result = event
                .target()
                .unwrap()
                .dyn_ref::<IdbRequest>()
                .unwrap()
                .result()
                .unwrap();
            let mut group = None;
            if !result.is_undefined() && !result.is_null() {
                group = Some(serde_wasm_bindgen::from_value(result).unwrap());
            }
            tx.send(group).unwrap();
        });
        request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));

        request.set_onerror(Some(self.on_err_callback.as_ref().unchecked_ref()));
        Ok(rx.await.unwrap())
    }

    async fn get_list(&self) -> Result<IndexMap<AttrValue, Group>> {
        let (tx, rx) = oneshot::channel::<IndexMap<AttrValue, Group>>();
        let store = self.store(GROUP_TABLE_NAME).await?;
        let request = store.open_cursor()?;

        let groups = std::rc::Rc::new(std::cell::RefCell::new(IndexMap::new()));
        let groups = groups.clone();
        let mut tx = Some(tx);

        request.set_onerror(Some(self.on_err_callback.as_ref().unchecked_ref()));

        {
            let mut on_get_list = self.on_get_list_success.borrow_mut();
            if let Some(ref on_get_list) = *on_get_list {
                request.set_onsuccess(Some(on_get_list.as_ref().unchecked_ref()));
            } else {
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
                        let group: Group = serde_wasm_bindgen::from_value(value).unwrap();
                        let id = group.id.to_string().into();
                        groups.borrow_mut().insert(id, group);
                        let _ = cursor.continue_();
                    } else {
                        // 如果为null说明已经遍历完成
                        //将总的结果发送出来
                        let _ = tx.take().unwrap().send(groups.borrow().clone());
                    }
                }) as Box<dyn FnMut(&Event)>);

                request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
                *on_get_list = Some(success);
            }
        }
        Ok(rx.await.unwrap())
    }

    // delete group and related group members
    async fn delete(&self, id: &str) -> Result<()> {
        let store = self.store(GROUP_TABLE_NAME).await?;
        store.delete(&JsValue::from(id))?;

        // delete group members
        let store = self.store(GROUP_MEMBERS_TABLE_NAME).await?;
        // need query group members first
        // use group id index
        let index = store.index(GROUP_ID_INDEX)?;
        let range = JsValue::from(IdbKeyRange::only(&JsValue::from(id))?);
        let request = index.open_cursor_with_range(&range)?;

        {
            let mut on_del_mem = self.on_del_mem_success.borrow_mut();
            if let Some(ref on_del) = *on_del_mem {
                request.set_onsuccess(Some(on_del.as_ref().unchecked_ref()));
            } else {
                let success = Closure::wrap(Box::new(move |event: &Event| {
                    let target = event.target().expect("msg");
                    let req = target
                        .dyn_ref::<IdbRequest>()
                        .expect("Event target is IdbRequest; qed");
                    let result = match req.result() {
                        Ok(data) => data,
                        Err(_err) => {
                            error!("query friend list error ...:{:?}", _err);
                            JsValue::null()
                        }
                    };
                    if !result.is_null() {
                        let cursor = result
                            .dyn_ref::<web_sys::IdbCursorWithValue>()
                            .expect("result is IdbCursorWithValue; qed");
                        let value = cursor.value().unwrap();
                        // 反序列化
                        let group: GroupMember = serde_wasm_bindgen::from_value(value).unwrap();
                        store.delete(&JsValue::from(group.id)).unwrap();
                        let _ = cursor.continue_();
                    }
                }) as Box<dyn FnMut(&Event)>);
                request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
                *on_del_mem = Some(success);
            }
        }

        // delete group messages
        let store = self.store(GROUP_MSG_TABLE_NAME).await?;
        // use group id index
        let index = store.index(MESSAGE_FRIEND_ID_INDEX)?;
        let range = JsValue::from(IdbKeyRange::only(&JsValue::from(id))?);
        let request = index.open_cursor_with_range(&range)?;

        {
            let mut on_del_msg = self.on_del_msg_success.borrow_mut();
            if let Some(ref on_del) = *on_del_msg {
                request.set_onsuccess(Some(on_del.as_ref().unchecked_ref()));
            } else {
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
                        let group: Message = serde_wasm_bindgen::from_value(value).unwrap();
                        store.delete(&JsValue::from(group.id)).unwrap();
                        let _ = cursor.continue_();
                    }
                }) as Box<dyn FnMut(&Event)>);
                request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
                *on_del_msg = Some(success);
            }
        }
        Ok(())
    }

    async fn dismiss(&self, id: &str) -> Result<()> {
        if let Ok(Some(mut group)) = self.get(id).await {
            group.deleted = true;
            return self.put(&group).await;
        }
        Ok(())
    }
}
