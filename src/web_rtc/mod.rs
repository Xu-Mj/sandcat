use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    HtmlAudioElement, HtmlVideoElement, RtcConfiguration, RtcIceConnectionState, RtcIceServer,
    RtcPeerConnection, RtcSessionDescriptionInit, RtcSignalingState,
};
use yew::platform::spawn_local;
use yew::{AttrValue, Callback, NodeRef};

use crate::model::message::{Candidate, InviteType, Msg, Offer, SingleCall};
use crate::ws::WebSocketManager;

pub struct WebRTC;

impl WebRTC {
    pub fn send_msg1(ws: Rc<RefCell<WebSocketManager>>, msg: &Msg) {
        // 发送已收到消息给服务器
        match ws
            .borrow()
            .send_message(&serde_json::to_string(&msg).unwrap())
        {
            Ok(_) => { /*log::info!("发送成功:{:?}", &msg)*/ }
            Err(e) => {
                log::error!("发送失败: {:?}", e)
            }
        };
    }

    pub fn create_pc(
        ws: Rc<RefCell<WebSocketManager>>,
        send_id: AttrValue,
        friend_id: AttrValue,
        close_event: Callback<AttrValue>,
        invite_type: InviteType,
        friend_video_node: NodeRef,
        friend_audio_node: NodeRef,
    ) -> Result<RtcPeerConnection, JsValue> {
        // 创建一个新的RtcIceServer
        let mut ice_server = RtcIceServer::new();
        ice_server.url("stun:stun.l.google.com:19302");

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
                        create_time: chrono::Local::now().timestamp_millis(),
                    }));
                    // log::debug!("on ice candidate send message:candidate:{:?}, ", &msg_clone);
                    WebRTC::send_msg1(ws_clone.clone(), &msg_clone)
                }
            })
                as Box<dyn FnMut(web_sys::RtcPeerConnectionIceEvent)>);

        let pc_clone = pc.clone();
        let callback = close_event.clone();
        let friend = friend_id.clone();
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
                    callback.clone().emit(friend.clone());
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
        let friend = friend_id.clone();
        let on_signaling_state_change = Closure::wrap(Box::new(move || {
            // handle signaling state change event
            log::debug!(
                "on signaling state change: {:?}",
                pc_clone.signaling_state()
            );
            if pc_clone.signaling_state() == RtcSignalingState::Closed {
                // 关闭视频流
                close_event.emit(friend.clone());
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
                let msg = &Msg::SingleCall(SingleCall::Offer(Offer {
                    sdp: sdp.into(),
                    send_id,
                    friend_id,
                    create_time: chrono::Local::now().timestamp_millis(),
                }));
                // log::debug!("on negotiation needed send message: {:?}", &msg);
                WebRTC::send_msg1(ws.clone(), msg);
            });
        }) as Box<dyn FnOnce()>);
        let friend_node = friend_video_node.clone();
        let friend_audio_node = friend_audio_node.clone();
        let on_track = Closure::wrap(Box::new(move |event: web_sys::RtcTrackEvent| {
            // handle track event
            match invite_type {
                InviteType::Video => {
                    let friend_video: HtmlVideoElement = friend_node.cast().unwrap();
                    friend_video.set_src_object(Some(&event.streams().get(0).into()));
                    let _ = friend_video.play().expect("friend video play error");
                }
                InviteType::Audio => {
                    let friend_audio: HtmlAudioElement = friend_audio_node.cast().unwrap();
                    friend_audio.set_src_object(Some(&event.streams().get(0).into()));
                    let _ = friend_audio.play().expect("friend video play error");
                }
            }
        }) as Box<dyn FnMut(web_sys::RtcTrackEvent)>);

        // 添加到 RtcPeerConnection
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

        on_ice_candidate.forget();
        on_ice_connection_state_change.forget();
        on_ice_gathering_state_change.forget();
        on_signaling_state_change.forget();
        on_negotiation_needed.forget();
        on_track.forget();
        Ok(pc)
    }
}
