use std::cell::RefCell;
use std::rc::Rc;

use sandcat_sdk::db::TOKEN;
use sandcat_sdk::state::ConnectState;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CloseEvent, ErrorEvent, MessageEvent, WebSocket};
use yew::Callback;

use sandcat_sdk::model::message::convert_server_msg;
use sandcat_sdk::model::message::Msg;
use sandcat_sdk::pb::message::Msg as PbMsg;
use yewdux::Dispatch;

const KNOCKOFF_CODE: u16 = 4001;
pub const UNAUTHORIZED_CODE: u16 = 4002;

#[derive(Debug)]
pub struct WebSocketManager {
    url: String,
    ws: Option<WebSocket>,
    reconnect_attempts: u32,
    max_reconnect_attempts: u32,
    reconnect_interval: i32,
    receive_callback: Callback<Msg>,
    knockoff_callback: Callback<()>,
    logout_callback: Callback<()>,
    // prevent memory leaks
    on_open: Option<Closure<dyn FnMut()>>,
    on_close: Option<Closure<dyn FnMut(CloseEvent)>>,
    on_error: Option<Closure<dyn FnMut(ErrorEvent)>>,
    on_message: Option<Closure<dyn FnMut(MessageEvent)>>,
}

impl PartialEq for WebSocketManager {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
    }
}

impl WebSocketManager {
    pub fn new(
        url: String,
        receive_callback: Callback<Msg>,
        knockoff_callback: Callback<()>,
        logout_callback: Callback<()>,
    ) -> Self {
        Self {
            url,
            ws: None,
            reconnect_attempts: 0,
            max_reconnect_attempts: 5,
            reconnect_interval: 1000, // 初始重连间隔为1000毫秒
            receive_callback,
            knockoff_callback,
            logout_callback,
            on_open: None,
            on_close: None,
            on_error: None,
            on_message: None,
        }
    }

    // 初始化WebSocket连接
    pub fn connect(ws_manager: Rc<RefCell<Self>>) {
        // sentence the ws is connected
        if ws_manager.borrow().ws.is_some()
            && ws_manager.borrow().ws.as_ref().unwrap().ready_state() == WebSocket::OPEN
        {
            return;
        }

        let ws = WebSocket::new(
            format!(
                "{}/{}",
                ws_manager.borrow().url,
                utils::get_local_storage(TOKEN).unwrap()
            )
            .as_str(),
        )
        .unwrap();

        // send connecting state
        Dispatch::<ConnectState>::global().set(ConnectState::Connecting);
        // set default binary type
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
        let cloned_ws = ws_manager.clone();

        let on_open = Closure::wrap(Box::new(move || {
            log::info!("WebSocket connection opened");
            // set the count of reconnect to 0
            cloned_ws.borrow_mut().reconnect_attempts = 0;
            Dispatch::<ConnectState>::global().set(ConnectState::Connected);
        }) as Box<dyn FnMut()>);

        let ws_manager_clone = ws_manager.clone();
        let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(ab) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                let arr = js_sys::Uint8Array::new(&ab);
                let mut body = vec![0; arr.length() as usize];
                arr.copy_to(&mut body[..]);

                match bincode::deserialize(&body) {
                    Ok(msg) => match convert_server_msg(msg) {
                        Ok(msg) => ws_manager_clone.borrow_mut().receive_callback.emit(msg),
                        Err(e) => log::error!("convert message error {e}"),
                    },
                    Err(err) => log::error!("deserialize error: {:?}", err),
                }
            } else if let Ok(blob) = e.data().dyn_into::<web_sys::Blob>() {
                // if message type is we need to convert it to ArrayBuffer
                log::info!("Message received as a Blob, size: {}", blob.size());
                let arr = js_sys::Uint8Array::new(&blob);
                let mut body = vec![0; arr.length() as usize];
                arr.copy_to(&mut body[..]);

                match bincode::deserialize(&body) {
                    Ok(msg) => match convert_server_msg(msg) {
                        Ok(msg) => ws_manager_clone.borrow().receive_callback.emit(msg),
                        Err(e) => log::error!("convert message error {e}"),
                    },
                    Err(err) => log::error!("deserialize error: {:?}", err),
                }
            } else {
                log::error!("Unexpected message format!")
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        let on_error = Closure::wrap(Box::new(move |e: ErrorEvent| {
            log::error!("WebSocket error: {:?}", e.message());
            Dispatch::<ConnectState>::global().set(ConnectState::DisConnect);
        }) as Box<dyn FnMut(ErrorEvent)>);

        let ws_manager_clone = ws_manager.clone();
        let on_close = Closure::wrap(Box::new(move |e: CloseEvent| {
            match e.code() {
                KNOCKOFF_CODE => {
                    log::info!("Knocked off by another client");
                    ws_manager_clone.borrow().knockoff_callback.emit(());
                    return;
                }
                UNAUTHORIZED_CODE => {
                    log::warn!("Unauthorized access");
                    // todo need to reauthorize
                    ws_manager_clone.borrow().logout_callback.emit(());
                    return;
                }
                _ => {
                    log::warn!("WebSocket closed: {:?}", e);
                }
            }
            // reconnect
            ws_manager_clone
                .borrow_mut()
                .reconnect(ws_manager_clone.clone());
        }) as Box<dyn FnMut(CloseEvent)>);

        ws.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        ws.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        ws.set_onerror(Some(on_error.as_ref().unchecked_ref()));
        ws.set_onclose(Some(on_close.as_ref().unchecked_ref()));

        let mut manager = ws_manager.borrow_mut();
        // 将ws以及事件函数保存到manager对象上，直接使用事件函数.forget()造成内存泄露
        manager.ws = Some(ws);
        manager.on_open = Some(on_open);
        manager.on_close = Some(on_close);
        manager.on_message = Some(on_message);
        manager.on_error = Some(on_error);
    }

    pub fn send_message(&self, message: Msg) -> Result<(), JsValue> {
        if let Some(ws) = &self.ws {
            // encode message
            let msg = bincode::serialize(&PbMsg::from(message))
                .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
            ws.send_with_u8_array(&msg)
        } else {
            Err(JsValue::from_str("websocket is none"))
        }
    }

    fn reconnect(&mut self, ws_manager: Rc<RefCell<Self>>) {
        log::debug!("第{}次重连", self.reconnect_attempts);
        if self.reconnect_attempts < self.max_reconnect_attempts {
            self.reconnect_attempts += 1;
            let interval = self.reconnect_interval * self.reconnect_attempts as i32;
            let window = web_sys::window().unwrap();
            let closure = Closure::wrap(Box::new(move || {
                WebSocketManager::connect(ws_manager.clone());
            }) as Box<dyn FnMut()>);

            window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    closure.as_ref().unchecked_ref(),
                    interval,
                )
                .unwrap();

            closure.forget();
        } else {
            log::error!("Reached maximum reconnect attempts");
            Dispatch::<ConnectState>::global().set(ConnectState::DisConnect);
        }
    }

    // clean WebSocket connection and events
    pub fn cleanup(&mut self) {
        if let Some(ws) = self.ws.take() {
            log::debug!("WebSocket connection closing...");
            let _ = ws
                .close()
                .map_err(|err| log::error!("close WebSocket error: {:?}", err));
            ws.set_onopen(None);
            ws.set_onmessage(None);
            ws.set_onerror(None);
            ws.set_onclose(None);
        }

        drop(self.on_open.take());
        drop(self.on_close.take());
        drop(self.on_message.take());
        drop(self.on_error.take());
    }
}
