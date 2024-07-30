use std::ops::Deref;

use futures_channel::oneshot;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::IdbRequest;
use yew::Event;

use crate::{db::offline_time::OfflineTimes, error::Result, model::offline_time::OfflineTime};

use super::{repository::Repository, OFFLINE_TIME_TABLE_NAME};

const OFFLINE_TIME_ID: i32 = 1;

#[derive(Debug)]
pub struct OfflineTimeRepo {
    repo: Repository,
}

impl Deref for OfflineTimeRepo {
    type Target = Repository;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl OfflineTimeRepo {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }
}

#[async_trait::async_trait(?Send)]
impl OfflineTimes for OfflineTimeRepo {
    async fn save(&self, time: i64) -> Result<()> {
        let db = self.store(OFFLINE_TIME_TABLE_NAME).await?;
        db.put(&serde_wasm_bindgen::to_value(&OfflineTime::new(time)).unwrap())?;
        Ok(())
    }

    async fn get(&self) -> Result<Option<OfflineTime>> {
        let db = self.store(OFFLINE_TIME_TABLE_NAME).await?;
        let request = db.get(&JsValue::from(OFFLINE_TIME_ID))?;

        let (tx, rx) = oneshot::channel::<Option<OfflineTime>>();
        let onsuccess = Closure::once(move |event: &Event| {
            let result = event
                .target()
                .unwrap()
                .dyn_ref::<IdbRequest>()
                .unwrap()
                .result()
                .unwrap();
            if !result.is_undefined() && !result.is_null() {
                let obj = serde_wasm_bindgen::from_value(result).unwrap();
                tx.send(Some(obj)).unwrap();
            } else {
                tx.send(None).unwrap();
            }
        });
        request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        Ok(rx.await.unwrap())
    }

    async fn del(&self) -> Result<()> {
        let store = self.store(OFFLINE_TIME_TABLE_NAME).await?;
        store.delete(&JsValue::from(OFFLINE_TIME_ID))?;
        Ok(())
    }
}
