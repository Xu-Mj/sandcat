use std::cell::RefCell;
use std::rc::Rc;

use gloo::timers::callback::{Interval, Timeout};
use log::{debug, error};
use nanoid::nanoid;
use sandcat_sdk::error::Error;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    HtmlAudioElement, HtmlDivElement, HtmlVideoElement, MediaStream, MouseEvent,
    RtcIceCandidateInit, RtcSdpType, RtcSessionDescription, RtcSessionDescriptionInit,
};
use yew::platform::spawn_local;
use yew::{html, AttrValue, Callback, Component, Context, Html, Properties, TouchEvent};

use i18n::{en_us, zh_cn, LanguageType};
use icons::{
    AnswerPhoneIcon, AudioZoomInIcon, AudioZoomOutIcon, HangUpLoadingIcon, HangupInNotifyIcon,
    MicrophoneIcon, MicrophoneMuteIcon, VideoRecordIcon, VolumeIcon, VolumeMuteIcon,
};
use sandcat_sdk::db;
use sandcat_sdk::model::message::{
    Agree, Hangup, InviteAnswerMsg, InviteCancelMsg, InviteInfo, InviteMsg, InviteNotAnswerMsg,
    InviteType, Message, Msg, SingleCall,
};
use sandcat_sdk::model::notification::Notification;
use sandcat_sdk::model::ContentType;
use sandcat_sdk::model::ItemInfo;
use sandcat_sdk::state::{I18nState, SendCallState};
use utils::tr;
use ws::WebSocketManager;

use crate::call::ConnectionState;
use crate::constant::{
    CALL_BUSY, CONNECTING, CONN_ERROR, INCOMING_CALL, OTHER_ERROR, STREAM_ERROR, UNKNOW_ERROR,
    WAITING,
};
use crate::get_platform;

use super::PhoneCall;

#[derive(Properties, Clone, PartialEq, Debug)]
pub struct PhoneCallProps {
    pub ws: Rc<RefCell<WebSocketManager>>,
    pub user_id: AttrValue,
    pub send_msg: Callback<SingleCall>,
    pub msg: SingleCall,
    pub lang: LanguageType,
}

pub enum PhoneCallMsg {
    // send single call invitation
    SendCallInvite(InviteMsg),
    // cancel call
    SendInviteCancel,
    // agree call
    ResponseCall,
    // 挂断视频
    HangUpCall,
    // 同意视频通话请求
    AgreeCall,
    // 类似与VideoOnReady（当本地视频流准备好之后完全接通视频电话）
    ConnectedCall(MediaStream),
    DenyCall,
    DisConnCall,
    ShowVideoWindow(MediaStream, Box<dyn ItemInfo>),
    ShowAudioWindow(MediaStream, Box<dyn ItemInfo>),
    CallTimeout,
    // callback from pc on_track
    OnConnect(web_sys::RtcTrackEvent),
    TickCallDuration,
    // 显示顶部消息通知
    ShowCallNotify(Box<dyn ItemInfo>),
    SwitchVolume,
    SwitchMicrophoneMute,
    SendMessage(SingleCall),
    CallStateChange(Rc<SendCallState>),
    I18nStateChange(Rc<I18nState>),
    Close,
    Error(String),
    OnMouseDown(MouseEvent),
    OnMouseMove(MouseEvent),
    OnMouseUp,
    SwitchZoom,
    OnTouchStart(TouchEvent),
    OnTouchMove(TouchEvent),
    OnTouchEnd,
}

const TIMEOUT: u32 = 120;

impl Component for PhoneCall {
    type Message = PhoneCallMsg;
    type Properties = PhoneCallProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self::new(ctx)
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        // don't update the ui when the message is empty or the msg is the same
        if ctx.props().msg == SingleCall::default() || ctx.props().msg == _old_props.msg {
            return false;
        }

        let message = ctx.props().msg.clone();
        match message {
            SingleCall::Invite(msg) => {
                // 判断是否占线
                if self.invite_info.is_some() {
                    // todo回复占线
                    return false;
                }

                // display call notify
                self.show_notify = true;
                self.invited = true;
                let friend_id = msg.send_id.clone();
                self.invite_info = Some(InviteInfo {
                    send_id: msg.send_id,
                    friend_id: msg.friend_id,
                    invite_type: msg.invite_type.clone(),
                    ..Default::default()
                });
                debug!("invite_info: {:?}", self.invite_info);
                ctx.link().send_future(async move {
                    // 查询好友数据
                    let friend = db::db_ins()
                        .friends
                        .get(friend_id.as_str())
                        .await
                        .unwrap()
                        .unwrap_or_default();
                    debug!("收到邀请: {:?}", friend);
                    PhoneCallMsg::ShowCallNotify(Box::new(friend))
                });
            }
            SingleCall::InviteCancel(mut msg) => {
                debug!("对方取消通话");
                // 判断是否是当前用户
                if let Some(info) = self.invite_info.as_ref() {
                    if info.send_id == msg.send_id {
                        debug!("正在关闭通知");

                        self.finish_call();
                        // 数据入库
                        let friend_id = msg.send_id.clone();
                        msg.send_id = msg.friend_id.clone();
                        msg.friend_id = friend_id;
                        self.show_notify = false;
                        self.call_friend_info = None;
                        self.save_call_msg(msg.into());
                        debug!("已经关闭通知");
                        return true;
                    }
                }
            }
            SingleCall::InviteAnswer(mut msg) => {
                debug!("single invite answer");
                if msg.agree {
                    // 对方同意了通话请求，正在建立连接，为了简化代码邀请方和被邀请方的创建pc方法合并到一起了
                    if let Err(e) = self.create_pc(ctx, "") {
                        Notification::error(Error::js_err(e)).notify();
                        return false;
                    }
                } else {
                    // 拒绝通话请求，数据入库
                    debug!("对方拒绝了请求");
                    let friend_id = msg.send_id.clone();
                    msg.send_id = msg.friend_id.clone();
                    msg.friend_id = friend_id;
                    match msg.invite_type {
                        InviteType::Video => {
                            self.show_video = false;
                        }
                        InviteType::Audio => {
                            self.show_audio = false;
                        }
                    }
                    self.save_call_msg(msg.into());
                    self.finish_call();
                    return true;
                }
            }
            SingleCall::Offer(msg) => {
                // 建立通话连接，收到offer，设置sdp
                if self.rtc.is_some() {
                    log::warn!("收到邀请，但是占线: {:?}", &msg);
                    return false;
                }

                if let Err(e) = self.create_pc(ctx, &msg.sdp) {
                    error!("创建连接失败:{:?}", e);
                    return false;
                }

                match self.invite_info.as_ref().unwrap().invite_type {
                    InviteType::Video => {
                        self.show_video = true;
                    }
                    InviteType::Audio => {
                        self.show_audio = true;
                    }
                }

                ctx.link().send_message(PhoneCallMsg::ResponseCall)
            }
            SingleCall::Agree(msg) => {
                // 同意通话请求并建立连接，设置sdp
                // 判断是否是我们发出去的邀请回复
                if let Some(info) = &self.invite_info {
                    if info.friend_id == msg.send_id && self.rtc.is_some() {
                        // 接通
                        debug!("请求被对方同意");
                        // todo需要在webrtc状态为Connected下进行回调修改
                        // self.invite_info.as_mut().unwrap().start_time =
                        //     chrono::Utc::now().timestamp_millis();
                        // self.invite_info.as_mut().unwrap().connected = true;
                        let mut description = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
                        description.sdp(&msg.sdp.unwrap());
                        let future = JsFuture::from(
                            self.rtc
                                .as_ref()
                                .unwrap()
                                .pc()
                                .set_remote_description(&description),
                        );
                        spawn_local(async {
                            if let Err(err) = future.await {
                                error!("remote desc set failed: {:?}", err);
                            }
                        });
                        return true;
                    }
                }
            }
            SingleCall::NewIceCandidate(msg) => {
                let mut candidate = RtcIceCandidateInit::new(&msg.candidate);
                if let Some(index) = msg.sdp_m_index {
                    candidate.sdp_m_line_index(Some(index));
                }
                if let Some(mid) = msg.sdp_mid {
                    candidate.sdp_mid(Some(&mid));
                }
                let future = JsFuture::from(
                    self.rtc
                        .as_ref()
                        .unwrap()
                        .pc()
                        .add_ice_candidate_with_opt_rtc_ice_candidate_init(Some(&candidate)),
                );
                spawn_local(async {
                    if let Err(err) = future.await {
                        // todo need to interrupt the call
                        error!("set ice candidate failed: {:?}", err);
                    }
                })
            }
            SingleCall::NotAnswer(mut msg) => {
                // 判断是否是当前用户
                if let Some(info) = self.invite_info.as_ref() {
                    if info.send_id == msg.send_id {
                        // 数据入库
                        let friend_id = msg.send_id.clone();
                        msg.send_id = msg.friend_id.clone();
                        msg.friend_id = friend_id;
                        self.invite_info = None;
                        self.show_notify = false;
                        self.call_friend_info = None;
                        self.save_call_msg(msg.into());
                        return true;
                    }
                }
            }
            SingleCall::HangUp(mut msg) => {
                // 判断是否是当前连接
                if let Some(info) = self.invite_info.as_ref() {
                    if info.send_id == msg.friend_id || info.send_id == msg.send_id {
                        let friend_id = msg.send_id.clone();
                        msg.send_id = msg.friend_id.clone();
                        msg.friend_id = friend_id;
                        let info = self.invite_info.as_ref().unwrap();
                        let create_time = chrono::Utc::now().timestamp_millis();
                        let sustain = create_time - info.start_time;
                        msg.sustain = sustain;
                        self.save_call_msg(msg.into());
                        self.finish_call();
                        return true;
                    }
                }
            }
        }
        false
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PhoneCallMsg::I18nStateChange(state) => {
                let res = match state.lang {
                    LanguageType::ZhCN => zh_cn::CALL_COM,
                    LanguageType::EnUS => en_us::CALL_COM,
                };
                self.i18n = utils::create_bundle(res);
                true
            }
            PhoneCallMsg::SendCallInvite(msg) => {
                debug!("send call invite");
                // 判断是否正在通话中
                if self.invite_info.is_some() {
                    Notification::warn(tr!(self.i18n, CALL_BUSY)).notify();
                    return false;
                }
                self.invite_info = Some(InviteInfo {
                    send_id: ctx.props().user_id.clone(),
                    friend_id: msg.friend_id.clone(),
                    invite_type: msg.invite_type.clone(),
                    start_time: chrono::Utc::now().timestamp_millis(),
                    end_time: 0,
                    connected: false,
                });
                let call_type = msg.invite_type.clone();
                match call_type {
                    InviteType::Video => {
                        self.show_video = true;
                    }
                    InviteType::Audio => {
                        self.show_audio = true;
                    }
                }
                let friend_id = msg.friend_id.clone();

                // send invite message; no necessary to notify other components
                let ws = ctx.props().ws.clone();
                ctx.link().send_future(async move {
                    let friend = db::db_ins()
                        .friends
                        .get(friend_id.as_str())
                        .await
                        .unwrap()
                        .unwrap_or_default();

                    match call_type {
                        InviteType::Video => match utils::get_video_stream().await {
                            Ok(stream) => {
                                if let Err(e) = ws
                                    .borrow()
                                    .send_message(Msg::SingleCall(SingleCall::Invite(msg)))
                                {
                                    error!("send message error: {:?}", e);
                                    return PhoneCallMsg::Close;
                                }
                                PhoneCallMsg::ShowVideoWindow(stream, Box::new(friend))
                            }
                            Err(e) => {
                                let content = if let Some(dom_exception) =
                                    e.dyn_ref::<web_sys::DomException>()
                                {
                                    log::warn!("dom exception: {}", dom_exception.name());
                                    if dom_exception.name() == "NotFoundError" {
                                        log::warn!("没有检测到音视频设备");
                                        // "没有检测到音视频设备"
                                        STREAM_ERROR
                                    } else {
                                        // "其他错误"
                                        OTHER_ERROR
                                    }
                                } else {
                                    error!("未知错误获取音频流: {:?}", e);
                                    UNKNOW_ERROR
                                    // "其他错误"
                                };
                                PhoneCallMsg::Error(content.to_string())
                            }
                        },
                        InviteType::Audio => {
                            match utils::get_audio_stream().await {
                                Ok(stream) => {
                                    // send invite message
                                    if let Err(e) = ws
                                        .borrow()
                                        .send_message(Msg::SingleCall(SingleCall::Invite(msg)))
                                    {
                                        error!("send invite msg error: {:?}", e);
                                    } else {
                                        debug!("send invite msg success");
                                    }
                                    PhoneCallMsg::ShowAudioWindow(stream, Box::new(friend))
                                }
                                Err(e) => {
                                    let content = if let Some(dom_exception) =
                                        e.dyn_ref::<web_sys::DomException>()
                                    {
                                        log::warn!("dom exception: {}", dom_exception.name());
                                        if dom_exception.name() == "NotFoundError" {
                                            log::warn!("没有检测到音频设备");
                                            // "没有检测到音视频设备"
                                            STREAM_ERROR
                                        } else {
                                            OTHER_ERROR
                                        }
                                    } else {
                                        error!("未知错误获取音频流: {:?}", e);
                                        UNKNOW_ERROR
                                    };
                                    PhoneCallMsg::Error(content.to_string())
                                }
                            }
                        }
                    }
                });
                // true
                false
            }
            PhoneCallMsg::SendInviteCancel => {
                debug!("SendInviteCancele");
                let local_id = AttrValue::from(nanoid!());
                let info = self.invite_info.as_ref().unwrap();
                let friend_id = info.friend_id.clone();
                let create_time = chrono::Utc::now().timestamp_millis();
                let send_id = ctx.props().user_id.clone();

                // save data to db
                let content_type = match info.invite_type {
                    InviteType::Video => ContentType::VideoCall,
                    InviteType::Audio => ContentType::AudioCall,
                };
                let invite_type = info.invite_type.clone();
                ctx.link().send_future(async move {
                    let _ = db::db_ins()
                        .messages
                        .add_message(&Message {
                            local_id: local_id.clone(),
                            send_id: send_id.clone(),
                            friend_id: friend_id.clone(),
                            content_type,
                            content: AttrValue::from("cancel"),
                            create_time,
                            is_self: true,
                            ..Default::default()
                        })
                        .await
                        .map_err(|err| error!("消息入库失败:{:?}", err));

                    PhoneCallMsg::SendMessage(SingleCall::InviteCancel(InviteCancelMsg {
                        local_id,
                        send_id,
                        friend_id,
                        create_time,
                        invite_type,
                        is_self: true,
                        ..Default::default()
                    }))
                });
                self.finish_call();
                true
            }
            PhoneCallMsg::ResponseCall => {
                debug!("ResponseCall");
                self.invited = true;

                // get stream
                let invite_type = self.invite_info.as_ref().unwrap().invite_type.clone();
                ctx.link().send_future(async move {
                    match invite_type {
                        InviteType::Video => match utils::get_video_stream().await {
                            Ok(stream) => PhoneCallMsg::ConnectedCall(stream),
                            Err(e) => {
                                Notification::error(Error::js_err(e)).notify();
                                PhoneCallMsg::Close
                            }
                        },
                        InviteType::Audio => match utils::get_audio_stream().await {
                            Ok(stream) => PhoneCallMsg::ConnectedCall(stream),
                            Err(e) => {
                                Notification::error(Error::js_err(e)).notify();
                                PhoneCallMsg::Close
                            }
                        },
                    }
                });

                false
            }
            PhoneCallMsg::HangUpCall => {
                debug!("HangUpCall");
                let info = self.invite_info.as_ref().unwrap();
                let create_time = chrono::Utc::now().timestamp_millis();
                let sustain = create_time - info.start_time;
                let local_id = AttrValue::from(nanoid!());
                let friend_id = if self.invited {
                    info.send_id.clone()
                } else {
                    info.friend_id.clone()
                };
                let send_id = ctx.props().user_id.clone();
                let content_type = match info.invite_type {
                    InviteType::Video => ContentType::VideoCall,
                    InviteType::Audio => ContentType::AudioCall,
                };
                let invite_type = info.invite_type.clone();

                // save message to db
                ctx.link().send_future(async move {
                    db::db_ins()
                        .messages
                        .add_message(&Message {
                            local_id: local_id.clone(),
                            send_id: send_id.clone(),
                            friend_id: friend_id.clone(),
                            content_type,
                            content: format!("duration||{}", utils::format_milliseconds(sustain))
                                .into(),
                            create_time,
                            is_read: 1,
                            is_self: true,
                            ..Default::default()
                        })
                        .await
                        .map_err(|err| error!("消息入库失败:{:?}", err))
                        .unwrap();

                    PhoneCallMsg::SendMessage(SingleCall::HangUp(Hangup {
                        local_id,
                        server_id: AttrValue::default(),
                        send_id,
                        friend_id,
                        create_time,
                        invite_type,
                        sustain,
                        is_self: true,
                        ..Default::default()
                    }))
                });

                self.finish_call();
                true
            }
            PhoneCallMsg::AgreeCall => {
                debug!("AgreeCall");
                // 同意视频通话
                let info = self.invite_info.as_ref().unwrap();
                let msg = SingleCall::InviteAnswer(InviteAnswerMsg {
                    local_id: nanoid!().into(),
                    server_id: AttrValue::default(),
                    send_id: ctx.props().user_id.clone(),
                    friend_id: self.invite_info.as_ref().unwrap().send_id.clone(),
                    create_time: chrono::Utc::now().timestamp_millis(),
                    agree: true,
                    is_self: true,
                    invite_type: info.invite_type.clone(),
                    ..Default::default()
                });
                if let Err(e) = ctx.props().ws.borrow().send_message(Msg::SingleCall(msg)) {
                    // todo notify user we failed to send message
                    error!("send message error: {:?}", e);
                    return false;
                }
                self.show_notify = false;
                match info.invite_type {
                    InviteType::Video => {
                        self.show_video = true;
                    }
                    InviteType::Audio => {
                        self.show_audio = true;
                    }
                }
                // change the connection state
                self.conn_state = ConnectionState::Connecting;
                true
            }
            PhoneCallMsg::ConnectedCall(stream) => {
                debug!("ConnectedCall");
                // set stream to peer connection
                let pc = self.rtc.as_ref().unwrap().pc().clone();
                let invite_info = self.invite_info.as_ref().unwrap();
                match invite_info.invite_type {
                    InviteType::Video => {
                        let video: HtmlVideoElement = self.video_node.cast().unwrap();
                        video.set_src_object(Some(&stream));
                        let _ = video.play().unwrap();
                        video.set_muted(true);
                    }
                    InviteType::Audio => {}
                }

                for track in stream.get_tracks() {
                    pc.add_track_0(&track.into(), &stream);
                }

                self.stream = Some(stream);
                let js_future = JsFuture::from(pc.create_answer());
                let ws = Rc::clone(&ctx.props().ws.clone());
                // let pc = pc.clone();
                let send_id = invite_info.friend_id.clone();
                let friend_id = invite_info.send_id.clone();
                let platform = get_platform(self.is_mobile);
                spawn_local(async move {
                    let js_value = js_future.await.unwrap();
                    let rtc_desc = RtcSessionDescription::from(js_value);
                    let mut desc = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
                    desc.sdp(&rtc_desc.sdp());
                    let _ = pc.set_local_description(&desc);
                    debug!("set_local_description: {:?}", rtc_desc.sdp());

                    if let Err(e) =
                        ws.borrow()
                            .send_message(Msg::SingleCall(SingleCall::Agree(Agree {
                                sdp: Some(rtc_desc.sdp()),
                                send_id,
                                friend_id,
                                create_time: 0,
                                platform,
                            })))
                    {
                        error!("send message error: {:?}", e);
                    }
                });
                true
            }
            PhoneCallMsg::DenyCall => {
                debug!("DenyCall");
                let info = self.invite_info.as_ref().unwrap();
                let local_id = AttrValue::from(nanoid!());
                let send_id = ctx.props().user_id.clone();
                let friend_id = info.send_id.clone();
                let content_type = match info.invite_type {
                    InviteType::Video => ContentType::VideoCall,
                    InviteType::Audio => ContentType::AudioCall,
                };
                let create_time = chrono::Utc::now().timestamp_millis();
                let invite_type = info.invite_type.clone();
                self.show_notify = false;
                // self.finish_call();
                self.invite_info = None;
                self.invited = false;
                ctx.link().send_future(async move {
                    let _ = db::db_ins()
                        .messages
                        .add_message(&Message {
                            local_id: local_id.clone(),
                            server_id: AttrValue::default(),
                            send_id: send_id.clone(),
                            friend_id: friend_id.clone(),
                            content_type,
                            content: AttrValue::from("deny"),
                            create_time,
                            is_read: 1,
                            is_self: true,
                            ..Default::default()
                        })
                        .await;

                    PhoneCallMsg::SendMessage(SingleCall::InviteAnswer(InviteAnswerMsg {
                        local_id,
                        server_id: AttrValue::default(),
                        send_id,
                        friend_id,
                        create_time,
                        invite_type,
                        is_self: true,
                        ..Default::default()
                    }))
                });
                true
            }
            PhoneCallMsg::DisConnCall => {
                // 判断视频窗口是否还存在
                if self.show_video {
                    // 窗口还在说明连接被中断，因为不在了的情况是对端主动发起的挂断请求
                    // 是正常的主动处理
                    self.finish_call();
                    // TODO 记录中断消息
                }
                true
            }
            PhoneCallMsg::ShowVideoWindow(stream, friend) => {
                let video: HtmlVideoElement = self.video_node.cast().unwrap();
                self.call_friend_info = Some(friend);
                video.set_src_object(Some(&stream));
                let _ = video.play().unwrap();
                video.set_muted(true);
                self.stream = Some(stream);
                let ctx = ctx.link().clone();
                self.call_timeout = Some(Timeout::new(TIMEOUT * 1000, move || {
                    ctx.send_message(PhoneCallMsg::CallTimeout);
                }));
                false
            }
            PhoneCallMsg::CallTimeout => {
                debug!("CallTimeout");
                if self.show_video || self.show_audio {
                    let info = self.invite_info.as_ref().unwrap();
                    if info.connected {
                        return false;
                    }
                    let local_id = AttrValue::from(nanoid!());
                    let friend_id = info.friend_id.clone();
                    let create_time = chrono::Utc::now().timestamp_millis();
                    let send_id = ctx.props().user_id.clone();
                    // 数据入库
                    let content_type = match info.invite_type {
                        InviteType::Video => ContentType::VideoCall,
                        InviteType::Audio => ContentType::AudioCall,
                    };
                    let invite_type = info.invite_type.clone();
                    ctx.link().send_future(async move {
                        let _ = db::db_ins()
                            .messages
                            .add_message(&Message {
                                local_id: local_id.clone(),
                                send_id: send_id.clone(),
                                friend_id: friend_id.clone(),
                                content_type,
                                content: AttrValue::from("not_answer"),
                                create_time,
                                is_self: true,
                                ..Default::default()
                            })
                            .await
                            .map_err(|err| error!("消息入库失败:{:?}", err));

                        PhoneCallMsg::SendMessage(SingleCall::NotAnswer(InviteNotAnswerMsg {
                            local_id,
                            send_id,
                            friend_id,
                            create_time,
                            invite_type,
                            is_self: true,
                            ..Default::default()
                        }))
                    });
                    self.finish_call();

                    return true;
                }
                false
            }
            PhoneCallMsg::SwitchVolume => {
                debug!("SwitchVolume");
                self.volume_mute = !self.volume_mute;
                match self.invite_info.as_ref().unwrap().invite_type {
                    InviteType::Video => {
                        let node: HtmlVideoElement = self.friend_video_node.cast().unwrap();
                        node.set_muted(self.volume_mute);
                    }
                    InviteType::Audio => {
                        let node: HtmlAudioElement = self.friend_audio_node.cast().unwrap();
                        node.set_muted(self.volume_mute);
                    }
                }
                true
            }
            PhoneCallMsg::SwitchMicrophoneMute => {
                debug!("SwitchMicrophoneMute");
                self.microphone_mute = !self.microphone_mute;
                self.mute_audio(self.microphone_mute);
                true
            }
            PhoneCallMsg::ShowAudioWindow(stream, friend) => {
                debug!("ShowAudioWindow");
                self.stream = Some(stream);
                self.call_friend_info = Some(friend);
                let ctx = ctx.link().clone();
                self.call_timeout = Some(Timeout::new(TIMEOUT * 1000, move || {
                    ctx.send_message(PhoneCallMsg::CallTimeout);
                }));
                true
            }

            PhoneCallMsg::ShowCallNotify(item) => {
                debug!("show call notify: {:?}", item.id());
                self.call_friend_info = Some(item);
                self.show_notify = true;
                self.invited = true;
                true
            }
            PhoneCallMsg::SendMessage(msg) => {
                ctx.props().send_msg.emit(msg);
                false
            }
            PhoneCallMsg::CallStateChange(state) => {
                debug!("CallStateChange");
                // 判断是否是空的状态
                if state.msg.local_id.is_empty() {
                    return false;
                }
                ctx.link()
                    .send_message(PhoneCallMsg::SendCallInvite(state.msg.clone()));
                // self.call_state = state;
                true
            }
            PhoneCallMsg::Close => {
                self.finish_call();
                true
            }
            PhoneCallMsg::OnMouseDown(event) => {
                event.stop_propagation();
                event.prevent_default();
                self.pos_x = event.client_x();
                self.pos_y = event.client_y();
                self.is_dragging = true;
                false
            }
            PhoneCallMsg::OnMouseMove(event) => {
                if !self.is_dragging {
                    return false;
                }
                let x = self.pos_x - event.client_x();
                let y = self.pos_y - event.client_y();
                self.pos_x = event.client_x();
                self.pos_y = event.client_y();
                // set new location for window
                if self.invite_info.is_some() {
                    if let Some(div) = self.wrapper_node.cast::<HtmlDivElement>() {
                        div.style()
                            .set_property("top", &format!("{}px", div.offset_top() - y))
                            .map_err(|e| error!("set top error: {:?}", e))
                            .expect("set top position panic");
                        div.style()
                            .set_property("left", &format!("{}px", div.offset_left() - x))
                            .expect("set left position panic");
                    }
                }
                true
            }
            PhoneCallMsg::OnMouseUp => {
                if !self.is_dragging {
                    return false;
                }
                self.pos_x = 0;
                self.pos_y = 0;
                self.is_dragging = false;
                false
            }
            PhoneCallMsg::SwitchZoom => {
                self.is_zoom = !self.is_zoom;
                if !self.is_zoom && self.is_mobile {
                    // set container to top:0; left:0;
                    if let Some(div) = self.wrapper_node.cast::<HtmlDivElement>() {
                        // ignore error
                        let _ = div.style().set_property("top", "0");
                        let _ = div.style().set_property("left", "0");
                    }
                }
                true
            }
            PhoneCallMsg::OnTouchStart(event) => {
                event.stop_propagation();
                event.prevent_default();
                if let Some(event) = event.touches().get(0) {
                    self.pos_x = event.client_x();
                    self.pos_y = event.client_y();
                    self.is_dragging = true;
                }
                false
            }
            PhoneCallMsg::OnTouchMove(event) => {
                if !self.is_dragging {
                    return false;
                }
                if let Some(event) = event.touches().get(0) {
                    let x = self.pos_x - event.client_x();
                    let y = self.pos_y - event.client_y();
                    self.pos_x = event.client_x();
                    self.pos_y = event.client_y();
                    // set new location for window
                    if self.invite_info.is_some() {
                        if let Some(div) = self.wrapper_node.cast::<HtmlDivElement>() {
                            div.style()
                                .set_property("top", &format!("{}px", div.offset_top() - y))
                                .map_err(|e| error!("set top error: {:?}", e))
                                .expect("set top position panic");
                            div.style()
                                .set_property("left", &format!("{}px", div.offset_left() - x))
                                .expect("set left position panic");
                        }
                    }
                }
                true
            }
            PhoneCallMsg::OnTouchEnd => {
                if !self.is_dragging {
                    return false;
                }
                self.pos_x = 0;
                self.pos_y = 0;
                self.is_dragging = false;
                false
            }
            PhoneCallMsg::OnConnect(event) => {
                // truly connected
                self.invite_info.as_mut().unwrap().connected = true;
                self.invite_info.as_mut().unwrap().start_time =
                    chrono::Utc::now().timestamp_millis();
                self.conn_state = ConnectionState::Connected;
                let ctx = ctx.link().clone();
                self.call_timer = Some(Interval::new(1000, move || {
                    ctx.send_message(PhoneCallMsg::TickCallDuration);
                }));
                match self.invite_info.as_ref().unwrap().invite_type {
                    InviteType::Video => {
                        let friend_video: HtmlVideoElement = self.friend_video_node.cast().unwrap();
                        friend_video.set_src_object(Some(&event.streams().get(0).into()));
                        let _ = friend_video.play().expect("friend video play error");
                    }
                    InviteType::Audio => {
                        let friend_audio: HtmlAudioElement = self.friend_audio_node.cast().unwrap();
                        friend_audio.set_src_object(Some(&event.streams().get(0).into()));
                        let _ = friend_audio.play().expect("friend video play error");
                    }
                }
                false
            }
            PhoneCallMsg::TickCallDuration => {
                self.call_duration += 1;
                true
            }
            PhoneCallMsg::Error(err) => {
                self.finish_call();
                Notification::error(Error::internal_with_details(tr!(self.i18n, &err))).notify();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // todo error notification
        if self.conn_state == ConnectionState::Error {
            return html! {
                    <div class="error">
                    </div>
            };
        }
        let mut class = if self.is_mobile {
            "phone-call-size-mobile"
        } else {
            "box-shadow phone-call-size"
        };
        let mut video_or_audio_notify = html!();
        if self.show_notify {
            let info = self.call_friend_info.as_ref().unwrap();
            let answer_icon;
            let answer_click;
            match self.invite_info.as_ref().unwrap().invite_type {
                InviteType::Video => {
                    answer_icon = html!(<VideoRecordIcon />);
                    answer_click = ctx.link().callback(|_| PhoneCallMsg::AgreeCall);
                }
                InviteType::Audio => {
                    answer_icon = html!(<AnswerPhoneIcon/>);
                    answer_click = ctx.link().callback(|_| PhoneCallMsg::AgreeCall);
                }
            };
            video_or_audio_notify = html! {
                <div class="video-or-audio-notify box-shadow" >
                    // 头像。昵称。挂断。接听
                    <img alt="avatar" src={utils::get_avatar_url(&info.avatar())}/>
                    <span class="video-or-audio-notify-text" >
                        {format!("{} {}", info.name(), tr!(self.i18n, INCOMING_CALL))}
                    </span>
                    <div class="video-audio-notify-operate" >
                        <span onclick={ctx.link().callback(|_| PhoneCallMsg::DenyCall)}>
                            <HangupInNotifyIcon />
                        </span>
                        <span onclick={answer_click}>
                            {answer_icon}
                        </span>
                    </div>
                </div>
            }
        }

        let mut video = html!();
        let mut audio = html!();

        let (hangup_icon, duration) = match self.conn_state {
            ConnectionState::Waiting => (html!(<HangupInNotifyIcon/>), tr!(self.i18n, WAITING)),
            ConnectionState::Connecting => {
                (html!(<HangUpLoadingIcon/>), tr!(self.i18n, CONNECTING))
            }
            ConnectionState::Connected => (html!(<HangupInNotifyIcon/>), self.format_duration()),
            ConnectionState::Error => (html!(<HangupInNotifyIcon/>), tr!(self.i18n, CONN_ERROR)),
        };

        if self.show_video || self.show_audio {
            if let Some(info) = self.invite_info.as_ref() {
                let hangup = if info.connected {
                    ctx.link().callback(|_| PhoneCallMsg::HangUpCall)
                } else {
                    ctx.link().callback(|_| PhoneCallMsg::SendInviteCancel)
                };

                let volume = if self.volume_mute {
                    html!(<VolumeMuteIcon />)
                } else {
                    html!(<VolumeIcon />)
                };

                let microphone = if self.microphone_mute {
                    html!(<MicrophoneMuteIcon />)
                } else {
                    html!(<MicrophoneIcon />)
                };

                let volume_click = ctx.link().callback(|_| PhoneCallMsg::SwitchVolume);
                let microphone_click = ctx.link().callback(|_| PhoneCallMsg::SwitchMicrophoneMute);

                let mut ontouchstart = None;
                let mut ontouchmove = None;
                let mut ontouchend = None;
                // get class
                let zoom = if self.is_zoom {
                    ontouchstart = Some(ctx.link().callback(PhoneCallMsg::OnTouchStart));
                    ontouchmove = Some(ctx.link().callback(PhoneCallMsg::OnTouchMove));
                    ontouchend = Some(ctx.link().callback(|_| PhoneCallMsg::OnTouchEnd));
                    class = "zoom-call";
                    html!(
                        <div class="zoom-call-icon">
                            <span onclick={ctx.link().callback(|_| PhoneCallMsg::SwitchZoom)}>
                                <AudioZoomInIcon />
                            </span>
                        </div>
                    )
                } else {
                    html!(
                        <div class="zoom-call-icon" onclick={ctx.link().callback(|_| PhoneCallMsg::SwitchZoom)}>
                            <AudioZoomOutIcon />
                        </div>
                    )
                };

                let call_duration_class = if self.is_zoom {
                    "call-duration-zoom"
                } else {
                    "call-duration"
                };

                if self.show_video {
                    let self_video_style = if info.connected {
                        "animation: video-self-zoom-in .4s forwards"
                    } else {
                        ""
                    };
                    video = html! {
                        <div class={format!("video-container {}", class)}
                            ref={self.wrapper_node.clone()}
                            onmousedown={ctx.link().callback(PhoneCallMsg::OnMouseDown)}
                            onmousemove={ctx.link().callback(PhoneCallMsg::OnMouseMove)}
                            onmouseup={ctx.link().callback(|_|PhoneCallMsg::OnMouseUp)}>
                            {zoom}
                            <video class="video-self" style={self_video_style} ref={self.video_node.clone()} playsinline={true} />
                            <video class="video-friend" ref={self.friend_video_node.clone()}  playsinline={true} />
                            <div class="call-duration">{duration}</div>
                            <div class="call-operate" >
                                <span class="switch-microphone" onclick={microphone_click} >
                                    {microphone}
                                </span>
                                <span class="hangup-icon" onclick={hangup} >
                                    {hangup_icon}
                                </span>
                                <span class="call-volume" onclick={volume_click} >
                                    {volume}
                                </span>
                            </div>
                        </div>
                    };
                } else if self.show_audio {
                    let mut avatar = AttrValue::default();
                    let mut background = AttrValue::default();
                    if let Some(info) = self.call_friend_info.as_ref() {
                        avatar = info.avatar();
                        background = format!(
                            "background-image: url('{}')",
                            utils::get_avatar_url(&avatar)
                        )
                        .into();
                    }

                    // let zoom_in_click = ctx.link().callback(|_|PhoneCallMsg::AudioZoomIn);
                    audio = html! {
                        <div class={format!("audio-container {}", class)} style={background}
                            ref={self.wrapper_node.clone()}
                            onmousedown={ctx.link().callback(PhoneCallMsg::OnMouseDown)}
                            onmousemove={ctx.link().callback(PhoneCallMsg::OnMouseMove)}
                            onmouseup={ctx.link().callback(|_|PhoneCallMsg::OnMouseUp)}
                            {ontouchstart}
                            {ontouchmove}
                            {ontouchend}>
                            {zoom}
                            <img class="audio-avatar" alt="avatar" src={utils::get_avatar_url(&avatar)} />
                            <audio ref={self.friend_audio_node.clone()}/>
                            <div class={call_duration_class}>{duration}</div>
                            <div class="call-operate" >
                                    <span class="switch-microphone" onclick={microphone_click} >
                                        {microphone}
                                    </span>
                                    <span class="hangup-icon" onclick={hangup} >
                                        {hangup_icon}
                                    </span>
                                    <span class="call-volume" onclick={volume_click} >
                                        {volume}
                                    </span>
                                </div>
                        </div>
                    }
                }
            }
        }
        html! {
            <>
                {video}
                {audio}
                {video_or_audio_notify}
            </>
        }
    }
}
