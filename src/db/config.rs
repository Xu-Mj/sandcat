#![allow(dead_code)]

use std::ops::Deref;

use futures_channel::oneshot;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Event, IdbRequest};
use yew::AttrValue;

use super::{repository::Repository, Config, CONFIG_TABLE_NAME};

pub struct ConfigRepo(Repository);

impl Deref for ConfigRepo {
    type Target = Repository;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

const CONFIG_INDEX_NAME: &str = "name";

impl ConfigRepo {
    pub async fn new() -> Self {
        ConfigRepo(Repository::new().await)
    }

    pub async fn get_conf(&self, name: &str) -> Config {
        let store = self.store(&String::from(CONFIG_TABLE_NAME)).await.unwrap();
        let index = store.index(CONFIG_INDEX_NAME).unwrap();
        let (tx, rx) = oneshot::channel::<Config>();
        let req = index.get(&JsValue::from_str(name)).unwrap();
        let onsuccess = Closure::once(move |event: &Event| {
            let value = event
                .target()
                .unwrap()
                .dyn_ref::<IdbRequest>()
                .unwrap()
                .result()
                .unwrap();
            let mut result = Config::default();
            if !value.is_undefined() && !value.is_null() {
                result = serde_wasm_bindgen::from_value(value).unwrap();
            }
            tx.send(result).unwrap();
        });
        req.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        rx.await.unwrap()
    }

    pub async fn get_config(&self, name: &str) -> AttrValue {
        let store = self.store(&String::from(CONFIG_TABLE_NAME)).await.unwrap();
        let index = store.index(CONFIG_INDEX_NAME).unwrap();
        let (tx, rx) = oneshot::channel::<AttrValue>();
        let req = index.get(&JsValue::from_str(name)).unwrap();
        let onsuccess = Closure::once(move |event: &Event| {
            let value = event
                .target()
                .unwrap()
                .dyn_ref::<IdbRequest>()
                .unwrap()
                .result()
                .unwrap();
            let mut result = AttrValue::default();
            if !value.is_undefined() && !value.is_null() {
                let config: Config = serde_wasm_bindgen::from_value(value).unwrap();
                result = AttrValue::from(config.value);
            }
            tx.send(result).unwrap();
        });
        req.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        rx.await.unwrap()
    }

    // 新增一个配置
    pub async fn put_config(&self, config: &mut Config) {
        let conf = self.get_conf(&config.name).await;
        config.id = conf.id;
        let value = serde_wasm_bindgen::to_value(config).unwrap();
        let store = self.store(&String::from(CONFIG_TABLE_NAME)).await.unwrap();

        let _req = store.put(&value).unwrap();
        let on_add_error = Closure::once(move |event: &Event| {
            web_sys::console::log_1(&String::from("更新配置内容数据失败").into());
            web_sys::console::log_1(&event.into());
        });
        _req.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        on_add_error.forget();
    }
}
