mod phone_call;
pub use phone_call::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};

use gloo::timers::callback::{Interval, Timeout};
use sandcat_sdk::{
    db,
    model::{
        message::{InviteInfo, InviteType, Message},
        ItemInfo,
    },
    state::{MobileState, SendCallState},
};
use std::fmt::Debug;
use wasm_bindgen::{JsCast, JsValue};
use web_rtc::WebRTC;
use web_sys::{
    HtmlAudioElement, HtmlVideoElement, MediaStream, MediaStreamTrack, RtcSdpType,
    RtcSessionDescriptionInit, RtcSignalingState,
};
use yew::prelude::*;
use yewdux::Dispatch;

pub struct PhoneCall {
    /// 显示视频通话
    show_video: bool,
    /// 显示语音通话
    show_audio: bool,
    /// 标记当前是否是被邀请方
    invited: bool,
    /// 显示通话邀请通知
    show_notify: bool,
    /// 音量是否静音
    volume_mute: bool,
    /// 麦克风是否静音
    microphone_mute: bool,
    /// 好友语音通话node ref
    friend_audio_node: NodeRef,
    /// 自己的视频通话node ref
    video_node: NodeRef,
    /// 好友视频通话node ref
    friend_video_node: NodeRef,
    /// 通话面板ref，用来面板拖动
    wrapper_node: NodeRef,
    /// 通话邀请信息
    invite_info: Option<InviteInfo>,
    /// 通话webrtc PeerConnection
    // pc: Option<RtcPeerConnection>,
    rtc: Option<WebRTC>,
    /// 音视频流
    stream: Option<MediaStream>,
    /// 通话的好友信息
    call_friend_info: Option<Box<dyn ItemInfo>>,
    /// 邀请计时器，到时间即为未接听
    call_timeout: Option<Timeout>,
    call_timer: Option<Interval>,
    /// 用来监听是否有通话消息
    /// 通话状态， 用来挂断、取消等等。。
    _call_state_dis: Dispatch<SendCallState>,
    /// 面板拖动记录x、y坐标
    pos_x: i32,
    pos_y: i32,
    /// 是否正在拖动面板
    is_dragging: bool,
    is_mobile: bool,
    is_zoom: bool,
}

impl Debug for PhoneCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        <NodeRef as Debug>::fmt(&self.friend_video_node, f)
    }
}

impl PhoneCall {
    fn new(ctx: &Context<Self>) -> Self {
        let call_state_dis =
            Dispatch::global().subscribe(ctx.link().callback(PhoneCallMsg::CallStateChange));
        let is_mobile = Dispatch::<MobileState>::global().get().is_mobile();
        Self {
            show_video: false,
            show_audio: false,
            friend_audio_node: NodeRef::default(),
            invited: false,
            video_node: NodeRef::default(),
            friend_video_node: NodeRef::default(),
            wrapper_node: NodeRef::default(),
            invite_info: None,
            rtc: None,
            stream: None,
            show_notify: false,
            call_friend_info: None,
            call_timeout: None,
            call_timer: None,
            volume_mute: false,
            microphone_mute: false,
            _call_state_dis: call_state_dis,
            pos_x: 0,
            pos_y: 0,
            is_dragging: false,
            is_mobile,
            is_zoom: false,
        }
    }

    fn create_pc(&mut self, ctx: &Context<Self>, sdp: &str) -> Result<(), JsValue> {
        let close_event = ctx.link().callback(|_| PhoneCallMsg::DisConnCall);
        let conn_event = ctx.link().callback(|_| PhoneCallMsg::DisConnCall);
        let invite_info = self.invite_info.as_ref().unwrap();
        let mut friend_id = invite_info.friend_id.clone();
        if self.invited {
            friend_id = invite_info.send_id.clone();
        }

        // let pc = web_rtc::WebRTC::create_pc(
        //     ctx.props().ws.clone(),
        //     ctx.props().user_id.clone(),
        //     friend_id,
        //     callback,
        //     invite_info.invite_type.clone(),
        //     self.friend_video_node.clone(),
        //     self.friend_audio_node.clone(),
        // )?;

        let mut rtc = web_rtc::WebRTC::new(close_event, conn_event);
        rtc.create_pc(
            ctx.props().ws.clone(),
            ctx.props().user_id.clone(),
            friend_id,
            invite_info.invite_type.clone(),
            self.friend_video_node.clone(),
            self.friend_audio_node.clone(),
        )?;
        let pc = rtc.pc();
        if self.invited {
            let mut description = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
            description.sdp(sdp);
            if pc.signaling_state() == RtcSignalingState::Stable {
                let future = JsFuture::from(pc.set_remote_description(&description));
                spawn_local(async move {
                    match future.await {
                        Ok(_) => {
                            log::debug!("set remote desc success in rtc signal state stable")
                        }
                        Err(err) => {
                            log::error!(
                                "set remote desc failed in rtc signal state stable: {:?}",
                                err
                            )
                        }
                    }
                });
            } else {
                let future = JsFuture::from(pc.set_remote_description(&description));
                spawn_local(async move {
                    match future.await {
                        Ok(_) => {
                            log::debug!("set remote desc success in rtc signal state not stable")
                        }
                        Err(err) => {
                            log::error!(
                                "set remote desc failed in rtc signal state not stable: {:?}",
                                err
                            )
                        }
                    }
                });
                // 生成并设置answer为本地描述
                let future = JsFuture::from(
                    pc.set_local_description(&RtcSessionDescriptionInit::new(RtcSdpType::Rollback)),
                );
                spawn_local(async move {
                    match future.await {
                        Ok(_) => {
                            log::debug!("set remote desc success in rtc signal state not stable")
                        }
                        Err(err) => {
                            log::error!(
                                "set remote desc failed in rtc signal state not stable: {:?}",
                                err
                            )
                        }
                    }
                });
                return Ok(());
            }
        } else {
            let stream = self.stream.as_ref().unwrap();
            for track in stream.get_tracks() {
                pc.add_track_0(&track.into(), stream);
            }
        }

        // self.pc = Some(pc);
        self.rtc = Some(rtc);
        Ok(())
    }

    fn mute_audio(&mut self, mute: bool) {
        if let Some(stream) = self.stream.as_ref() {
            let tracks = stream.get_audio_tracks();
            for track in tracks {
                let track = track.unchecked_into::<MediaStreamTrack>();
                track.set_enabled(!mute);
            }
        }
    }

    fn finish_call(&mut self) {
        if let Some(ref mut rtc) = self.rtc {
            log::debug!("hang up video clear pc");
            rtc.close();
        }

        if let Some(stream) = &self.stream {
            for track in stream.get_tracks() {
                if let Ok(track) = track.dyn_into::<MediaStreamTrack>() {
                    track.stop();
                }
            }
        }

        if let Some(invite_info) = self.invite_info.as_ref() {
            match invite_info.invite_type {
                InviteType::Video => {
                    if let Some(video) = self.video_node.cast::<HtmlVideoElement>() {
                        log::debug!("hang up video clear stream");
                        video.set_src_object(None);
                    }
                    if let Some(friend_video_node) =
                        self.friend_video_node.cast::<HtmlVideoElement>()
                    {
                        log::debug!("hang up video clear stream2");
                        friend_video_node.set_src_object(None);
                    }
                    self.show_video = false;
                }
                InviteType::Audio => {
                    if let Some(audio) = self.friend_audio_node.cast::<HtmlAudioElement>() {
                        log::debug!("hang up audio clear stream");
                        audio.set_src_object(None);
                    }
                    self.show_audio = false;
                }
            }
        }

        self.rtc = None;
        self.stream = None;
        self.invite_info = None;
        self.invited = false;
        self.call_timeout = None;
        self.call_timer = None;
        self.call_friend_info = None;
        self.volume_mute = false;
        self.microphone_mute = false;
        self.is_zoom = false;
    }

    fn save_call_msg(&self, mut msg: Message) {
        spawn_local(async move {
            db::db_ins()
                .messages
                .add_message(&mut msg)
                .await
                .map_err(|err| log::error!("消息入库失败:{:?}", err))
                .unwrap();
        });
    }
}
