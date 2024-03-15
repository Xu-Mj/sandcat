use std::ops::Deref;

use futures_channel::oneshot;
use js_sys::Array;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{IdbKeyRange, IdbRequest};
use yew::{AttrValue, Event};

use crate::model::group::GroupMember;

use super::{repository::Repository, GROUP_ID_AND_USER_ID, GROUP_MEMBERS_TABLE_NAME};

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

    pub async fn get_by_group_id_and_friend_id(
        &self,
        group_id: AttrValue,
        friend_id: AttrValue,
    ) -> Result<Option<GroupMember>, JsValue> {
        let (tx, rx) = oneshot::channel::<Option<GroupMember>>();
        let store = self.store(GROUP_MEMBERS_TABLE_NAME).await?;
        let index = store.index(GROUP_ID_AND_USER_ID)?;
        let indices = Array::new();
        indices.push(&JsValue::from(friend_id.as_str()));
        indices.push(&JsValue::from(group_id.as_str()));
        let request = index.get(&JsValue::from(indices))?;
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

    pub async fn get_list_by_group_id(&self, group_id: &str) -> Result<Vec<GroupMember>, JsValue> {
        let (tx, rx) = oneshot::channel::<Vec<GroupMember>>();
        let store = self.store(GROUP_MEMBERS_TABLE_NAME).await?;
        let index = store.index("group_id")?;
        let range = IdbKeyRange::only(&JsValue::from(group_id))?;
        let request = index.open_cursor_with_range(&range.into())?;
        let on_add_error = Closure::once(move |event: &Event| {
            web_sys::console::log_1(&String::from("读取数据失败").into());
            web_sys::console::log_1(&event.into());
        });

        let mut groups = Vec::new();
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
                groups.push(group);
                let _ = cursor.continue_();
            } else {
                // 如果为null说明已经遍历完成
                //将总的结果发送出来
                let _ = tx.take().unwrap().send(groups.to_owned());
            }
        }) as Box<dyn FnMut(&Event)>);

        request.set_onsuccess(Some(success.as_ref().unchecked_ref()));
        success.forget();
        Ok(rx.await.unwrap())
    }
}
