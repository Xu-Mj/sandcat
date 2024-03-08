use std::ops::Deref;

use futures_channel::oneshot;
use indexmap::IndexMap;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::IdbRequest;
use yew::{AttrValue, Event};

use crate::model::group::GroupMember;

use super::{repository::Repository, GROUP_MEMBERS_TABLE_NAME};

pub struct GroupMembersRepo(Repository);

impl Deref for GroupMembersRepo {
    type Target = Repository;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[allow(dead_code)]
impl GroupMembersRepo {
    pub async fn new() -> Self {
        Self(Repository::new().await)
    }

    pub async fn put(&self, mem: &GroupMember) -> Result<(), JsValue> {
        let store = self.store(GROUP_MEMBERS_TABLE_NAME).await?;
        let value = serde_wasm_bindgen::to_value(mem)?;
        store.put(&value)?;
        Ok(())
    }

    pub async fn put_list(&self, members: Vec<GroupMember>) -> Result<(), JsValue> {
        let store = self.store(GROUP_MEMBERS_TABLE_NAME).await?;
        for member in members {
            let value = serde_wasm_bindgen::to_value(&member)?;
            store.put(&value)?;
        }
        Ok(())
    }

    pub async fn get(&self, id: i64) -> Result<Option<GroupMember>, JsValue> {
        let (tx, rx) = oneshot::channel::<Option<GroupMember>>();
        let store = self.store(GROUP_MEMBERS_TABLE_NAME).await?;
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
        let on_add_error = Closure::once(move |event: &Event| {
            web_sys::console::log_1(&String::from("读取数据失败").into());
            web_sys::console::log_1(&event.into());
        });
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        Ok(rx.await.unwrap())
    }

    pub async fn get_list(&self) -> Result<IndexMap<AttrValue, GroupMember>, JsValue> {
        let (tx, rx) = oneshot::channel::<IndexMap<AttrValue, GroupMember>>();
        let store = self.store(GROUP_MEMBERS_TABLE_NAME).await?;
        let request = store.open_cursor()?;
        let on_add_error = Closure::once(move |event: &Event| {
            web_sys::console::log_1(&String::from("读取数据失败").into());
            web_sys::console::log_1(&event.into());
        });

        let groups = std::rc::Rc::new(std::cell::RefCell::new(IndexMap::new()));
        let groups = groups.clone();
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
                let group: GroupMember = serde_wasm_bindgen::from_value(value).unwrap();
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
        success.forget();
        Ok(rx.await.unwrap())
    }
}
