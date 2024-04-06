use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CloseEvent, ErrorEvent, MessageEvent, WebSocket};
use yew::Callback;

use crate::model::message::Msg;
use crate::ws::convert;
// 定义WebSocket管理器结构体
pub struct WebSocketManager {
    url: String,
    ws: Option<WebSocket>,
    reconnect_attempts: u32,
    max_reconnect_attempts: u32,
    reconnect_interval: i32,
    receive_callback: Callback<Msg>,
    // 为了防止内存泄露
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
    // 创建新的WebSocket管理器实例
    pub fn new(url: String, receive_callback: Callback<Msg>) -> Self {
        Self {
            url,
            ws: None,
            reconnect_attempts: 0,
            max_reconnect_attempts: 5,
            reconnect_interval: 1000, // 初始重连间隔为1000毫秒
            receive_callback,
            on_open: None,
            on_close: None,
            on_error: None,
            on_message: None,
        }
    }

    // 初始化WebSocket连接
    pub fn connect(ws_manager: Rc<RefCell<Self>>) {
        let ws = WebSocket::new(ws_manager.borrow().url.as_str()).unwrap();
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
        let cloned_ws = ws_manager.clone();
        let on_open = Closure::wrap(Box::new(move || {
            log::info!("WebSocket connection opened");
            // 链接成功要把重连次数归零
            cloned_ws.borrow_mut().reconnect_attempts = 0;
        }) as Box<dyn FnMut()>);
        // ON_OPEN.get_or_init(on_open);
        let ws_manager_clone = ws_manager.clone();
        let on_message = Closure::wrap(Box::new(move |e: MessageEvent| {
            web_sys::console::log_1(&e.data());
            if let Ok(ab) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                let arr = js_sys::Uint8Array::new(&ab);
                let mut body = vec![0; arr.length() as usize];
                arr.copy_to(&mut body[..]);
                match bincode::deserialize(&body) {
                    Ok(msg) => match convert(msg) {
                        Ok(msg) => ws_manager_clone.borrow_mut().receive_callback.emit(msg),
                        Err(e) => log::error!("convert message error {e}"),
                    },
                    Err(err) => log::error!("反序列化消息失败: {:?}", err),
                }
            } else if let Ok(blob) = e.data().dyn_into::<web_sys::Blob>() {
                // 如果消息是一个Blob，我们需要将它先转换为ArrayBuffer
                // 然后再按同样的方式处理
                log::info!("Message received as a Blob, size: {}", blob.size());
                web_sys::console::log_1(&blob);
                let arr = js_sys::Uint8Array::new(&blob);
                let mut body = vec![0; arr.length() as usize];
                arr.copy_to(&mut body[..]);
                match bincode::deserialize(&body) {
                    Ok(msg) => match convert(msg) {
                        Ok(msg) => ws_manager_clone.borrow_mut().receive_callback.emit(msg),
                        Err(e) => log::error!("convert message error {e}"),
                    },
                    Err(err) => log::error!("反序列化消息失败: {:?}", err),
                }
                // 要做的操作...
            } else {
                log::error!("Unexpected message format!")
            }

            /*
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                if let Some(msg) = txt.as_string() {
                    log::info!("Message received: {}", msg.clone());
                    let result = serde_json::from_str(&msg);
                    match result {
                        Ok(msg) => match convert(msg) {
                            Ok(msg) => ws_manager_clone.borrow_mut().receive_callback.emit(msg),
                            Err(e) => log::error!("convert message error {e}"),
                        },
                        Err(err) => log::error!("反序列化消息失败: {:?}", err),
                    }
                }
            } */
        }) as Box<dyn FnMut(MessageEvent)>);

        let on_error = Closure::wrap(Box::new(move |e: ErrorEvent| {
            log::error!("WebSocket error: {:?}", e);
        }) as Box<dyn FnMut(ErrorEvent)>);
        let ws_manager_clone = ws_manager.clone();
        let on_close = Closure::wrap(Box::new(move |e: CloseEvent| {
            log::warn!("WebSocket closed: {:?}", e);
            // 重连逻辑
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

    // 发送消息
    pub fn send_message(&self, message: &Msg) -> Result<(), JsValue> {
        if let Some(ws) = &self.ws {
            // encode message
            let msg =
                bincode::serialize(message).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
            ws.send_with_u8_array(&msg)
        } else {
            Err(JsValue::from_str("websocket is none"))
        }
    }

    // 重连逻辑
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
        }
    }
    // 清理WebSocket连接和事件监听器
    pub fn cleanup(&mut self) {
        if let Some(ws) = self.ws.take() {
            log::debug!("WebSocket connection closing...");
            let _ = ws
                .close()
                .map_err(|err| log::error!("关闭WebSocket连接出错: {:?}", err));
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
