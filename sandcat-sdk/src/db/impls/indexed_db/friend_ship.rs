use futures_channel::oneshot;
use log::error;
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Event, IdbKeyRange, IdbRequest};

use crate::db::friendships::Friendships;
use crate::error::{Error, Result};
use crate::model::friend::{FriendShipWithUser, FriendStatus, ReadStatus};

use super::SuccessCallback;
use super::{
    repository::Repository, FRIENDSHIP_TABLE_NAME, FRIENDSHIP_UNREAD_INDEX, FRIEND_USER_ID_INDEX,
};

#[derive(Debug)]
pub struct FriendShipRepo {
    repo: Repository,
    on_err_callback: Closure<dyn FnMut(&Event)>,
    /// use `RefCell` that we can modify this attr through the `&self`
    on_clean_success: SuccessCallback,
    on_get_list_success: SuccessCallback,
}

impl Deref for FriendShipRepo {
    type Target = Repository;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl FriendShipRepo {
    pub fn new(repo: Repository) -> Self {
        let on_err_callback =
            Closure::once(move |event: &Event| error!("friendship operate error: {:?}", event));

        Self {
            repo,
            on_err_callback,
            on_clean_success: Rc::new(RefCell::new(None)),
            on_get_list_success: Rc::new(RefCell::new(None)),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl Friendships for FriendShipRepo {
    async fn agree(&self, friendship_id: &str) -> Result<()> {
        let mut friendship = self
            .get_friendship(friendship_id)
            .await?
            .ok_or(Error::local_not_found("friend not found"))?;
        friendship.status = FriendStatus::Accepted as i32;
        self.put_friendship(&friendship).await
    }

    async fn agree_by_friend_id(&self, friend_id: &str) -> Result<()> {
        let mut friendship = self
            .get_friendship_by_friend_id(friend_id)
            .await?
            .ok_or(Error::local_not_found("friend not found"))?;
        friendship.status = FriendStatus::Accepted as i32;
        self.put_friendship(&friendship).await?;
        Ok(())
    }

    async fn put_friendship(&self, friendship: &FriendShipWithUser) -> Result<()> {
        let store = self.store(&String::from(FRIENDSHIP_TABLE_NAME)).await?;
        let value = serde_wasm_bindgen::to_value(friendship)?;
        store.put(&value)?;
        Ok(())
    }

    async fn put_fs_batch(&self, friendship: &[FriendShipWithUser]) -> Result<()> {
        let store = self.store(&String::from(FRIENDSHIP_TABLE_NAME)).await?;
        for fs in friendship.iter() {
            let value = serde_wasm_bindgen::to_value(fs)?;
            store.put(&value)?;
        }
        Ok(())
    }

    async fn get_friendship(&self, friendship_id: &str) -> Result<Option<FriendShipWithUser>> {
        // 声明一个channel，接收查询结果
        let (tx, rx) = oneshot::channel::<Option<FriendShipWithUser>>();
        let store = self.store(&String::from(FRIENDSHIP_TABLE_NAME)).await?;

        let request = store.get(&JsValue::from(friendship_id))?;
        let onsuccess = Closure::once(move |event: &Event| {
            let result = event
                .target()
                .unwrap()
                .dyn_ref::<IdbRequest>()
                .unwrap()
                .result()
                .unwrap();
            if !result.is_undefined() && !result.is_null() {
                let friend = serde_wasm_bindgen::from_value(result).unwrap();
                tx.send(Some(friend)).unwrap();
            } else {
                tx.send(None).unwrap();
            }
        });
        request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        let on_add_error =
            Closure::once(move |event: &Event| log::error!("read friendship error: {:?}", event));
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        Ok(rx.await.unwrap())
    }

    async fn get_friendship_by_friend_id(
        &self,
        friend_id: &str,
    ) -> Result<Option<FriendShipWithUser>> {
        // 声明一个channel，接收查询结果
        let (tx, rx) = oneshot::channel::<Option<FriendShipWithUser>>();
        let store = self.store(&String::from(FRIENDSHIP_TABLE_NAME)).await?;
        let index = store.index(FRIEND_USER_ID_INDEX)?;
        let request = index.get(&JsValue::from(friend_id))?;
        let onsuccess = Closure::once(move |event: &Event| {
            let result = event
                .target()
                .unwrap()
                .dyn_ref::<IdbRequest>()
                .unwrap()
                .result()
                .unwrap();
            if !result.is_undefined() && !result.is_null() {
                let friend = serde_wasm_bindgen::from_value(result).unwrap();
                tx.send(Some(friend)).unwrap();
            } else {
                tx.send(None).unwrap();
            }
        });
        request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        let on_add_error =
            Closure::once(move |event: &Event| log::error!("read friendship error: {:?}", event));
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        Ok(rx.await.unwrap())
    }

    async fn get_unread_count(&self) -> Result<usize> {
        // 声明一个channel，接收查询结果
        let (tx, rx) = oneshot::channel::<usize>();
        let store = self.store(&String::from(FRIENDSHIP_TABLE_NAME)).await?;
        let index = store.index(FRIENDSHIP_UNREAD_INDEX)?;
        let unread = IdbKeyRange::only(&JsValue::from("False"))?;
        let request = index.count_with_key(&unread)?;
        let onsuccess = Closure::once(move |event: &Event| {
            let result = event.target().unwrap().dyn_into::<IdbRequest>().unwrap();
            let result = result.result().unwrap_or(JsValue::NULL);

            tx.send(result.as_f64().unwrap_or_default() as usize)
                .unwrap();
        });
        request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        let on_add_error =
            Closure::once(move |event: &Event| log::error!("get unread count error: {:?}", event));
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        Ok(rx.await.unwrap())
    }

    async fn clean_unread_count(&self) -> Result<Vec<String>> {
        let (tx, rx) = oneshot::channel::<Vec<String>>();
        let store = self.store(&String::from(FRIENDSHIP_TABLE_NAME)).await?;
        let index = store.index(FRIENDSHIP_UNREAD_INDEX)?;
        let unread = IdbKeyRange::only(&JsValue::from("False"))?;
        let request = index.open_cursor_with_range(&unread)?;
        let mut tx = Some(tx);
        let mut ids = Vec::new();

        let onsuccess = Closure::wrap(Box::new(move |event: &Event| {
            let result = event.target().unwrap().dyn_into::<IdbRequest>().unwrap();
            let result = result.result().unwrap_or(JsValue::NULL);
            if !result.is_null() {
                let cursor = result
                    .dyn_ref::<web_sys::IdbCursorWithValue>()
                    .expect("cursor error");
                let value = cursor.value().expect("cursor value error");
                let mut res: FriendShipWithUser = serde_wasm_bindgen::from_value(value).unwrap();
                res.read = ReadStatus::True;
                match cursor.update(&serde_wasm_bindgen::to_value(&res).unwrap()) {
                    Ok(_) => {
                        ids.push(res.msg_id.to_string());
                        let _ = cursor.continue_();
                    }
                    Err(err) => {
                        log::error!("更新好友请求错误: {:?}", err);
                    }
                };
            } else {
                tx.take().unwrap().send(ids.to_owned()).unwrap();
            }
        }) as Box<dyn FnMut(&Event)>);
        request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        *self.on_clean_success.borrow_mut() = Some(onsuccess);

        request.set_onerror(Some(self.on_err_callback.as_ref().unchecked_ref()));
        Ok(rx.await.unwrap())
    }

    async fn get_list(&self) -> Result<Vec<FriendShipWithUser>> {
        let (tx, rx) = oneshot::channel::<Vec<FriendShipWithUser>>();
        let store = self.store(&String::from(FRIENDSHIP_TABLE_NAME)).await?;
        let request = store.open_cursor()?;
        let mut friends = Vec::new();
        let mut tx = Some(tx);

        let onsuccess = Closure::wrap(Box::new(move |event: &Event| {
            let target = event.target().unwrap();
            let request = target.dyn_ref::<IdbRequest>().unwrap();
            let result = match request.result() {
                Ok(data) => data,
                Err(_) => JsValue::NULL,
            };
            if result.is_null() {
                let _ = tx.take().unwrap().send(friends.to_owned());
            } else {
                let cursor = result
                    .dyn_ref::<web_sys::IdbCursorWithValue>()
                    .expect("cursor error");
                let value = cursor.value().expect("cursor value error");
                friends.push(serde_wasm_bindgen::from_value(value).unwrap());
                let _ = cursor.continue_();
            }
        }) as Box<dyn FnMut(&Event)>);
        request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        *self.on_get_list_success.borrow_mut() = Some(onsuccess);

        request.set_onerror(Some(self.on_err_callback.as_ref().unchecked_ref()));
        Ok(rx.await.unwrap())
    }
}
