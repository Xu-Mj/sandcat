use futures_channel::oneshot;
use js_sys::Array;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    IdbDatabase, IdbIndexParameters, IdbObjectStore, IdbObjectStoreParameters, IdbRequest,
    IdbTransactionMode,
};
use yew::prelude::*;

use crate::db::{
    CONVERSATION_IS_PINED_WITH_TIME_INDEX, CONVERSATION_LAST_MSG_TIME_INDEX,
    CONVERSATION_TABLE_NAME, FRIENDSHIP_ID_INDEX, FRIENDSHIP_TABLE_NAME, FRIENDSHIP_UNREAD_INDEX,
    FRIEND_ADDRESS_INDEX, FRIEND_GENDER_INDEX, FRIEND_NAME_INDEX, FRIEND_PHONE_INDEX,
    FRIEND_REMARK_INDEX, FRIEND_TABLE_NAME, FRIEND_TIME_INDEX, FRIEND_USER_ID_INDEX,
    GROUP_ID_AND_IS_DELETE, GROUP_ID_AND_USER_ID, GROUP_ID_INDEX, GROUP_MEMBERS_TABLE_NAME,
    GROUP_MSG_TABLE_NAME, GROUP_TABLE_NAME, MESSAGE_CONTENT_INDEX,
    MESSAGE_FRIEND_AND_IS_READ_INDEX, MESSAGE_FRIEND_AND_SEND_TIME_INDEX, MESSAGE_FRIEND_ID_INDEX,
    MESSAGE_ID_INDEX, MESSAGE_IS_READ_INDEX, MESSAGE_TABLE_NAME, MESSAGE_TIME_INDEX,
    MESSAGE_TYPE_INDEX, OFFLINE_TIME_TABLE_NAME, SEQ_TABLE_NAME, USER_TABLE_NAME, VOICE_TABLE_NAME,
};
use crate::error::Result;

use super::DB_NAME;

const DB_VERSION: u32 = 1;

type Func = Option<Closure<dyn FnMut(&Event)>>;

#[derive(Debug)]
pub struct Repository {
    db: IdbDatabase,
    onupgread: Func,
    onsuccess: Func,
}

impl Clone for Repository {
    fn clone(&self) -> Self {
        Self {
            db: self.db.clone(),
            onupgread: None,
            onsuccess: None,
        }
    }
}

impl Drop for Repository {
    fn drop(&mut self) {
        self.onsuccess = None;
        self.onupgread = None;
        self.db.close();
    }
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
            parameters.key_path(Some(&JsValue::from_str("local_id")));

            db.create_object_store_with_optional_parameters(
                &String::from(VOICE_TABLE_NAME),
                &parameters,
            )
            .unwrap();

            let mut parameters: IdbObjectStoreParameters = IdbObjectStoreParameters::new();
            parameters.key_path(Some(&JsValue::from_str("id")));
            parameters.auto_increment(true);

            db.create_object_store_with_optional_parameters(
                &String::from(USER_TABLE_NAME),
                &parameters,
            )
            .unwrap();
            // store.create_index_with_str("login", "login").unwrap();
            create_msg_table(&db, MESSAGE_TABLE_NAME).expect("create message table panic");
            create_msg_table(&db, GROUP_MSG_TABLE_NAME).expect("create group message table panic");
            create_friend_table(&db).expect("create friend table panic");
            create_friendship_table(&db).expect("create friendship table panic");
            create_group_members_table(&db).expect("create group members table panic");
            create_conv_table(&db).expect("create conversations table panic");

            // create sequence table
            db.create_object_store_with_optional_parameters(
                &String::from(SEQ_TABLE_NAME),
                &parameters,
            )
            .unwrap();

            let mut id_key: IdbObjectStoreParameters = IdbObjectStoreParameters::new();
            id_key.key_path(Some(&JsValue::from_str("id")));

            // create groups table
            db.create_object_store_with_optional_parameters(
                &String::from(GROUP_TABLE_NAME),
                &id_key,
            )
            .unwrap();

            // create offline time table
            db.create_object_store_with_optional_parameters(
                &String::from(OFFLINE_TIME_TABLE_NAME),
                &parameters,
            )
            .unwrap();
        });
        open_request.set_onupgradeneeded(Some(on_upgradeneeded.as_ref().unchecked_ref()));
        // on_upgradeneeded.forget();

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
        // on_success.forget();

        let db = rx.await.unwrap();

        Repository {
            db,
            onupgread: Some(on_upgradeneeded),
            onsuccess: Some(on_success),
        }
    }

    pub async fn store(&self, name: &str) -> Result<IdbObjectStore> {
        let transaction = self
            .db
            .transaction_with_str_and_mode(name, IdbTransactionMode::Readwrite)?;
        Ok(transaction.object_store(name)?)
    }

    pub async fn delete_db() {
        let db_name = DB_NAME.get().unwrap();

        let window = web_sys::window().unwrap();
        // 获取indexedDB对象
        let idb_factory = window.indexed_db().unwrap().unwrap();
        idb_factory.delete_database(db_name).unwrap();
    }
}

fn create_msg_table(db: &IdbDatabase, table_name: &str) -> Result<()> {
    // use local_id as primary key
    let mut parameter = IdbObjectStoreParameters::new();
    parameter.key_path(Some(&JsValue::from(MESSAGE_ID_INDEX)));

    let store =
        db.create_object_store_with_optional_parameters(&String::from(table_name), &parameter)?;

    let indexes = Array::new();
    indexes.push(&JsValue::from("friend_id"));
    indexes.push(&JsValue::from("send_time"));
    let indexes = JsValue::from(indexes);
    store.create_index_with_str_sequence(MESSAGE_FRIEND_AND_SEND_TIME_INDEX, &indexes)?;

    let indexes = Array::new();
    indexes.push(&JsValue::from("friend_id"));
    indexes.push(&JsValue::from("is_read"));
    let indexes = JsValue::from(indexes);
    store.create_index_with_str_sequence(MESSAGE_FRIEND_AND_IS_READ_INDEX, &indexes)?;

    store.create_index_with_str(MESSAGE_FRIEND_ID_INDEX, "friend_id")?;
    store.create_index_with_str(MESSAGE_CONTENT_INDEX, "content")?;
    store.create_index_with_str(MESSAGE_TIME_INDEX, "create_time")?;
    store.create_index_with_str(MESSAGE_TYPE_INDEX, "content_type")?;
    store.create_index_with_str(MESSAGE_IS_READ_INDEX, "is_read")?;
    Ok(())
}

fn create_conv_table(db: &IdbDatabase) -> Result<()> {
    // use friend_id as primary key
    let mut parameters: IdbObjectStoreParameters = IdbObjectStoreParameters::new();
    parameters.key_path(Some(&JsValue::from_str("friend_id")));
    let store = db.create_object_store_with_optional_parameters(
        &String::from(CONVERSATION_TABLE_NAME),
        &parameters,
    )?;

    store.create_index_with_str(CONVERSATION_LAST_MSG_TIME_INDEX, "last_msg_time")?;

    let indexes = Array::new();
    indexes.push(&JsValue::from("is_pined"));
    indexes.push(&JsValue::from("last_msg_time"));
    let indexes = JsValue::from(indexes);
    store.create_index_with_str_sequence(CONVERSATION_IS_PINED_WITH_TIME_INDEX, &indexes)?;

    Ok(())
}

fn create_group_members_table(db: &IdbDatabase) -> Result<()> {
    // create group_members table
    let indexes = Array::new();
    indexes.push(&JsValue::from("user_id"));
    indexes.push(&JsValue::from("group_id"));
    let indexes = JsValue::from(indexes);

    let mut parameter = IdbObjectStoreParameters::new();
    parameter.key_path(Some(&indexes));
    let store = db.create_object_store_with_optional_parameters(
        &String::from(GROUP_MEMBERS_TABLE_NAME),
        &parameter,
    )?;

    store.create_index_with_str(GROUP_ID_INDEX, "group_id")?;

    // create composite index
    let mut param: IdbIndexParameters = IdbIndexParameters::new();
    param.unique(true);
    store.create_index_with_str_sequence_and_optional_parameters(
        GROUP_ID_AND_USER_ID,
        &indexes,
        &param,
    )?;

    // create group_id and is_delete index
    let indexes = Array::new();
    indexes.push(&JsValue::from("group_id"));
    indexes.push(&JsValue::from("is_deleted"));
    let indexes = JsValue::from(indexes);
    store.create_index_with_str_sequence(GROUP_ID_AND_IS_DELETE, &indexes)?;

    Ok(())
}

fn create_friendship_table(db: &IdbDatabase) -> Result<()> {
    // create friendship table and index
    let mut parameter = IdbObjectStoreParameters::new();
    parameter.key_path(Some(&JsValue::from(FRIENDSHIP_ID_INDEX)));
    let store = db.create_object_store_with_optional_parameters(
        &String::from(FRIENDSHIP_TABLE_NAME),
        &parameter,
    )?;

    let mut param: IdbIndexParameters = IdbIndexParameters::new();
    param.unique(true);
    store.create_index_with_str_and_optional_parameters(
        FRIENDSHIP_ID_INDEX,
        "friendship_id",
        &param,
    )?;

    store.create_index_with_str(FRIEND_USER_ID_INDEX, "user_id")?;
    store.create_index_with_str(FRIENDSHIP_UNREAD_INDEX, "read")?;

    Ok(())
}

fn create_friend_table(db: &IdbDatabase) -> Result<()> {
    // create friend table and index
    let mut p = IdbObjectStoreParameters::new();
    p.key_path(Some(&JsValue::from_str("friend_id")));
    let store =
        db.create_object_store_with_optional_parameters(&String::from(FRIEND_TABLE_NAME), &p)?;

    store.create_index_with_str(FRIEND_NAME_INDEX, "name")?;
    store.create_index_with_str(FRIEND_REMARK_INDEX, "remark")?;
    store.create_index_with_str(FRIEND_GENDER_INDEX, "gender")?;
    store.create_index_with_str(FRIEND_PHONE_INDEX, "phone")?;
    store.create_index_with_str(FRIEND_ADDRESS_INDEX, "address")?;
    store.create_index_with_str(FRIEND_TIME_INDEX, "update_time")?;

    Ok(())
}
