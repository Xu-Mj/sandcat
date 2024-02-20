#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_assignments)]

use super::{
    db::Repository, FRIENDSHIP_ID_INDEX, FRIENDSHIP_TABLE_NAME, FRIENDSHIP_UNREAD_INDEX,
    FRIEND_USER_ID_INDEX,
};
use crate::model::friend::FriendShipWithUser;
use futures_channel::oneshot;
use std::ops::Deref;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Event, IdbKeyRange, IdbRequest};
use yew::AttrValue;

pub struct FriendShipRepo(Repository);

impl Deref for FriendShipRepo {
    type Target = Repository;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub async fn new() -> FriendShipRepo {
    FriendShipRepo::new().await
}

impl FriendShipRepo {
    pub async fn new() -> Self {
        Self(Repository::new().await)
    }

    pub async fn agree(&self, friendship_id: AttrValue) {
        let mut friendship = self.get_friendship(friendship_id).await.unwrap();
        friendship.status = AttrValue::from("1");
        self.put_friendship(&friendship).await;
    }
    pub async fn agree_by_friend_id(&self, friend_id: AttrValue) {
        let mut friendship = self.get_friendship_by_friend_id(friend_id).await.unwrap();
        friendship.status = AttrValue::from("1");
        self.put_friendship(&friendship).await;
    }
    pub async fn put_friendship(&self, friendship: &FriendShipWithUser) {
        let store = self
            .store(&String::from(FRIENDSHIP_TABLE_NAME))
            .await
            .unwrap();
        let value = serde_wasm_bindgen::to_value(friendship).unwrap();
        store.put(&value).unwrap();
    }
    pub async fn put_friendships(&self, friends: &Vec<FriendShipWithUser>) {
        let store = self
            .store(&String::from(FRIENDSHIP_TABLE_NAME))
            .await
            .unwrap();
        friends.iter().for_each(|item| {
            let value = serde_wasm_bindgen::to_value(item).unwrap();
            store.put(&value).unwrap();
        });
    }

    // todo 增加错误结果，用来标识
    pub async fn get_friendship(&self, friendship_id: AttrValue) -> Option<FriendShipWithUser> {
        // 声明一个channel，接收查询结果
        let (tx, rx) = oneshot::channel::<Option<FriendShipWithUser>>();
        let store = self
            .store(&String::from(FRIENDSHIP_TABLE_NAME))
            .await
            .unwrap();
        let index = store
            .index(FRIENDSHIP_ID_INDEX)
            .expect("friend select index error");
        let request = index
            .get(&JsValue::from(friendship_id.as_str()))
            .expect("friend select get error");
        let onsuccess = Closure::once(move |event: &Event| {
            let result = event
                .target()
                .unwrap()
                .dyn_ref::<IdbRequest>()
                .unwrap()
                .result()
                .unwrap();
            let mut friend = FriendShipWithUser::default();
            if !result.is_undefined() && !result.is_null() {
                friend = serde_wasm_bindgen::from_value(result).unwrap();
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
    pub async fn get_friendship_by_friend_id(
        &self,
        friend_id: AttrValue,
    ) -> Option<FriendShipWithUser> {
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
            let mut friend = FriendShipWithUser::default();
            if !result.is_undefined() && !result.is_null() {
                friend = serde_wasm_bindgen::from_value(result).unwrap();
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

    pub async fn get_unread_count(&self) -> usize {
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
            let result = result.result().unwrap_or_else(|_| JsValue::NULL);

            tx.send(result.as_f64().unwrap_or_default() as usize)
                .unwrap();

            // if result.is_null() {
            //     tx.send(0).unwrap();
            // } else {
            //     // let count = result.as_f64().unwrap();
            //
            //     tx.send(count as usize).unwrap();
            // }
        });
        request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        let on_add_error = Closure::once(move |event: &Event| {
            log::error!("获取好友请求错误: {:?}", event);
        });
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        rx.await.unwrap()
    }

    pub async fn get_list(&self) -> Vec<FriendShipWithUser> {
        let (tx, rx) = oneshot::channel::<Vec<FriendShipWithUser>>();
        let store = self
            .store(&String::from(FRIENDSHIP_TABLE_NAME))
            .await
            .unwrap();
        let request = store.open_cursor().expect("friend select all error");
        let mut friends = Vec::new();
        let mut tx = Some(tx);
        let onsuccess = Closure::wrap(Box::new(move |event: &Event| {
            // gloo::console::log!("query friends onsuccess");
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
