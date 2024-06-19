use std::ops::Deref;

use futures_channel::oneshot;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::IdbRequest;
use yew::Event;

use crate::{db::voice::Voices, error::Result, model::voice::Voice};

use super::{repository::Repository, VOICE_TABLE_NAME};

#[derive(Debug)]
pub struct VoiceRepo {
    repo: Repository,
    // on_get_success: Option<Closure<dyn FnOnce(Event)>>,
}

impl Deref for VoiceRepo {
    type Target = Repository;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl VoiceRepo {
    pub fn new(repo: Repository) -> Self {
        Self { repo }
    }
}

#[async_trait::async_trait(?Send)]
impl Voices for VoiceRepo {
    async fn save(&self, voice: &Voice) -> Result<()> {
        let db = self.store(VOICE_TABLE_NAME).await?;
        db.put(&serde_wasm_bindgen::to_value(voice).unwrap())?;
        Ok(())
    }

    async fn get(&self, local_id: &str) -> Result<Voice> {
        let db = self.store(VOICE_TABLE_NAME).await?;
        let request = db.get(&JsValue::from(local_id))?;

        let (tx, rx) = oneshot::channel::<Voice>();
        let onsuccess = Closure::once(move |event: &Event| {
            let result = event
                .target()
                .unwrap()
                .dyn_ref::<IdbRequest>()
                .unwrap()
                .result()
                .unwrap();
            let mut voice = Voice::default();
            if !result.is_undefined() && !result.is_null() {
                voice = serde_wasm_bindgen::from_value(result).unwrap();
            }
            tx.send(voice).unwrap();
        });
        request.set_onsuccess(Some(onsuccess.as_ref().unchecked_ref()));
        // if self.on_get_success.is_none() {
        // self.on_get_success = Some(onsuccess);
        // }
        Ok(rx.await.unwrap())
    }

    async fn del(&self, local_id: &str) -> Result<()> {
        let store = self.store(VOICE_TABLE_NAME).await?;
        store.delete(&JsValue::from(local_id))?;
        Ok(())
    }
}
