use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use indexmap::IndexMap;
use log::error;
use wasm_bindgen::closure::Closure;
use yew::{AttrValue, Event};

use crate::db::group_msg::GroupMessages;
use crate::error::Result;
use crate::model::message::{Message, ServerResponse};

use super::message::{
    add, delete_batch, get, get_last_msg, get_messages, update_msg_status, update_read_status,
};
use super::SuccessCallback;
use super::{repository::Repository, GROUP_MSG_TABLE_NAME};

#[derive(Debug)]
pub struct GroupMsgRepo {
    repo: Repository,
    on_err_callback: Closure<dyn FnMut(&Event)>,
    on_get_list_success: SuccessCallback,
    on_batch_del_success: SuccessCallback,
    on_update_state_success: SuccessCallback,
    on_update_success: SuccessCallback,
}

impl Deref for GroupMsgRepo {
    type Target = Repository;

    fn deref(&self) -> &Self::Target {
        &self.repo
    }
}

impl GroupMsgRepo {
    pub fn new(repo: Repository) -> Self {
        let on_err_callback =
            Closure::once(move |event: &Event| error!("group message operate error: {:?}", event));

        Self {
            repo,
            on_err_callback,
            on_update_success: Rc::new(RefCell::new(None)),
            on_get_list_success: Rc::new(RefCell::new(None)),
            on_batch_del_success: Rc::new(RefCell::new(None)),
            on_update_state_success: Rc::new(RefCell::new(None)),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl GroupMessages for GroupMsgRepo {
    async fn put(&self, msg: &Message) -> Result<()> {
        let store = self.store(GROUP_MSG_TABLE_NAME).await?;
        add(store, msg, &self.on_err_callback).await
    }

    /// friend id is group id
    /// send id is group member id
    async fn get_messages(
        &self,
        friend_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<IndexMap<AttrValue, Message>> {
        let store = self.store(GROUP_MSG_TABLE_NAME).await?;
        let (result, onsuccess) =
            get_messages(store, friend_id, page, page_size, &self.on_err_callback).await?;
        *self.on_get_list_success.borrow_mut() = Some(onsuccess);
        Ok(result)
    }

    async fn get_last_msg(&self, group_id: &str) -> Result<Option<Message>> {
        let store = self.store(GROUP_MSG_TABLE_NAME).await?;
        get_last_msg(store, group_id, &self.on_err_callback).await
    }

    async fn get(&self, local_id: &str) -> Result<Option<Message>> {
        let store = self.store(GROUP_MSG_TABLE_NAME).await?;
        get(store, local_id).await
    }

    async fn update_msg_status(&self, msg: &ServerResponse) -> Result<()> {
        let store = self.store(GROUP_MSG_TABLE_NAME).await?;
        let onsuccess = update_msg_status(store, msg, &self.on_err_callback).await?;
        *self.on_update_state_success.borrow_mut() = Some(onsuccess);
        Ok(())
    }

    async fn update_read_status(&self, friend_id: &str) -> Result<Vec<i64>> {
        let store = self.store(GROUP_MSG_TABLE_NAME).await?;
        let (seq, success) = update_read_status(store, friend_id, &self.on_err_callback).await?;
        *self.on_update_success.borrow_mut() = Some(success);

        Ok(seq)
    }

    async fn delete_batch(&self, group_id: &str) -> Result<()> {
        let store = self.store(GROUP_MSG_TABLE_NAME).await?;

        let onsuccess = delete_batch(store, group_id).await?;
        *self.on_batch_del_success.borrow_mut() = Some(onsuccess);

        Ok(())
    }
}
