use futures_channel::oneshot;
use js_sys::Array;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    IdbDatabase, IdbIndexParameters, IdbObjectStore, IdbObjectStoreParameters, IdbRequest,
    IdbTransactionMode,
};
use yew::prelude::*;

use crate::{
    CONVERSATION_LAST_MSG_TIME_INDEX, CONVERSATION_TABLE_NAME, FRIENDSHIP_ID_INDEX,
    FRIENDSHIP_TABLE_NAME, FRIENDSHIP_UNREAD_INDEX, FRIEND_ADDRESS_INDEX, FRIEND_GENDER_INDEX,
    FRIEND_NAME_INDEX, FRIEND_PHONE_INDEX, FRIEND_REMARK_INDEX, FRIEND_TABLE_NAME,
    FRIEND_TIME_INDEX, FRIEND_USER_ID_INDEX, GROUP_ID_AND_USER_ID, GROUP_ID_INDEX,
    GROUP_MEMBERS_TABLE_NAME, GROUP_MSG_TABLE_NAME, GROUP_TABLE_NAME, MESSAGE_CONTENT_INDEX,
    MESSAGE_FRIEND_AND_SEND_TIME_INDEX, MESSAGE_FRIEND_ID_INDEX, MESSAGE_ID_INDEX,
    MESSAGE_IS_READ_INDEX, MESSAGE_TABLE_NAME, MESSAGE_TIME_INDEX, MESSAGE_TYPE_INDEX,
    SEQ_TABLE_NAME, USER_TABLE_NAME,
};

use super::DB_NAME;

// const DATE_FORMAT_STR: &str = "%Y-%m-%d %H:%M:%S";
const DB_VERSION: u32 = 1;
#[derive(Debug, Clone)]
pub struct Repository {
    db: IdbDatabase,
}

impl Repository {
    pub async fn new() -> Repository {
        let db_name = DB_NAME.get().unwrap();
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

            db.create_object_store_with_optional_parameters(
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
                .create_index_with_str_and_optional_parameters(MESSAGE_ID_INDEX, "local_id", &param)
                .unwrap();
            let indexes = Array::new();
            indexes.push(&JsValue::from("friend_id"));
            indexes.push(&JsValue::from("send_time"));
            let indexes = JsValue::from(indexes);
            store
                .create_index_with_str_sequence(MESSAGE_FRIEND_AND_SEND_TIME_INDEX, &indexes)
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
                    &String::from(GROUP_MSG_TABLE_NAME),
                    &parameters,
                )
                .unwrap();
            let mut param: IdbIndexParameters = IdbIndexParameters::new();
            param.unique(true);
            store
                .create_index_with_str_and_optional_parameters(MESSAGE_ID_INDEX, "local_id", &param)
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
            let mut parameters: IdbObjectStoreParameters = IdbObjectStoreParameters::new();
            parameters.key_path(Some(&JsValue::from_str("friend_id")));
            let store = db
                .create_object_store_with_optional_parameters(
                    &String::from(CONVERSATION_TABLE_NAME),
                    &parameters,
                )
                .unwrap(); /* let store = db
                           .create_object_store_with_optional_parameters(
                               &String::from(CONVERSATION_TABLE_NAME),
                               &parameters,
                           )
                           .unwrap(); */
            // store
            //     .create_index_with_str(CONVERSATION_FRIEND_ID_INDEX, "friend_id")
            //     .unwrap();
            store
                .create_index_with_str(CONVERSATION_LAST_MSG_TIME_INDEX, "last_msg_time")
                .unwrap();

            let mut parameters: IdbObjectStoreParameters = IdbObjectStoreParameters::new();
            parameters.key_path(Some(&JsValue::from_str("id")));
            parameters.auto_increment(true);
            let _store = db
                .create_object_store_with_optional_parameters(
                    &String::from(SEQ_TABLE_NAME),
                    &parameters,
                )
                .unwrap();
            let _store = db
                .create_object_store_with_optional_parameters(
                    &String::from(GROUP_TABLE_NAME),
                    &parameters,
                )
                .unwrap();

            let store = db
                .create_object_store_with_optional_parameters(
                    &String::from(GROUP_MEMBERS_TABLE_NAME),
                    &parameters,
                )
                .unwrap();
            store
                .create_index_with_str(GROUP_ID_INDEX, "group_id")
                .unwrap();
            // create multipal index
            let indexes = Array::new();
            indexes.push(&JsValue::from("user_id"));
            indexes.push(&JsValue::from("group_id"));
            let indexes = JsValue::from(indexes);
            store
                .create_index_with_str_sequence(GROUP_ID_AND_USER_ID, &indexes)
                .unwrap();
            let mut parameter = IdbObjectStoreParameters::new();
            parameter.key_path(Some(&JsValue::from(FRIENDSHIP_ID_INDEX)));
            let store = db
                .create_object_store_with_optional_parameters(
                    &String::from(FRIENDSHIP_TABLE_NAME),
                    &parameter,
                )
                .unwrap();

            store
                .create_index_with_str_and_optional_parameters(
                    FRIENDSHIP_ID_INDEX,
                    "friendship_id",
                    &param,
                )
                .unwrap();
            store
                .create_index_with_str(FRIEND_USER_ID_INDEX, "user_id")
                .unwrap();
            store
                .create_index_with_str(FRIENDSHIP_UNREAD_INDEX, "read")
                .unwrap();

            let mut p = IdbObjectStoreParameters::new();
            p.key_path(Some(&JsValue::from_str("friend_id")));
            let store = db
                .create_object_store_with_optional_parameters(&String::from(FRIEND_TABLE_NAME), &p)
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

        Repository { db }
    }

    pub async fn store(&self, name: &str) -> Result<IdbObjectStore, JsValue> {
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
}
