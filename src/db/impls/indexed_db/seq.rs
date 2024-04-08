use std::ops::Deref;

use async_trait::async_trait;
use futures_channel::oneshot;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::IdbRequest;
use yew::Event;

use crate::{db::seq::SeqInterface, model::seq::Seq};

use super::{repository::Repository, SEQ_TABLE_NAME};

pub struct SeqRepo(Repository);
impl Deref for SeqRepo {
    type Target = Repository;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl SeqRepo {
    pub async fn new() -> Self {
        Self(Repository::new().await)
    }
}

const ID: i64 = 1;

#[async_trait(?Send)]
impl SeqInterface for SeqRepo {
    async fn put(&self, seq: &Seq) -> Result<(), JsValue> {
        let store = self.store(SEQ_TABLE_NAME).await?;
        let value = serde_wasm_bindgen::to_value(seq)?;
        store.put(&value)?;
        Ok(())
    }
    async fn get(&self) -> Result<Seq, JsValue> {
        let (tx, rx) = oneshot::channel::<Seq>();
        let store = self.store(SEQ_TABLE_NAME).await?;
        let request = store.get(&JsValue::from(ID))?;
        let onsuccess = Closure::once(move |event: &Event| {
            let result = event
                .target()
                .unwrap()
                .dyn_ref::<IdbRequest>()
                .unwrap()
                .result()
                .unwrap();
            let mut seq = Seq::default();
            if !result.is_undefined() && !result.is_null() {
                seq = serde_wasm_bindgen::from_value(result).unwrap();
            }
            tx.send(seq).unwrap();
        });
        request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        let on_add_error = Closure::once(move |event: &Event| {
            web_sys::console::log_1(&String::from("读取数据失败").into());
            web_sys::console::log_1(&event.into());
        });
        request.set_onerror(Some(on_add_error.as_ref().unchecked_ref()));
        Ok(rx.await.unwrap())
    }
}
