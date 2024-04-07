use std::cell::RefCell;
use std::rc::Rc;

use gloo::timers::callback::Timeout;
use nanoid::nanoid;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    HtmlAudioElement, HtmlDivElement, HtmlVideoElement, MediaStream, MediaStreamTrack, MouseEvent,
    RtcIceCandidateInit, RtcPeerConnection, RtcSdpType, RtcSessionDescription,
    RtcSessionDescriptionInit, RtcSignalingState,
};
use yew::platform::spawn_local;
use yew::{html, AttrValue, Component, Context, ContextHandle, Html, NodeRef, Properties};

use crate::icons::{
    AnswerPhoneIcon, HangupInNotifyIcon, MicrophoneIcon, MicrophoneMuteIcon, VideoRecordIcon,
    VolumeIcon, VolumeMuteIcon,
};
use crate::model::message::{
    Agree, Hangup, InviteAnswerMsg, InviteCancelMsg, InviteInfo, InviteMsg, InviteNotAnswerMsg,
    InviteType, Message, Msg, SingleCall,
};
use crate::model::notification::{Notification, NotificationState, NotificationType};
use crate::model::ContentType;
use crate::model::ItemInfo;
use crate::pages::{RecSendCallState, RecSendMessageState};
use crate::ws::WebSocketManager;
use crate::{db, utils, web_rtc};

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
    pc: Option<RtcPeerConnection>,
    /// 音视频流
    stream: Option<MediaStream>,
    /// 通话的好友信息
    call_friend_info: Option<Box<dyn ItemInfo>>,
    /// 邀请计时器，到时间即为未接听
    call_timer: Option<Timeout>,
    /// 用来监听是否有通话消息
    _listener: ContextHandle<SingleCall>,
    /// 通话状态， 用来挂断、取消等等。。
    call_state: Rc<RecSendCallState>,
    _call_listener: ContextHandle<Rc<RecSendCallState>>,
    /// 发送通知
    notify_state: Rc<NotificationState>,
    _notify_listener: ContextHandle<Rc<NotificationState>>,
    /// send receive message
    msg_state: Rc<RecSendMessageState>,
    _msg_listener: ContextHandle<Rc<RecSendMessageState>>,
    /// 面板拖动记录x、y坐标
    pos_x: i32,
    pos_y: i32,
    /// 是否正在拖动面板
    is_dragging: bool,
}

#[derive(Properties, Clone, PartialEq)]
pub struct PhoneCallProps {
    pub ws: Rc<RefCell<WebSocketManager>>,
    pub user_id: AttrValue,
}

pub enum PhoneCallMsg {
    // 发送视频电话请求
    SendCallInvite(InviteMsg),
    // 发送取消邀请消息
    SendInviteCancel,
    // 同意并进行接通视频通话
    ResponseCall,
    // 挂断视频
    HangUpCall,
    // 同意视频通话请求
    AgreeCall,
    // 类似与VideoOnReady（当本地视频流准备好之后完全接通视频电话）
    ConnectedCall(MediaStream),
    DenyCall,
    DisConnCall(AttrValue),
    ShowVideoWindow(MediaStream, Box<dyn ItemInfo>),
    ShowAudioWindow(MediaStream, Box<dyn ItemInfo>),
    CallTimeout,
    Notification(Notification),
    // 显示顶部消息通知
    ShowCallNotify(Box<dyn ItemInfo>),
    SwitchVolume,
    // SwitchCamera,
    SwitchMicrophoneMute,
    ReceiveCallMsg(SingleCall),
    SendMessage(SingleCall),
    SendInsideMessage(SingleCall),
    CallStateChange(Rc<RecSendCallState>),
    None,
    OnMouseDown(MouseEvent),
    OnMouseMove(MouseEvent),
    OnMouseUp,
}

impl Component for PhoneCall {
    type Message = PhoneCallMsg;
    type Properties = PhoneCallProps;

    fn create(ctx: &Context<Self>) -> Self {
        let (_, _listener) = ctx
            .link()
            .context(ctx.link().callback(PhoneCallMsg::ReceiveCallMsg))
            .expect("need msg context");
        let (call_state, _call_listener) = ctx
            .link()
            .context(ctx.link().callback(PhoneCallMsg::CallStateChange))
            .expect("need msg context");
        let (notify_state, _notify_listener) = ctx
            .link()
            .context(ctx.link().callback(|_| PhoneCallMsg::None))
            .expect("need msg context");
        let (msg_state, _msg_listener) = ctx
            .link()
            .context(ctx.link().callback(|_| PhoneCallMsg::None))
            .expect("need conv state in item");
        Self {
            show_video: false,
            show_audio: false,
            friend_audio_node: NodeRef::default(),
            invited: false,
            video_node: NodeRef::default(),
            friend_video_node: NodeRef::default(),
            wrapper_node: NodeRef::default(),
            invite_info: None,
            pc: None,
            stream: None,
            show_notify: false,
            call_friend_info: None,
            call_timer: None,
            volume_mute: false,
            microphone_mute: false,
            _listener,
            call_state,
            _call_listener,
            notify_state,
            _notify_listener,
            msg_state,
            _msg_listener,
            pos_x: 0,
            pos_y: 0,
            is_dragging: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PhoneCallMsg::SendCallInvite(msg) => {
                // 判断是否正在通话中
                if self.invite_info.is_some() {
                    log::debug!("占线");
                    // 给出提示
                    ctx.link()
                        .send_message(PhoneCallMsg::Notification(Notification {
                            type_: NotificationType::Warn,
                            title: AttrValue::from("Warn"),
                            content: AttrValue::from("您正在通话中!"),
                        }));
                    return false;
                }
                self.invite_info = Some(InviteInfo {
                    send_id: ctx.props().user_id.clone(),
                    friend_id: msg.friend_id.clone(),
                    invite_type: msg.invite_type.clone(),
                    start_time: chrono::Local::now().timestamp_millis(),
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
                let send_msg_event = self.call_state.send_msg_event.clone();
                ctx.link().send_future(async move {
                    let friend = db::friends().await.get_friend(friend_id.as_str()).await;

                    match call_type {
                        InviteType::Video => match utils::get_video_stream().await {
                            Ok(stream) => {
                                send_msg_event.emit(Msg::SingleCall(SingleCall::Invite(msg)));
                                PhoneCallMsg::ShowVideoWindow(stream, Box::new(friend))
                            }
                            Err(e) => {
                                let content = if let Some(dom_exception) =
                                    e.dyn_ref::<web_sys::DomException>()
                                {
                                    log::warn!("dom exception: {}", dom_exception.name());
                                    if dom_exception.name() == "NotFoundError" {
                                        log::warn!("没有检测到音视频设备");
                                        "没有检测到音视频设备"
                                    } else {
                                        "其他错误"
                                    }
                                } else {
                                    log::error!("未知错误获取音频流: {:?}", e);
                                    "其他错误"
                                };
                                PhoneCallMsg::Notification(Notification {
                                    type_: NotificationType::Error,
                                    title: AttrValue::from("ERROR"),
                                    content: AttrValue::from(content),
                                })
                            }
                        },
                        InviteType::Audio => {
                            // 查询好友信息
                            match utils::get_audio_stream().await {
                                Ok(stream) => {
                                    send_msg_event.emit(Msg::SingleCall(SingleCall::Invite(msg)));
                                    PhoneCallMsg::ShowAudioWindow(stream, Box::new(friend))
                                }
                                Err(e) => {
                                    let content = if let Some(dom_exception) =
                                        e.dyn_ref::<web_sys::DomException>()
                                    {
                                        log::warn!("dom exception: {}", dom_exception.name());
                                        if dom_exception.name() == "NotFoundError" {
                                            log::warn!("没有检测到音频设备");
                                            "没有检测到音视频设备"
                                        } else {
                                            "其他错误"
                                        }
                                    } else {
                                        log::error!("未知错误获取音频流: {:?}", e);
                                        "其他错误"
                                    };
                                    PhoneCallMsg::Notification(Notification {
                                        type_: NotificationType::Error,
                                        title: AttrValue::from("ERROR"),
                                        content: AttrValue::from(content),
                                    })
                                }
                            }
                        }
                    }
                });
                true
            }
            PhoneCallMsg::SendInviteCancel => {
                let msg_id = AttrValue::from(nanoid!());
                let info = self.invite_info.as_ref().unwrap();
                let friend_id = info.friend_id.clone();
                let create_time = chrono::Local::now().timestamp_millis();
                let send_id = ctx.props().user_id.clone();

                // 数据入库
                let content_type = match info.invite_type {
                    InviteType::Video => ContentType::VideoCall,
                    InviteType::Audio => ContentType::AudioCall,
                };
                let invite_type = info.invite_type.clone();
                ctx.link().send_future(async move {
                    let _ = db::messages()
                        .await
                        .add_message(&mut Message {
                            id: 0,
                            msg_id: msg_id.clone(),
                            send_id: send_id.clone(),
                            friend_id: friend_id.clone(),
                            content_type,
                            content: AttrValue::from("已经取消"),
                            create_time,
                            is_read: false,
                            is_self: true,
                            file_content: Default::default(),
                        })
                        .await
                        .map_err(|err| log::error!("消息入库失败:{:?}", err));
                    PhoneCallMsg::SendMessage(SingleCall::InviteCancel(InviteCancelMsg {
                        msg_id,
                        send_id,
                        friend_id,
                        create_time,
                        invite_type,
                        is_self: true,
                    }))
                });
                self.finish_call();
                true
            }
            PhoneCallMsg::ResponseCall => {
                self.invited = true;
                // 读取视频流
                let invite_type = self.invite_info.as_ref().unwrap().invite_type.clone();
                ctx.link().send_future(async move {
                    match invite_type {
                        InviteType::Video => match utils::get_video_stream().await {
                            Ok(stream) => PhoneCallMsg::ConnectedCall(stream),
                            Err(e) => PhoneCallMsg::Notification(Notification {
                                type_: NotificationType::Error,
                                title: AttrValue::from("ERROR"),
                                content: format!("get video stream error: {:?}", e).into(),
                            }),
                        },
                        InviteType::Audio => match utils::get_audio_stream().await {
                            Ok(stream) => PhoneCallMsg::ConnectedCall(stream),
                            Err(e) => PhoneCallMsg::Notification(Notification {
                                type_: NotificationType::Error,
                                title: AttrValue::from("ERROR"),
                                content: format!("get video stream error: {:?}", e).into(),
                            }),
                        },
                    }
                });
                false
            }
            PhoneCallMsg::HangUpCall => {
                let info = self.invite_info.as_ref().unwrap();
                let create_time = chrono::Local::now().timestamp_millis();
                let sustain = create_time - info.start_time;
                let msg_id = AttrValue::from(nanoid!());
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
                // 消息入库
                ctx.link().send_future(async move {
                    db::messages()
                        .await
                        .add_message(&mut Message {
                            id: 0,
                            msg_id: msg_id.clone(),
                            send_id: send_id.clone(),
                            friend_id: friend_id.clone(),
                            content_type,
                            content: AttrValue::from(utils::format_milliseconds(sustain)),
                            create_time,
                            is_read: true,
                            is_self: true,
                            file_content: Default::default(),
                        })
                        .await
                        .map_err(|err| log::error!("消息入库失败:{:?}", err))
                        .unwrap();
                    PhoneCallMsg::SendMessage(SingleCall::HangUp(Hangup {
                        msg_id,
                        send_id,
                        friend_id,
                        create_time,
                        invite_type,
                        sustain,
                        is_self: true,
                    }))
                });
                self.finish_call();
                true
            }
            PhoneCallMsg::AgreeCall => {
                // 同意视频通话
                let info = self.invite_info.as_ref().unwrap();
                let msg = SingleCall::InviteAnswer(InviteAnswerMsg {
                    msg_id: nanoid!().into(),
                    send_id: ctx.props().user_id.clone(),
                    friend_id: self.invite_info.as_ref().unwrap().send_id.clone(),
                    create_time: chrono::Local::now().timestamp_millis(),
                    agree: true,
                    is_self: true,
                    invite_type: info.invite_type.clone(),
                });
                self.call_state.send_msg_event.emit(Msg::SingleCall(msg));
                self.show_notify = false;
                match info.invite_type {
                    InviteType::Video => {
                        self.show_video = true;
                    }
                    InviteType::Audio => {
                        self.show_audio = true;
                    }
                }
                true
            }
            PhoneCallMsg::ConnectedCall(stream) => {
                // 设置视频流
                log::debug!("同意请求，正在连接...");
                let pc = self.pc.as_ref().unwrap().clone();
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
                let pc = pc.clone();
                let send_id = invite_info.friend_id.clone();
                let friend_id = invite_info.send_id.clone();
                spawn_local(async move {
                    let js_value = js_future.await.unwrap();
                    let rtc_desc = RtcSessionDescription::from(js_value);
                    let mut desc = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
                    desc.sdp(&rtc_desc.sdp());
                    let _ = pc.set_local_description(&desc);
                    web_rtc::WebRTC::send_msg1(
                        ws,
                        Msg::SingleCall(SingleCall::Agree(Agree {
                            sdp: Some(rtc_desc.sdp()),
                            send_id,
                            friend_id,
                            create_time: 0,
                        })),
                    );
                });
                self.invite_info.as_mut().unwrap().connected = true;
                self.invite_info.as_mut().unwrap().start_time =
                    chrono::Local::now().timestamp_millis();

                true
            }
            PhoneCallMsg::DenyCall => {
                let info = self.invite_info.as_ref().unwrap();
                let msg_id = AttrValue::from(nanoid!());
                let send_id = ctx.props().user_id.clone();
                let friend_id = info.send_id.clone();
                let content_type = match info.invite_type {
                    InviteType::Video => ContentType::VideoCall,
                    InviteType::Audio => ContentType::AudioCall,
                };
                let create_time = chrono::Local::now().timestamp_millis();
                let invite_type = info.invite_type.clone();
                self.show_notify = false;
                ctx.link().send_future(async move {
                    let _ = db::messages()
                        .await
                        .add_message(&mut Message {
                            id: 0,
                            msg_id: msg_id.clone(),
                            send_id: send_id.clone(),
                            friend_id: friend_id.clone(),
                            content_type,
                            content: AttrValue::from("已拒绝"),
                            create_time,
                            is_read: true,
                            is_self: true,
                            file_content: Default::default(),
                        })
                        .await;
                    PhoneCallMsg::SendMessage(SingleCall::InviteAnswer(InviteAnswerMsg {
                        msg_id,
                        send_id,
                        friend_id,
                        create_time,
                        agree: false,
                        invite_type,
                        is_self: true,
                    }))
                });
                true
            }
            PhoneCallMsg::DisConnCall(_) => {
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
                self.call_timer = Some(Timeout::new(60 * 1000, move || {
                    ctx.send_message(PhoneCallMsg::CallTimeout);
                }));
                false
            }
            PhoneCallMsg::CallTimeout => {
                if self.show_video || self.show_audio {
                    let info = self.invite_info.as_ref().unwrap();
                    if info.connected {
                        return false;
                    }
                    let msg_id = AttrValue::from(nanoid!());
                    let friend_id = info.friend_id.clone();
                    let create_time = chrono::Local::now().timestamp_millis();
                    let send_id = ctx.props().user_id.clone();
                    // 数据入库
                    let content_type = match info.invite_type {
                        InviteType::Video => ContentType::VideoCall,
                        InviteType::Audio => ContentType::AudioCall,
                    };
                    let invite_type = info.invite_type.clone();
                    ctx.link().send_future(async move {
                        let _ = db::messages()
                            .await
                            .add_message(&mut Message {
                                id: 0,
                                msg_id: msg_id.clone(),
                                send_id: send_id.clone(),
                                friend_id: friend_id.clone(),
                                content_type,
                                content: AttrValue::from("未接听"),
                                create_time,
                                is_read: false,
                                is_self: true,
                                file_content: Default::default(),
                            })
                            .await
                            .map_err(|err| log::error!("消息入库失败:{:?}", err));
                        PhoneCallMsg::SendMessage(SingleCall::NotAnswer(InviteNotAnswerMsg {
                            msg_id,
                            send_id,
                            friend_id,
                            create_time,
                            invite_type,
                            is_self: true,
                        }))
                    });
                    self.finish_call();
                    // self.info(AttrValue::from("未接听"));
                    // let ctx = ctx.link().clone();
                    // if self.notification_interval.is_none() {
                    //     self.notification_interval = Some(Interval::new(3 * 1000, move || { ctx.send_message(PhoneCallMsg::CleanNotification) }));
                    // }
                    return true;
                }
                false
            }
            PhoneCallMsg::Notification(item) => {
                let type_ = item.type_.clone();
                self.notify_state.notify.emit(item);
                if type_ == NotificationType::Error {
                    self.finish_call();
                    return true;
                }
                false
            }
            PhoneCallMsg::SwitchVolume => {
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
                self.microphone_mute = !self.microphone_mute;
                self.mute_audio(self.microphone_mute);
                true
            }
            PhoneCallMsg::ShowAudioWindow(stream, friend) => {
                self.stream = Some(stream);
                self.call_friend_info = Some(friend);
                let ctx = ctx.link().clone();
                self.call_timer = Some(Timeout::new(60 * 1000, move || {
                    ctx.send_message(PhoneCallMsg::CallTimeout);
                }));
                true
            }
            PhoneCallMsg::ReceiveCallMsg(mut message) => {
                match message {
                    SingleCall::Invite(msg) => {
                        // 判断是否占线
                        if self.invite_info.is_some() {
                            // todo回复占线
                            return false;
                        }
                        self.show_notify = true;
                        self.invited = true;
                        let friend_id = msg.send_id.clone();
                        self.invite_info = Some(InviteInfo {
                            send_id: msg.send_id,
                            friend_id: msg.friend_id,
                            invite_type: msg.invite_type.clone(),
                            ..Default::default()
                        });
                        ctx.link().send_future(async move {
                            // 查询好友数据
                            let friend = db::friends().await.get_friend(friend_id.as_str()).await;
                            PhoneCallMsg::ShowCallNotify(Box::new(friend))
                        });
                    }
                    SingleCall::InviteCancel(ref mut msg) => {
                        log::debug!("对方取消通话");
                        // 判断是否是当前用户
                        if let Some(info) = self.invite_info.as_ref() {
                            if info.send_id == msg.send_id {
                                log::debug!("正在关闭通知");

                                self.finish_call();
                                // 数据入库
                                let friend_id = msg.send_id.clone();
                                msg.send_id = msg.friend_id.clone();
                                msg.friend_id = friend_id;
                                self.show_notify = false;
                                self.call_friend_info = None;
                                self.save_call_msg(ctx, msg.clone_as_message(), message);
                                log::debug!("已经关闭通知");
                                return true;
                            }
                        }
                    }
                    SingleCall::InviteAnswer(ref mut msg) => {
                        log::debug!("single invite answer");
                        if msg.agree {
                            // 对方同意了通话请求，正在建立连接，为了简化代码邀请方和被邀请方的创建pc方法合并到一起了
                            if let Err(e) = self.create_pc(ctx, "") {
                                ctx.link()
                                    .send_message(PhoneCallMsg::Notification(Notification {
                                        type_: Default::default(),
                                        title: AttrValue::from("创建PC错误"),
                                        content: AttrValue::from(e.as_string().unwrap()),
                                    }));
                                return false;
                            }
                        } else {
                            // 拒绝通话请求，数据入库
                            log::debug!("对方拒绝了请求");
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
                            self.save_call_msg(ctx, msg.clone_as_message(), message);
                            self.finish_call();
                            return true;
                        }
                    }
                    SingleCall::Offer(msg) => {
                        // 建立通话连接，收到offer，设置sdp
                        if self.pc.is_some() {
                            log::warn!("收到邀请，但是占线: {:?}", &msg);
                            return false;
                        }
                        if let Err(e) = self.create_pc(ctx, &msg.sdp) {
                            log::error!("创建连接失败:{:?}", e);
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
                            if info.friend_id == msg.send_id && self.pc.is_some() {
                                // 接通
                                log::debug!("请求被对方同意");
                                // todo需要在webrtc状态为Connected下进行回调修改
                                self.invite_info.as_mut().unwrap().start_time =
                                    chrono::Local::now().timestamp_millis();
                                self.invite_info.as_mut().unwrap().connected = true;
                                let mut description =
                                    RtcSessionDescriptionInit::new(RtcSdpType::Answer);
                                description.sdp(&msg.sdp.unwrap());
                                let future = JsFuture::from(
                                    self.pc
                                        .as_ref()
                                        .unwrap()
                                        .set_remote_description(&description),
                                );
                                spawn_local(async {
                                    if let Err(err) = future.await {
                                        log::error!("remote desc set failed: {:?}", err);
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
                            self.pc
                                .as_ref()
                                .unwrap()
                                .add_ice_candidate_with_opt_rtc_ice_candidate_init(Some(
                                    &candidate,
                                )),
                        );
                        spawn_local(async {
                            if let Err(err) = future.await {
                                log::error!("set ice candidate failed: {:?}", err);
                            }
                        })
                    }
                    SingleCall::NotAnswer(ref mut msg) => {
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
                                self.save_call_msg(ctx, msg.clone_as_message(), message);
                                return true;
                            }
                        }
                    }
                    SingleCall::HangUp(ref mut msg) => {
                        // 判断是否是当前连接
                        if let Some(info) = self.invite_info.as_ref() {
                            if info.send_id == msg.friend_id || info.send_id == msg.send_id {
                                let friend_id = msg.send_id.clone();
                                msg.send_id = msg.friend_id.clone();
                                msg.friend_id = friend_id;
                                let info = self.invite_info.as_ref().unwrap();
                                let create_time = chrono::Local::now().timestamp_millis();
                                let sustain = create_time - info.start_time;
                                msg.sustain = sustain;
                                self.save_call_msg(ctx, msg.clone_as_message(), message);
                                self.finish_call();
                                return true;
                            }
                        }
                    }
                }
                false
            }
            PhoneCallMsg::ShowCallNotify(item) => {
                self.call_friend_info = Some(item);
                true
            }
            PhoneCallMsg::SendMessage(msg) => {
                self.call_state.send_msg_event.emit(Msg::SingleCall(msg));
                false
            }
            PhoneCallMsg::CallStateChange(state) => {
                // 判断是否是空的状态
                if state.msg.msg_id.is_empty() {
                    return false;
                }
                ctx.link()
                    .send_message(PhoneCallMsg::SendCallInvite(state.msg.clone()));
                self.call_state = state;
                true
            }
            PhoneCallMsg::None => false,
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
                // 设置新的位置
                if self.invite_info.is_some() {
                    if let Some(div) = self.wrapper_node.cast::<HtmlDivElement>() {
                        div.style()
                            .set_property("top", &format!("{}px", div.offset_top() - y))
                            .map_err(|e| log::error!("set top error: {:?}", e))
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
            PhoneCallMsg::SendInsideMessage(msg) => {
                self.call_state.rec_msg_event.emit(Msg::SingleCall(msg));
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
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
                    <img src={info.avatar()}/>
                    <span class="video-or-audio-notify-text" >
                        {format!("{} 来电", info.name())}
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
                if self.show_video {
                    let self_video_style = if info.connected {
                        "animation: video-self-zoom-in .4s forwards"
                    } else {
                        ""
                    };
                    video = html! {
                        <div class="video-container box-shadow"
                            ref={self.wrapper_node.clone()}
                            onmousedown={ctx.link().callback(PhoneCallMsg::OnMouseDown)}
                            onmousemove={ctx.link().callback(PhoneCallMsg::OnMouseMove)}
                            onmouseup={ctx.link().callback(|_|PhoneCallMsg::OnMouseUp)}>
                            <video class="video-self" style={self_video_style} ref={self.video_node.clone()} playsinline={true} />
                            <video class="video-friend" ref={self.friend_video_node.clone()}  playsinline={true} />
                            <div class="call-operate" >
                                <span class="switch-microphone" onclick={microphone_click} >
                                    {microphone}
                                </span>
                                <span class="hangup-icon" onclick={hangup} >
                                    <HangupInNotifyIcon/>
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
                        background = format!("background-image: url('{}')", info.avatar()).into();
                    }

                    // let zoom_in_click = ctx.link().callback(|_|PhoneCallMsg::AudioZoomIn);
                    audio = html! {
                        <div class="audio-container box-shadow" style={background}
                            ref={self.wrapper_node.clone()}
                            onmousedown={ctx.link().callback(PhoneCallMsg::OnMouseDown)}
                            onmousemove={ctx.link().callback(PhoneCallMsg::OnMouseMove)}
                            onmouseup={ctx.link().callback(|_|PhoneCallMsg::OnMouseUp)}>
                            // <div class="audio-zoom">
                            //     <span click={zoom_in_click}>
                            //     <AudioZoomInIcon/>
                            //     </span>
                            // </div>
                            <img class="audio-avatar" src={avatar} />
                            <audio ref={self.friend_audio_node.clone()}/>
                            <div class="call-operate" >
                                    <span class="switch-microphone" onclick={microphone_click} >
                                        {microphone}
                                    </span>
                                    <span class="hangup-icon" onclick={hangup} >
                                        <HangupInNotifyIcon/>
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

impl PhoneCall {
    fn create_pc(&mut self, ctx: &Context<Self>, sdp: &str) -> Result<(), JsValue> {
        let callback = ctx.link().callback(PhoneCallMsg::DisConnCall);
        let invite_info = self.invite_info.as_ref().unwrap();
        let mut friend_id = invite_info.friend_id.clone();
        if self.invited {
            friend_id = invite_info.send_id.clone();
        }
        let pc = web_rtc::WebRTC::create_pc(
            ctx.props().ws.clone(),
            ctx.props().user_id.clone(),
            friend_id,
            callback,
            invite_info.invite_type.clone(),
            self.friend_video_node.clone(),
            self.friend_audio_node.clone(),
        )?;
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
        self.pc = Some(pc);
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
        if let Some(pc) = self.pc.as_ref() {
            log::debug!("hang up video clear pc");
            pc.close();
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
        self.pc = None;
        self.stream = None;
        self.invite_info = None;
        self.invited = false;
        self.call_timer = None;
        self.call_friend_info = None;
        self.volume_mute = false;
        self.microphone_mute = false;
    }

    fn save_call_msg(&self, ctx: &Context<Self>, mut msg: Message, message: SingleCall) {
        let msg_id = msg.msg_id.clone();
        ctx.link().send_future(async move {
            db::messages()
                .await
                .add_message(&mut msg)
                .await
                .map_err(|err| log::error!("消息入库失败:{:?}", err))
                .unwrap();
            PhoneCallMsg::SendInsideMessage(message)
        });
        // send receive message
        self.msg_state
            .send_back_event
            .emit(Msg::SingleDeliveredNotice(msg_id.to_string()));
    }
}
