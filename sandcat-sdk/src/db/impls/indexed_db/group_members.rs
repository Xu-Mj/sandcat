use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use futures_channel::oneshot;
use js_sys::Array;
use log::error;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{IdbKeyRange, IdbRequest};
use yew::Event;

use crate::db::group_members::GroupMembers;
use crate::error::Result;
use crate::model::group::GroupMember;

use super::{repository::Repository, GROUP_ID_AND_USER_ID, GROUP_MEMBERS_TABLE_NAME};
use super::{SuccessCallback, GROUP_ID_INDEX};

#[derive(Debug)]
pub struct GroupMembersRepo {
    repo: Repository,
    on_err_callback: Closure<dyn FnMut(&Event)>,
    on_get_list_success: SuccessCallback,
}

impl Deref for GroupMembersRepo {
    type Target = Repository;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl GroupMembersRepo {
    pub fn new(repo: Repository) -> Self {
        let on_err_callback =
            Closure::once(move |event: &Event| error!("group member operate error: {:?}", event));

        Self {
            repo,
            on_err_callback,
            on_get_list_success: Rc::new(RefCell::new(None)),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl GroupMembers for GroupMembersRepo {
    async fn put(&self, mem: &GroupMember) -> Result<()> {
        let store = self.store(GROUP_MEMBERS_TABLE_NAME).await?;
        let value = serde_wasm_bindgen::to_value(mem)?;
        store.put(&value)?;
        Ok(())
    }

    async fn put_list(&self, members: &[GroupMember]) -> Result<()> {
        let store = self.store(GROUP_MEMBERS_TABLE_NAME).await?;
        for member in members {
            let value = serde_wasm_bindgen::to_value(member)?;
            store.put(&value)?;
        }
        Ok(())
    }

    async fn get(&self, id: i64) -> Result<Option<GroupMember>> {
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
        request.set_onerror(Some(self.on_err_callback.as_ref().unchecked_ref()));
        Ok(rx.await.unwrap())
    }

    async fn get_by_group_id_and_friend_id(
        &self,
        group_id: &str,
        friend_id: &str,
    ) -> Result<Option<GroupMember>> {
        let (tx, rx) = oneshot::channel::<Option<GroupMember>>();
        let store = self.store(GROUP_MEMBERS_TABLE_NAME).await?;
        let index = store.index(GROUP_ID_AND_USER_ID)?;
        let indices = Array::new();
        indices.push(&JsValue::from(friend_id));
        indices.push(&JsValue::from(group_id));
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
        request.set_onerror(Some(self.on_err_callback.as_ref().unchecked_ref()));
        Ok(rx.await.unwrap())
    }

    async fn get_list_by_group_id(&self, group_id: &str) -> Result<Vec<GroupMember>> {
        let (tx, rx) = oneshot::channel::<Vec<GroupMember>>();
        let store = self.store(GROUP_MEMBERS_TABLE_NAME).await?;
        let index = store.index(GROUP_ID_INDEX)?;
        let range = IdbKeyRange::only(&JsValue::from(group_id))?;
        let request = index.open_cursor_with_range(&range.into())?;

        let mut groups = Vec::new();
        let mut tx = Some(tx);

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
                groups.push(group);
                let _ = cursor.continue_();
            } else {
                // 如果为null说明已经遍历完成
                //将总的结果发送出来
                let _ = tx.take().unwrap().send(groups.to_owned());
            }
        }) as Box<dyn FnMut(&Event)>);

        request.set_onsuccess(Some(success.as_ref().unchecked_ref()));

        *self.on_get_list_success.borrow_mut() = Some(success);

        request.set_onerror(Some(self.on_err_callback.as_ref().unchecked_ref()));

        Ok(rx.await.unwrap())
    }

    async fn delete(&self, group_id: &str, user_id: &str) -> Result<()> {
        let store = self.store(GROUP_MEMBERS_TABLE_NAME).await?;
        let index = store.index(GROUP_ID_AND_USER_ID)?;

        let indices = Array::new();
        indices.push(&JsValue::from(user_id));
        indices.push(&JsValue::from(group_id));

        let request = index.get(&JsValue::from(indices))?;

        let (tx, rx) = oneshot::channel::<u8>();

        let onsuccess = Closure::once(move |event: &Event| {
            let result = event
                .target()
                .unwrap()
                .dyn_ref::<IdbRequest>()
                .unwrap()
                .result()
                .unwrap();
            if !result.is_undefined() && !result.is_null() {
                let mut member: GroupMember = serde_wasm_bindgen::from_value(result).unwrap();
                member.is_deleted = 0;

                if let Err(err) = store.put(&serde_wasm_bindgen::to_value(&member).unwrap()) {
                    error!("delete group member error: {:?}", err);
                }
            }
            tx.send(0).unwrap();
        });
        request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        request.set_onerror(Some(self.on_err_callback.as_ref().unchecked_ref()));

        rx.await.unwrap();
        Ok(())
    }

    async fn delete_batch(&self, group_id: &str, user_ids: &[String]) -> Result<()> {
        for id in user_ids {
            self.delete(group_id, id).await?;
        }
        Ok(())
    }
}
