use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    RtcConfiguration, RtcIceConnectionState, RtcIceServer, RtcPeerConnection,
    RtcSessionDescriptionInit, RtcSignalingState,
};
use yew::platform::spawn_local;
use yew::{AttrValue, Callback};

use sandcat_sdk::model::message::{Candidate, Msg, Offer, SingleCall};
use ws::WebSocketManager;

pub struct WebRTC {
    on_ice_candidate: Option<Closure<dyn FnMut(web_sys::RtcPeerConnectionIceEvent)>>,
    on_track: Option<Closure<dyn FnMut(web_sys::RtcTrackEvent)>>,
    on_ice_connection_state_change: Option<Closure<dyn FnMut()>>,
    on_ice_gathering_state_change: Option<Closure<dyn FnMut()>>,
    on_signaling_state_change: Option<Closure<dyn FnMut()>>,
    on_negotiation: Option<Closure<dyn FnMut()>>,
    pc: Option<RtcPeerConnection>,
    close_event: Callback<()>,
    conn_event: Callback<web_sys::RtcTrackEvent>,
}

impl WebRTC {
    pub fn new(close_event: Callback<()>, conn_event: Callback<web_sys::RtcTrackEvent>) -> Self {
        Self {
            on_ice_candidate: None,
            on_track: None,
            on_ice_connection_state_change: None,
            on_ice_gathering_state_change: None,
            on_signaling_state_change: None,
            on_negotiation: None,
            pc: None,
            close_event,
            conn_event,
        }
    }

    pub fn pc(&self) -> &RtcPeerConnection {
        self.pc.as_ref().unwrap()
    }

    pub fn close(&mut self) {
        if let Some(ref pc) = self.pc {
            pc.close();
            self.pc = None;
        }
        self.on_negotiation = None;
        self.on_signaling_state_change = None;
        self.on_ice_candidate = None;
        self.on_ice_connection_state_change = None;
        self.on_ice_gathering_state_change = None;
        self.on_track = None;
    }
    pub fn send_msg1(ws: Rc<RefCell<WebSocketManager>>, msg: Msg) {
        // 发送已收到消息给服务器
        match ws.borrow().send_message(msg) {
            Ok(_) => { /*log::info!("发送成功:{:?}", &msg)*/ }
            Err(e) => {
                log::error!("发送失败: {:?}", e)
            }
        };
    }

    pub fn create_pc(
        &mut self,
        ws: Rc<RefCell<WebSocketManager>>,
        send_id: AttrValue,
        friend_id: AttrValue,
    ) -> Result<(), JsValue> {
        // 创建一个新的RtcIceServer
        let mut ice_server = RtcIceServer::new();
        ice_server.url("stun:localhost:3478");

        // 将RtcIceServer添加到RtcConfiguration中
        let ice_servers = js_sys::Array::from(&ice_server);
        let mut config = RtcConfiguration::new();
        config.ice_servers(&ice_servers);

        // 使用配置创建新的RtcPeerConnection
        let pc = RtcPeerConnection::new_with_configuration(&config)?;
        let send = send_id.clone();
        let friend = friend_id.clone();
        let ws_clone = ws.clone();
        // 设定处理函数
        let on_ice_candidate =
            Closure::wrap(Box::new(move |event: web_sys::RtcPeerConnectionIceEvent| {
                // handle ICE candidate event
                if let Some(candidate) = event.candidate() {
                    let msg_clone = Msg::SingleCall(SingleCall::NewIceCandidate(Candidate {
                        candidate: candidate.candidate().into(),
                        sdp_mid: candidate.sdp_mid(),
                        sdp_m_index: candidate.sdp_m_line_index(),
                        send_id: send.clone(),
                        friend_id: friend.clone(),
                        create_time: chrono::Utc::now().timestamp_millis(),
                    }));
                    // log::debug!("on ice candidate send message:candidate:{:?}, ", &msg_clone);
                    WebRTC::send_msg1(ws_clone.clone(), msg_clone)
                }
            })
                as Box<dyn FnMut(web_sys::RtcPeerConnectionIceEvent)>);

        let pc_clone = pc.clone();
        let callback = self.close_event.clone();
        let on_ice_connection_state_change = Closure::wrap(Box::new(move || {
            log::debug!(
                "on ice connection state change:{:?}",
                pc_clone.ice_connection_state()
            );
            match pc_clone.ice_connection_state() {
                RtcIceConnectionState::Failed
                | RtcIceConnectionState::Disconnected
                | RtcIceConnectionState::Closed => {
                    // 关闭视频流
                    callback.clone().emit(());
                }
                _ => {}
            }
            // handle ICE connection state change event
        }) as Box<dyn FnMut()>);

        let pc_clone = pc.clone();
        let on_ice_gathering_state_change = Closure::wrap(Box::new(move || {
            // handle ICE gathering state change event
            log::debug!(
                "on ice gathering state change:{:?}",
                &pc_clone.ice_gathering_state()
            );
        }) as Box<dyn FnMut()>);
        let pc_clone = pc.clone();
        let close_event = self.close_event.clone();
        let on_signaling_state_change = Closure::wrap(Box::new(move || {
            // handle signaling state change event
            log::debug!(
                "on signaling state change: {:?}",
                pc_clone.signaling_state()
            );
            if pc_clone.signaling_state() == RtcSignalingState::Closed {
                // 关闭视频流
                close_event.emit(());
            }
        }) as Box<dyn FnMut()>);

        let pc_clone = pc.clone();
        let on_negotiation_needed = Closure::once(Box::new(move || {
            // handle negotiation needed event
            let ws = Rc::clone(&ws);
            spawn_local(async move {
                let pc = pc_clone.clone();
                let offer = JsFuture::from(pc.create_offer()).await.unwrap();
                if pc.signaling_state() != RtcSignalingState::Stable {
                    log::debug!(
                        "on negotiation needed signaling state is : {:?}",
                        pc.signaling_state()
                    );
                    return;
                }
                JsFuture::from(pc.set_local_description(&RtcSessionDescriptionInit::from(offer)))
                    .await
                    .unwrap();
                let sdp = pc.local_description().unwrap().sdp();
                let msg = Msg::SingleCall(SingleCall::Offer(Offer {
                    sdp: sdp.into(),
                    send_id,
                    friend_id,
                    create_time: chrono::Utc::now().timestamp_millis(),
                }));
                // log::debug!("on negotiation needed send message: {:?}", &msg);
                WebRTC::send_msg1(ws.clone(), msg);
            });
        }) as Box<dyn FnOnce()>);
        let conn_event = self.conn_event.clone();
        let on_track = Closure::wrap(Box::new(move |event: web_sys::RtcTrackEvent| {
            // callback the conn event
            conn_event.emit(event);
        }) as Box<dyn FnMut(web_sys::RtcTrackEvent)>);

        // add to RtcPeerConnection
        pc.set_onicecandidate(Some(on_ice_candidate.as_ref().unchecked_ref()));
        pc.set_oniceconnectionstatechange(Some(
            on_ice_connection_state_change.as_ref().unchecked_ref(),
        ));
        pc.set_onicegatheringstatechange(Some(
            on_ice_gathering_state_change.as_ref().unchecked_ref(),
        ));
        pc.set_onsignalingstatechange(Some(on_signaling_state_change.as_ref().unchecked_ref()));
        pc.set_onnegotiationneeded(Some(on_negotiation_needed.as_ref().unchecked_ref()));
        pc.set_ontrack(Some(on_track.as_ref().unchecked_ref()));

        self.on_track = Some(on_track);
        self.on_ice_candidate = Some(on_ice_candidate);
        self.on_ice_connection_state_change = Some(on_ice_connection_state_change);
        self.on_ice_gathering_state_change = Some(on_ice_gathering_state_change);
        self.on_signaling_state_change = Some(on_signaling_state_change);
        self.on_negotiation = Some(on_negotiation_needed);
        self.pc = Some(pc);
        Ok(())
    }
}
