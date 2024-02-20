#![allow(dead_code)]
#![allow(unused_variables)]

use futures_channel::oneshot;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    console, IdbCursorWithValue, IdbDatabase, IdbIndexParameters, IdbKeyRange, IdbObjectStore,
    IdbObjectStoreParameters, IdbRequest, IdbTransactionMode,
};
use yew::prelude::*;

use chrono::prelude::*;

const DATE_FORMAT_STR: &'static str = "%Y-%m-%d %H:%M:%S";
// const DB_NAME: &'static str = "im";
const DB_VERSION: u32 = 1;

use serde::{Deserialize, Serialize};

use crate::db::{
    CONFIG_TABLE_NAME, CONVERSATION_FRIEND_ID_INDEX, CONVERSATION_LAST_MSG_TIME_INDEX,
    CONVERSATION_TABLE_NAME, CURRENT_CONV_TABLE_NAME, FRIENDSHIP_ID_INDEX, FRIENDSHIP_TABLE_NAME,
    FRIENDSHIP_UNREAD_INDEX, FRIEND_ADDRESS_INDEX, FRIEND_FRIEND_ID_INDEX, FRIEND_GENDER_INDEX,
    FRIEND_NAME_INDEX, FRIEND_PHONE_INDEX, FRIEND_REMARK_INDEX, FRIEND_TABLE_NAME,
    FRIEND_TIME_INDEX, FRIEND_USER_ID_INDEX, MESSAGE_CONTENT_INDEX, MESSAGE_FRIEND_ID_INDEX,
    MESSAGE_ID_INDEX, MESSAGE_IS_READ_INDEX, MESSAGE_TABLE_NAME, MESSAGE_TIME_INDEX,
    MESSAGE_TYPE_INDEX, USER_TABLE_NAME,
};

use super::DB_NAME;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Note {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub id: Option<u32>,
    pub content: String,
    pub create_time: String,
}

// pub static REPO: OnceCell<Repository> = OnceCell::const_new();
//
// pub async fn repository() -> &'static Repository {
//     REPO.get_or_init(Repository::new)
// }

pub struct Repository {
    db: IdbDatabase,
}

impl Repository {
    pub async fn new() -> Repository {
        let db_name = DB_NAME.get().unwrap();
        // gloo::console::log!("正在创建数据库");
        // 这里使用channel来获取异步的结果
        let (tx, rx) = oneshot::channel::<IdbDatabase>();
        // 获取window对象
        let window = web_sys::window().unwrap();
        // 获取indexedDB对象
        let idb_factory = window.indexed_db().unwrap().unwrap();
        // 打开数据库
        let open_request = idb_factory.open_with_u32(db_name, DB_VERSION).unwrap();

        //
        let on_upgradeneeded = Closure::once(move |event: &Event| {
            let target = event.target().expect("Event should have a target; qed");
            let req = target
                .dyn_ref::<IdbRequest>()
                .expect("Event target is IdbRequest; qed");

            let result = req
                .result()
                .expect("IndexedDB.onsuccess should have a valid result; qed");
            assert!(result.is_instance_of::<IdbDatabase>());
            let db = IdbDatabase::from(result);
            let mut parameters: IdbObjectStoreParameters = IdbObjectStoreParameters::new();
            parameters.key_path(Some(&JsValue::from_str("id")));
            parameters.auto_increment(true);

            let store = db
                .create_object_store_with_optional_parameters(
                    &String::from(USER_TABLE_NAME),
                    &parameters,
                )
                .unwrap();
            // store.create_index_with_str("login", "login").unwrap();

            let store = db
                .create_object_store_with_optional_parameters(
                    &String::from(MESSAGE_TABLE_NAME),
                    &parameters,
                )
                .unwrap();
            let mut param: IdbIndexParameters = IdbIndexParameters::new();
            param.unique(true);
            store
                .create_index_with_str_and_optional_parameters(MESSAGE_ID_INDEX, "msg_id", &param)
                .unwrap();
            store
                .create_index_with_str(MESSAGE_FRIEND_ID_INDEX, "friend_id")
                .unwrap();
            store
                .create_index_with_str(MESSAGE_CONTENT_INDEX, "content")
                .unwrap();
            store
                .create_index_with_str(MESSAGE_TIME_INDEX, "create_time")
                .unwrap();
            store
                .create_index_with_str(MESSAGE_TYPE_INDEX, "content_type")
                .unwrap();
            store
                .create_index_with_str(MESSAGE_IS_READ_INDEX, "is_read")
                .unwrap();
            let store = db
                .create_object_store_with_optional_parameters(
                    &String::from(CONFIG_TABLE_NAME),
                    &parameters,
                )
                .unwrap();

            let mut param: IdbIndexParameters = IdbIndexParameters::new();
            param.unique(true);
            store
                .create_index_with_str_and_optional_parameters("name", "name", &param)
                .unwrap();
            let store = db
                .create_object_store_with_optional_parameters(
                    &String::from(CURRENT_CONV_TABLE_NAME),
                    &parameters,
                )
                .unwrap();

            let mut param: IdbIndexParameters = IdbIndexParameters::new();
            param.unique(true);
            store
                .create_index_with_str_and_optional_parameters("item_id", "item_id", &param)
                .unwrap();

            let store = db
                .create_object_store_with_optional_parameters(
                    &String::from(CONVERSATION_TABLE_NAME),
                    &parameters,
                )
                .unwrap();
            store
                .create_index_with_str(CONVERSATION_FRIEND_ID_INDEX, "friend_id")
                .unwrap();
            store
                .create_index_with_str(CONVERSATION_LAST_MSG_TIME_INDEX, "last_msg_time")
                .unwrap();

            let store = db
                .create_object_store_with_optional_parameters(
                    &String::from(FRIENDSHIP_TABLE_NAME),
                    &parameters,
                )
                .unwrap();
            store
                .create_index_with_str(FRIEND_USER_ID_INDEX, "user_id")
                .unwrap();
            store
                .create_index_with_str(FRIENDSHIP_ID_INDEX, "friendship_id")
                .unwrap();
            store
                .create_index_with_str(FRIENDSHIP_UNREAD_INDEX, "read")
                .unwrap();
            let store = db
                .create_object_store_with_optional_parameters(
                    &String::from(FRIEND_TABLE_NAME),
                    &parameters,
                )
                .unwrap();

            store
                .create_index_with_str(FRIEND_FRIEND_ID_INDEX, "friend_id")
                .unwrap();
            store
                .create_index_with_str(FRIEND_NAME_INDEX, "name")
                .unwrap();
            store
                .create_index_with_str(FRIEND_REMARK_INDEX, "remark")
                .unwrap();
            store
                .create_index_with_str(FRIEND_GENDER_INDEX, "gender")
                .unwrap();
            store
                .create_index_with_str(FRIEND_PHONE_INDEX, "phone")
                .unwrap();
            store
                .create_index_with_str(FRIEND_ADDRESS_INDEX, "address")
                .unwrap();
            store
                .create_index_with_str(FRIEND_TIME_INDEX, "update_time")
                .unwrap();

            // db.create_object_store("users").unwrap();
            // console::log_1(&JsValue::from("_store.unwrap()"));

            // let _index = store
            //     .create_index_with_str(&String::from("name"), &String::from("name"))
            //     .expect("create_index_with_str error");
        });
        open_request.set_onupgradeneeded(Some(on_upgradeneeded.as_ref().unchecked_ref()));
        on_upgradeneeded.forget();

        let on_success = Closure::once(move |event: &Event| {
            // Extract database handle from the event
            let target = event.target().expect("Event should have a target; qed");
            let req = target
                .dyn_ref::<IdbRequest>()
                .expect("Event target is IdbRequest; qed");

            let result = req
                .result()
                .expect("IndexedDB.onsuccess should have a valid result; qed");
            assert!(result.is_instance_of::<IdbDatabase>());

            let db = IdbDatabase::from(result);
            let _ = tx.send(db);
        });
        open_request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        on_success.forget();

        let db = rx.await.unwrap();
        // gloo::console::log!("数据库创建成功");

        Repository { db }
    }

    pub async fn store(&self, name: &String) -> Result<IdbObjectStore, JsValue> {
        // console::log_1(&JsValue::from(&self.db.clone()));

        let transaction = self
            .db
            .transaction_with_str_and_mode(name, IdbTransactionMode::Readwrite)?;
        transaction.object_store(name)
    }

    pub async fn delete_db(&self) {
        let db_name = DB_NAME.get().unwrap();

        let window = web_sys::window().unwrap();
        // 获取indexedDB对象
        let idb_factory = window.indexed_db().unwrap().unwrap();
        idb_factory.delete_database(db_name).unwrap();
    }

    pub async fn get_note(&self, id: u32) -> Note {
        let (tx, rx) = oneshot::channel::<Note>();

        let transaction = self
            .db
            .transaction_with_str_and_mode(&String::from("note"), IdbTransactionMode::Readwrite)
            .expect("transaction_with_str error");
        let store = transaction
            .object_store(&String::from("note"))
            .expect("store error");

        console::log_1(&id.clone().to_string().into());

        let request = store
            .get(&JsValue::from(id.clone()))
            .expect("get all error");
        let on_add_error = Closure::once(move |event: &Event| {
            console::log_1(&String::from("读取数据失败").into());
            console::log_1(&event.into());
        });
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        on_add_error.forget();

        let on_success = Closure::once(move |event: &Event| {
            let target = event.target().expect("msg");
            let req = target
                .dyn_ref::<IdbRequest>()
                .expect("Event target is IdbRequest; qed");
            let result = req.result().expect("read result error");
            console::log_1(&result.clone().into());
            let note: Note = serde_wasm_bindgen::from_value(result).expect("msg");
            console::log_1(&note.content.clone().into());
            // console::log_1(&String::from("读取数据成功").into());
            let _ = tx.send(note);
        });
        request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        on_success.forget();

        let note = rx.await.unwrap();
        note
    }

    pub fn put_note(&self, note: &Note) {
        // let (tx, rx) = oneshot::channel::<Note>();
        console::log_1(&String::from("更新笔记").into());

        let transaction = self
            .db
            .transaction_with_str_and_mode(&String::from("note"), IdbTransactionMode::Readwrite)
            .expect("transaction_with_str error");
        let store = transaction
            .object_store(&String::from("note"))
            .expect("store error");

        let request = store
            .put(&serde_wasm_bindgen::to_value(&note).unwrap())
            .expect("get all error");
        let on_add_error = Closure::once(move |event: &Event| {
            console::log_1(&String::from("更新数据失败").into());
            console::log_1(&event.into());
        });
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        on_add_error.forget();

        let on_success = Closure::once(move |event: &Event| {
            let target = event.target().expect("msg");
            let req = target
                .dyn_ref::<IdbRequest>()
                .expect("Event target is IdbRequest; qed");
            let result = req.result().expect("read result error");
            console::log_1(&result.clone().into());
            console::log_1(&String::from("更新数据成功").into());
            // let _ = tx.send(note);
        });
        request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        on_success.forget();

        // let note = rx.await.unwrap();
        // note
    }

    pub fn delete_note(&self, id: u32) {
        // let (tx, rx) = oneshot::channel::<Note>();
        console::log_1(&String::from("删除笔记").into());

        let transaction = self
            .db
            .transaction_with_str_and_mode(&String::from("note"), IdbTransactionMode::Readwrite)
            .expect("transaction_with_str error");
        let store = transaction
            .object_store(&String::from("note"))
            .expect("store error");

        let request = store.delete(&JsValue::from(id)).expect("get all error");
        let on_add_error = Closure::once(move |event: &Event| {
            console::log_1(&String::from("删除数据失败").into());
            console::log_1(&event.into());
        });
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        on_add_error.forget();

        let on_success = Closure::once(move |event: &Event| {
            let target = event.target().expect("msg");
            let req = target
                .dyn_ref::<IdbRequest>()
                .expect("Event target is IdbRequest; qed");
            let result = req.result().expect("read result error");
            console::log_1(&result.clone().into());
            console::log_1(&String::from("删除数据成功").into());
            // let _ = tx.send(note);
        });
        request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        on_success.forget();

        // let note = rx.await.unwrap();
        // note
    }

    pub async fn list(&self) -> Vec<Note> {
        let (tx, rx) = oneshot::channel::<Vec<Note>>();

        let transaction = self
            .db
            .transaction_with_str_and_mode(&String::from("note"), IdbTransactionMode::Readwrite)
            .expect("transaction_with_str error");
        let store = transaction
            .object_store(&String::from("note"))
            .expect("store error");

        let rang = IdbKeyRange::bound(&JsValue::from(0), &JsValue::from(100));
        let request = store
            .open_cursor_with_range_and_direction(
                &JsValue::from(&rang.unwrap()),
                web_sys::IdbCursorDirection::Prev,
            )
            .unwrap();
        let on_add_error = Closure::once(move |event: &Event| {
            console::log_1(&String::from("读取数据失败").into());
            console::log_1(&event.into());
        });
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        on_add_error.forget();

        let mut todo_list = Vec::new();
        let mut tx = Some(tx);
        let on_success = Closure::wrap(Box::new(move |event: &Event| {
            let target = event.target().expect("msg");
            let req = target
                .dyn_ref::<IdbRequest>()
                .expect("Event target is IdbRequest; qed");
            let result = match req.result() {
                Ok(data) => data,
                Err(_err) => JsValue::null(),
            };
            // let todo_list_ref = Rc::clone(&todo_list);
            if !result.is_null() {
                // console::log_1(&result.clone().into());
                let db_cursor_with_value = result
                    .dyn_ref::<IdbCursorWithValue>()
                    .expect("db_cursor_with_value error");
                let value = db_cursor_with_value.value().expect("value error");
                let note: Note = serde_wasm_bindgen::from_value(value).expect("msg");
                todo_list.push(note);
                let _ = db_cursor_with_value.continue_();

                // console::log_1(&(*todo_list_ref).borrow_mut().len().into());
            } else {
                let _ = tx.take().unwrap().send(todo_list.to_owned());
            }
        }) as Box<dyn FnMut(&Event)>);
        request.set_onsuccess(Some(on_success.as_ref().unchecked_ref()));
        on_success.forget();

        let list = rx.await.unwrap();
        list
    }

    pub fn save(&self, str: &String) {
        let transaction = self
            .db
            .transaction_with_str_and_mode(&String::from("note"), IdbTransactionMode::Readwrite)
            .expect("transaction_with_str error");
        let store = transaction
            .object_store(&String::from("note"))
            .expect("store error");

        let dt = DateTime::<Utc>::from(chrono::Local::now()); // 表示只在这个里面实现了

        let note = Note {
            id: None,
            content: str.clone(),
            create_time: dt.format(DATE_FORMAT_STR).to_string(),
        };
        let add_request = store
            .add(&serde_wasm_bindgen::to_value(&note).unwrap())
            .expect(&str);

        let on_add_error = Closure::once(move |event: &Event| {
            console::log_1(&String::from("写入数据失败").into());
            console::log_1(&event.into());
        });
        add_request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        on_add_error.forget();

        let on_add_success = Closure::once(move |_event: &Event| {
            console::log_1(&String::from("写入数据成功").into());
        });
        add_request.set_onsuccess(Some(on_add_success.as_ref().unchecked_ref()));
        on_add_success.forget();

        console::log_1(&String::from("do").into());
    }
}
