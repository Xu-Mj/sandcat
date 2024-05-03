use futures_channel::oneshot;
use std::ops::Deref;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Event, IdbKeyRange, IdbRequest};

use abi::model::friend::{FriendShipWithUser, FriendStatus, ReadStatus};

use crate::friendships::Friendships;

use super::{
    repository::Repository, FRIENDSHIP_TABLE_NAME, FRIENDSHIP_UNREAD_INDEX, FRIEND_USER_ID_INDEX,
};

#[derive(Debug, Clone)]
pub struct FriendShipRepo(Repository);

impl Deref for FriendShipRepo {
    type Target = Repository;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FriendShipRepo {
    pub fn new(repo: Repository) -> Self {
        Self(repo)
    }
}
#[async_trait::async_trait(?Send)]
impl Friendships for FriendShipRepo {
    async fn agree(&self, friendship_id: &str) {
        let mut friendship = self.get_friendship(friendship_id).await.unwrap();
        friendship.status = FriendStatus::Accepted as i32;
        self.put_friendship(&friendship).await;
    }

    async fn agree_by_friend_id(&self, friend_id: &str) {
        if let Some(mut friendship) = self.get_friendship_by_friend_id(friend_id).await {
            friendship.status = FriendStatus::Accepted as i32;
            self.put_friendship(&friendship).await;
        }
    }

    async fn put_friendship(&self, friendship: &FriendShipWithUser) {
        log::debug!("friendship: {:?}", friendship);
        let store = self
            .store(&String::from(FRIENDSHIP_TABLE_NAME))
            .await
            .unwrap();
        let value = serde_wasm_bindgen::to_value(friendship).unwrap();
        store.put(&value).unwrap();
    }
    /*
       async fn put_friendships(&self, friends: &[FriendShipWithUser]) {
           let store = self
               .store(&String::from(FRIENDSHIP_TABLE_NAME))
               .await
               .unwrap();
           friends.iter().for_each(|item| {
               let value = serde_wasm_bindgen::to_value(item).unwrap();
               store.put(&value).unwrap();
           });
       }
    */
    // todo 增加错误结果，用来标识
    async fn get_friendship(&self, friendship_id: &str) -> Option<FriendShipWithUser> {
        // 声明一个channel，接收查询结果
        let (tx, rx) = oneshot::channel::<Option<FriendShipWithUser>>();
        let store = self
            .store(&String::from(FRIENDSHIP_TABLE_NAME))
            .await
            .unwrap();

        let request = store
            .get(&JsValue::from(friendship_id))
            .expect("friend select get error");
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
        let on_add_error = Closure::once(move |event: &Event| {
            log::error!("获取好友请求错误: {:?}", event);
        });
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        rx.await.unwrap()
    }

    // todo 增加错误结果，用来标识
    async fn get_friendship_by_friend_id(&self, friend_id: &str) -> Option<FriendShipWithUser> {
        // 声明一个channel，接收查询结果
        let (tx, rx) = oneshot::channel::<Option<FriendShipWithUser>>();
        let store = self
            .store(&String::from(FRIENDSHIP_TABLE_NAME))
            .await
            .unwrap();
        let index = store
            .index(FRIEND_USER_ID_INDEX)
            .expect("friend select index error");
        let request = index
            .get(&JsValue::from(friend_id))
            .expect("friend select get error");
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
        let on_add_error = Closure::once(move |event: &Event| {
            log::error!("获取好友请求错误: {:?}", event);
        });
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        rx.await.unwrap()
    }

    async fn get_unread_count(&self) -> usize {
        // 声明一个channel，接收查询结果
        let (tx, rx) = oneshot::channel::<usize>();
        let store = self
            .store(&String::from(FRIENDSHIP_TABLE_NAME))
            .await
            .unwrap();
        let index = store
            .index(FRIENDSHIP_UNREAD_INDEX)
            .expect("friend select index error");
        let unread = IdbKeyRange::only(&JsValue::from("False")).unwrap();
        let request = index
            .count_with_key(&unread)
            .expect("friend select get error");
        let onsuccess = Closure::once(move |event: &Event| {
            let result = event.target().unwrap().dyn_into::<IdbRequest>().unwrap();
            let result = result.result().unwrap_or(JsValue::NULL);

            tx.send(result.as_f64().unwrap_or_default() as usize)
                .unwrap();
        });
        request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        let on_add_error = Closure::once(move |event: &Event| {
            log::error!("get unread count error: {:?}", event);
        });
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        rx.await.unwrap()
    }

    async fn clean_unread_count(&self) -> Result<Vec<String>, JsValue> {
        let (tx, rx) = oneshot::channel::<Vec<String>>();
        let store = self
            .store(&String::from(FRIENDSHIP_TABLE_NAME))
            .await
            .unwrap();
        let index = store
            .index(FRIENDSHIP_UNREAD_INDEX)
            .expect("friend select index error");
        let unread = IdbKeyRange::only(&JsValue::from("False")).unwrap();
        let request = index
            .open_cursor_with_range(&unread)
            .expect("friend select get error");
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
        let on_add_error = Closure::once(move |event: &Event| {
            log::error!("get unread count error: {:?}", event);
        });
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        onsuccess.forget();
        on_add_error.forget();
        Ok(rx.await.unwrap())
    }

    // async fn update_status(&self, fs: &str) -> Result<(), JsValue> {
    //     let store = self.store(FRIENDSHIP_TABLE_NAME).await?;
    //     let req = store.get(&JsValue::from(fs))?;
    //     let store = store.clone();
    //     let onsccess = Closure::once(move |event: &Event| {
    //         let target = event.target().unwrap();
    //         let result = target
    //             .dyn_into::<IdbRequest>()
    //             .unwrap()
    //             .result()
    //             .unwrap_or_default();
    //         if !result.is_undefined() && !result.is_null() {
    //             let mut fs: FriendShipWithUser = serde_wasm_bindgen::from_value(result).unwrap();
    //             fs.is_operated = true;
    //             store
    //                 .put(&serde_wasm_bindgen::to_value(&fs).unwrap())
    //                 .unwrap();
    //         }
    //     });
    //     req.set_onsuccess(Some(onsccess.as_ref().unchecked_ref()));
    //     onsccess.forget();
    //     Ok(())
    // }

    async fn get_list(&self) -> Vec<FriendShipWithUser> {
        let (tx, rx) = oneshot::channel::<Vec<FriendShipWithUser>>();
        let store = self
            .store(&String::from(FRIENDSHIP_TABLE_NAME))
            .await
            .unwrap();
        let request = store.open_cursor().expect("friend select all error");
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
        onsuccess.forget();
        let on_add_error = Closure::once(move |event: &Event| {
            log::error!("获取好友请求错误: {:?}", event);
        });
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        on_add_error.forget();
        rx.await.unwrap()
    }
}
