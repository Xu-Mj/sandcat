use fluent::{FluentBundle, FluentResource};
use gloo::timers::callback::Timeout;
use gloo::utils::{document, window};
use log::error;
use nanoid::nanoid;
use utils::tr;
use wasm_bindgen::JsCast;
use web_sys::{HtmlDivElement, Node};
use yew::platform::spawn_local;
use yew::prelude::*;
use yewdux::Dispatch;

use i18n::{en_us, zh_cn, LanguageType};
use icons::{ExclamationIcon, HangUpLoadingIcon, MsgLoadingIcon, MsgPhoneIcon, VideoRecordIcon};
use sandcat_sdk::db;
use sandcat_sdk::model::message::{
    GroupMsg, InviteMsg, InviteType, Message, Msg, SendStatus, ServerResponse,
};
use sandcat_sdk::model::user::UserWithMatchType;
use sandcat_sdk::model::ContentType;
use sandcat_sdk::model::RightContentType;
use sandcat_sdk::state::{I18nState, MobileState, SendCallState, SendMessageState};

use crate::get_platform;
use crate::right::friend_card::FriendCard;
use crate::right::msg_right_click::MsgRightClick;
pub struct MsgItem {
    avatar: AttrValue,
    show_img_preview: bool,
    show_friend_card: bool,
    // timeout for sending message
    timeout: Option<Timeout>,
    show_send_fail: bool,
    show_sending: bool,
    pointer: (i32, i32),
    friend_info: Option<UserWithMatchType>,
    text_node: NodeRef,
    audio_icon_node: NodeRef,
    // if timeout then show downloading icon
    show_audio_download_timer: Option<Timeout>,
    // if timeout then show download timeout icon
    audio_download_timeout: Option<Timeout>,
    download_stage: AudioDownloadStage,
    i18n: Option<FluentBundle<FluentResource>>,
    /// right click menu
    show_context_menu: bool,
    /// hold right click item position
    context_menu_pos: (i32, i32),
}

type FriendCardProps = (UserWithMatchType, i32, i32);
pub enum MsgItemMsg {
    PreviewImg,
    ShowFriendCard(MouseEvent),
    SpawnFriendCard(Box<FriendCardProps>),
    CallVideo,
    CallAudio,
    SendTimeout,
    ReSendMessage,
    QueryGroupMember(AttrValue),
    CloseFriendCard,
    TextDoubleClick(MouseEvent),
    PlayAudio,
    ShowAudioDownload,
    AudioDownloadTimeout,
    OnContextMenu(MouseEvent),
    CloseContextMenu,
    DeleteItem,
}

enum AudioDownloadStage {
    // component rendered < 200ms
    Hidden,
    Downloading,
    Timeout,
}

#[derive(Properties, Clone, PartialEq)]
pub struct MsgItemProps {
    pub user_id: AttrValue,
    pub friend_id: AttrValue,
    pub avatar: AttrValue,
    pub msg: Message,
    pub conv_type: RightContentType,
    pub del_item: Callback<AttrValue>,
    pub play_audio: Option<Callback<(AttrValue, Vec<u8>)>>,
}

impl Component for MsgItem {
    type Message = MsgItemMsg;
    type Properties = MsgItemProps;

    fn create(ctx: &Context<Self>) -> Self {
        // query data by conv type
        if ctx.props().conv_type == RightContentType::Group && !ctx.props().msg.is_self {
            let friend_id = ctx.props().msg.send_id.clone();
            let group_id = ctx.props().msg.friend_id.clone();
            ctx.link().send_future(async move {
                let member = db::db_ins()
                    .group_members
                    .get_by_group_id_and_friend_id(group_id.as_str(), friend_id.as_str())
                    .await
                    .unwrap();
                MsgItemMsg::QueryGroupMember(member.unwrap().avatar)
            });
        }

        let avatar = ctx.props().avatar.clone();
        let mut timeout = None;
        if ctx.props().msg.is_self && ctx.props().msg.send_status == SendStatus::Sending {
            let ctx = ctx.link().clone();
            timeout = Some(Timeout::new(3000, move || {
                ctx.send_message(MsgItemMsg::SendTimeout);
            }));
        }

        let mut timer = None;
        if ctx.props().msg.content_type == ContentType::Audio {
            let ctx = ctx.link().clone();
            timer = Some(Timeout::new(350, move || {
                ctx.send_message(MsgItemMsg::ShowAudioDownload);
            }));
        }

        // i18n
        let mut i18n = None;
        if ctx.props().msg.content_type == ContentType::VideoCall
            || ctx.props().msg.content_type == ContentType::AudioCall
        {
            let res = match Dispatch::<I18nState>::global().get().lang {
                LanguageType::ZhCN => zh_cn::MSG_ITEM,
                LanguageType::EnUS => en_us::MSG_ITEM,
            };
            i18n = Some(utils::create_bundle(res));
        }

        Self {
            timeout,
            show_img_preview: false,
            show_friend_card: false,
            avatar,
            show_send_fail: ctx.props().msg.send_status == SendStatus::Failed,
            show_sending: false,
            pointer: (0, 0),
            friend_info: None,
            text_node: NodeRef::default(),
            audio_icon_node: NodeRef::default(),
            show_audio_download_timer: timer,
            audio_download_timeout: None,
            download_stage: AudioDownloadStage::Hidden,
            i18n,
            show_context_menu: false,
            context_menu_pos: (0, 0),
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        if ctx.props().msg.send_status != SendStatus::Sending {
            self.show_send_fail = false;
            self.timeout = None;
            self.show_sending = false;
        }
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            MsgItemMsg::PreviewImg => {
                self.show_img_preview = !self.show_img_preview;
                true
            }
            MsgItemMsg::ShowFriendCard(event) => {
                // 获取xy
                let x = event.x();
                let y = event.y();

                log::debug!("friend id in msg item: {:?}", &ctx.props().msg);
                let is_self = ctx.props().msg.is_self;
                // the friend id is friend when msg type is single
                // because send id exchange the friend id when receive the message
                let friend_id = ctx.props().msg.friend_id.clone();
                let send_id = ctx.props().msg.send_id.clone();
                let user_id = ctx.props().user_id.clone();
                let conv_type = ctx.props().conv_type.clone();
                let group_id = ctx.props().friend_id.clone();
                ctx.link().send_future(async move {
                    // 查询好友信息
                    let user = if is_self {
                        let user = db::db_ins().users.get(user_id.as_str()).await.unwrap();
                        UserWithMatchType::from(user)
                    } else {
                        match conv_type {
                            RightContentType::Friend => {
                                let friend = db::db_ins().friends.get(friend_id.as_str()).await;
                                UserWithMatchType::from(friend)
                            }
                            // query group member
                            RightContentType::Group => {
                                let member = db::db_ins()
                                    .group_members
                                    .get_by_group_id_and_friend_id(
                                        group_id.as_str(),
                                        send_id.as_str(),
                                    )
                                    .await
                                    .unwrap()
                                    .unwrap();
                                UserWithMatchType::from(member)
                            }
                            _ => UserWithMatchType::default(),
                        }
                    };

                    MsgItemMsg::SpawnFriendCard(Box::new((user, x, y)))
                });
                false
            }
            MsgItemMsg::CallVideo => {
                Dispatch::<SendCallState>::global().reduce_mut(|s| {
                    s.msg = InviteMsg {
                        local_id: nanoid!().into(),
                        server_id: AttrValue::default(),
                        send_id: ctx.props().user_id.clone(),
                        friend_id: ctx.props().friend_id.clone(),
                        create_time: chrono::Utc::now().timestamp_millis(),
                        invite_type: InviteType::Video,
                        platform: get_platform(Dispatch::<MobileState>::global().get().is_mobile()),
                    }
                });
                false
            }
            MsgItemMsg::CallAudio => {
                Dispatch::<SendCallState>::global().reduce_mut(|s| {
                    s.msg = InviteMsg {
                        local_id: nanoid!().into(),
                        server_id: AttrValue::default(),
                        send_id: ctx.props().user_id.clone(),
                        friend_id: ctx.props().friend_id.clone(),
                        create_time: chrono::Utc::now().timestamp_millis(),
                        invite_type: InviteType::Audio,
                        platform: get_platform(Dispatch::<MobileState>::global().get().is_mobile()),
                    }
                });
                false
            }
            MsgItemMsg::QueryGroupMember(avatar) => {
                self.avatar = avatar;
                true
            }
            MsgItemMsg::SendTimeout => {
                if ctx.props().msg.send_status == SendStatus::Success {
                    self.timeout = None;
                    self.show_send_fail = false;
                    self.show_sending = false;
                    return true;
                }
                let msg_id = ctx.props().msg.local_id.clone();

                let conv_type = ctx.props().conv_type.clone();
                spawn_local(async move {
                    let msg = ServerResponse {
                        local_id: msg_id,
                        send_status: SendStatus::Failed,
                        err_msg: Some(AttrValue::from("TimeOut")),
                        ..Default::default()
                    };
                    match conv_type {
                        RightContentType::Friend => {
                            db::db_ins().messages.update_msg_status(&msg).await.unwrap();
                        }
                        RightContentType::Group => {
                            db::db_ins()
                                .group_msgs
                                .update_msg_status(&msg)
                                .await
                                .unwrap();
                        }
                        _ => {}
                    }
                });
                self.timeout = None;
                self.show_send_fail = true;
                self.show_sending = false;
                true
            }
            MsgItemMsg::ReSendMessage => {
                let mut msg = ctx.props().msg.clone();
                msg.send_status = SendStatus::Sending;
                let msg = match ctx.props().conv_type {
                    RightContentType::Friend => Msg::Single(msg),
                    RightContentType::Group => Msg::Group(GroupMsg::Message(msg)),
                    _ => return false,
                };

                // send message
                log::debug!("send message in message item ");
                Dispatch::<SendMessageState>::global().reduce_mut(|s| s.msg = msg);

                let ctx = ctx.link().clone();
                self.timeout = Some(Timeout::new(3000, move || {
                    ctx.send_message(MsgItemMsg::SendTimeout);
                }));
                self.show_send_fail = false;
                self.show_sending = true;
                true
            }
            MsgItemMsg::SpawnFriendCard(b) => {
                self.show_friend_card = true;
                self.friend_info = Some(b.0);
                self.pointer = (b.1, b.2);
                true
            }
            MsgItemMsg::CloseFriendCard => {
                self.show_friend_card = false;
                self.friend_info = None;
                true
            }
            MsgItemMsg::TextDoubleClick(event) => {
                event.prevent_default();
                if let Ok(Some(selection)) = window().get_selection() {
                    if selection.range_count() > 0 {
                        selection.remove_all_ranges().unwrap();
                    }
                    if let Ok(range) = document().create_range() {
                        let div = self.text_node.cast::<Node>().unwrap();
                        let _ = range.select_node_contents(&div);
                        let _ = selection.add_range(&range);
                    }
                }
                false
            }
            MsgItemMsg::PlayAudio => {
                if let Some(paly_audio) = ctx.props().play_audio.clone() {
                    let voice_id = ctx.props().msg.local_id.clone();
                    self.play_audio_animation();
                    spawn_local(async move {
                        let voice = match db::db_ins().voices.get(&voice_id).await {
                            Ok(voice) => voice,
                            Err(e) => {
                                error!("Failed to get voice: {:?}", e);
                                // todo download voice again
                                return;
                            }
                        };
                        paly_audio.emit((voice_id, voice.data));
                    });
                }
                false
            }
            MsgItemMsg::ShowAudioDownload => {
                self.download_stage = AudioDownloadStage::Downloading;
                let ctx = ctx.link().clone();
                self.show_audio_download_timer = None;
                self.audio_download_timeout = Some(Timeout::new(3000, move || {
                    ctx.send_message(MsgItemMsg::AudioDownloadTimeout);
                }));
                true
            }
            MsgItemMsg::AudioDownloadTimeout => {
                self.download_stage = AudioDownloadStage::Timeout;
                true
            }
            MsgItemMsg::OnContextMenu(event) => {
                event.prevent_default();
                self.context_menu_pos = (event.client_x(), event.client_y());
                self.show_context_menu = true;
                true
            }
            MsgItemMsg::CloseContextMenu => {
                self.show_context_menu = false;
                true
            }
            MsgItemMsg::DeleteItem => {
                let del_item = ctx.props().del_item.clone();
                let id = ctx.props().msg.id;
                let local_id = ctx.props().msg.local_id.clone();
                let content_type = ctx.props().msg.content_type;
                spawn_local(async move {
                    if content_type == ContentType::Audio {
                        // delete audio file
                        if let Err(e) = db::db_ins().voices.del(&local_id).await {
                            log::error!("delete audio file error: {:?}", e);
                            return;
                        }
                    }
                    if let Err(e) = db::db_ins().messages.delete(id).await {
                        log::error!("delete message error: {:?}", e);
                        return;
                    }
                    del_item.emit(local_id);
                });
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let id = ctx.props().msg.create_time;
        let mut classes = Classes::from("msg-item");
        let msg_type = ctx.props().msg.content_type;

        let mut msg_content_classes = Classes::from("msg-item-text");
        msg_content_classes.push("content-wrapper");
        if ctx.props().msg.is_self {
            msg_content_classes.push("background-self");
            classes = Classes::from("msg-item-reverse");
        } else {
            msg_content_classes.push("background-other");
        }

        let oncontextmenu = ctx.link().callback(Self::Message::OnContextMenu);

        let content = match msg_type {
            ContentType::Text => {
                let content_lines: Vec<_> = ctx.props().msg.content.split('\n').collect();
                let line_count = content_lines.len();

                let html_content = content_lines
                    .into_iter()
                    .enumerate()
                    .map(|(index, line)| {
                        html! {
                            <>
                                <span>{ line }</span>
                                { if index < line_count - 1 {
                                    html! { <br/> }
                                } else {
                                    html! {}
                                }}
                            </>
                        }
                    })
                    .collect::<Html>();
                html! {
                    <div
                        class={msg_content_classes}
                        {oncontextmenu}
                        ref={self.text_node.clone()}
                        ondblclick={ctx.link().callback(MsgItemMsg::TextDoubleClick)}>
                        {html_content}
                    </div>
                }
            }
            ContentType::Image => {
                let img_url = if ctx.props().msg.file_content.is_empty() {
                    let full_original = &ctx.props().msg.content;
                    let file_name_prefix =
                        full_original.split("||").next().unwrap_or(full_original);
                    AttrValue::from(format!("/api/file/get/{}", file_name_prefix))
                } else {
                    ctx.props().msg.file_content.clone()
                };
                let src = img_url.clone();
                let onclick = ctx
                    .link()
                    .callback(move |_: MouseEvent| MsgItemMsg::PreviewImg);
                let img_preview = if self.show_img_preview {
                    html! {
                        <div class="img-preview pointer" onclick={onclick.clone()}>
                            <img src={src} />
                        </div>
                    }
                } else {
                    html!()
                };
                html! {
                <>
                    {img_preview}
                    <div class="msg-item-content pointer" {oncontextmenu}>
                        <div class="img-mask">
                        </div>
                        <img class="msg-item-img" src={img_url} {onclick}/>
                    </div>
                </>
                }
            }
            ContentType::Video => html! {
                <div class="msg-item-content" {oncontextmenu}>
                    <video class="msg-item-video">
                        <source src={ctx.props().msg.content.clone()} type="video/mp4" />
                    </video>
                </div>
            },
            ContentType::File => {
                let full_original = ctx.props().msg.content.clone();
                let mut parts = full_original.split("||");
                let file_name_prefix = parts.next().unwrap_or(&full_original).to_string();
                let file_name = parts.next().unwrap_or(&full_original).to_string();

                html! {
                    <div class="msg-item-content" {oncontextmenu}>
                        <a href={file_name_prefix} download="" class="msg-item-file-name">
                            {file_name}
                        </a>
                    </div>
                }
            }
            ContentType::Emoji => {
                html! {
                    <div class="msg-item-emoji" {oncontextmenu}>
                        // <span class="msg-item-emoji">
                            <img class="emoji" src={ctx.props().msg.content.clone()} />
                        // </span>
                    </div>
                }
            }
            ContentType::VideoCall => {
                let onclick = ctx.link().callback(|_| MsgItemMsg::CallVideo);
                let full_original = ctx.props().msg.content.clone();
                let mut parts = full_original.split("||");
                let text = if parts.clone().count() < 2 {
                    tr!(self.i18n.as_ref().unwrap(), &full_original)
                } else {
                    let prefix = parts.next().unwrap_or(&full_original).to_string();
                    let duration = parts.next().unwrap_or(&full_original).to_string();

                    format!("{} {}", tr!(self.i18n.as_ref().unwrap(), &prefix), duration)
                };
                html! {
                    <div class={msg_content_classes} {oncontextmenu} {onclick} style="cursor: pointer; user-select: none;">
                        {text}
                        {"\t"}
                        <VideoRecordIcon/>
                    </div>
                }
            }
            ContentType::AudioCall => {
                let onclick = ctx.link().callback(|_| MsgItemMsg::CallAudio);
                let full_original = ctx.props().msg.content.clone();
                let mut parts = full_original.split("||");
                let text = if parts.clone().count() < 2 {
                    tr!(self.i18n.as_ref().unwrap(), &full_original)
                } else {
                    let prefix = parts.next().unwrap_or(&full_original).to_string();
                    let duration = parts.next().unwrap_or(&full_original).to_string();

                    format!("{} {}", tr!(self.i18n.as_ref().unwrap(), &prefix), duration)
                };
                html! {
                    <div class={msg_content_classes} {oncontextmenu} {onclick} style="cursor: pointer; user-select: none;">
                        {text}
                        {"\t"}
                         <MsgPhoneIcon />
                    </div>
                }
            }
            ContentType::Default => html!(),
            ContentType::Audio => {
                // if audio download success, the ctx.props().msg.audio_downloaded will be true
                let (icon, onclick) = if ctx.props().msg.audio_downloaded {
                    (
                        self.voice_in_msg_icon(),
                        Some(ctx.link().callback(|_| MsgItemMsg::PlayAudio)),
                    )
                } else {
                    match self.download_stage {
                        AudioDownloadStage::Hidden => (
                            self.voice_in_msg_icon(),
                            Some(ctx.link().callback(|_| MsgItemMsg::PlayAudio)),
                        ),
                        AudioDownloadStage::Downloading => (html!(<MsgLoadingIcon />), None),
                        AudioDownloadStage::Timeout => (html!(<ExclamationIcon />), None),
                    }
                };

                let duration = ctx.props().msg.audio_duration;
                msg_content_classes.push("audio-msg-item");

                html! {
                    <div class={msg_content_classes} {oncontextmenu} {onclick}>
                        {icon}
                        <span>{format!("{}''", duration)}</span>
                    </div>
                }
            }
            ContentType::Error => html!(),
        };

        let _avatar_click = ctx.link().callback(MsgItemMsg::ShowFriendCard);

        // send status
        let mut send_status = html!();
        if self.show_send_fail {
            let onclick = ctx.link().callback(|_| MsgItemMsg::ReSendMessage);
            send_status = html! {
                <div class="msg-send-failed" {onclick}>
                    <span class="pointer">
                        {"!"}
                    </span>
                </div>
            };
        } else if self.show_sending {
            send_status = html! {
                <div class="msg-sending">
                    <HangUpLoadingIcon fill={AttrValue::from("#000000")}/>
                </div>
            };
        }

        let mut friend_card = html!();
        if self.show_friend_card {
            friend_card = html! {
                <FriendCard
                    friend_info={self.friend_info.as_ref().unwrap().clone()}
                    user_id={&ctx.props().user_id}
                    lang={LanguageType::ZhCN}
                    close={ctx.link().callback(|_| MsgItemMsg::CloseFriendCard)}
                    is_self={ctx.props().msg.is_self}
                    x={self.pointer.0}
                    y={self.pointer.1}
                />
            };
        }

        let avatar = if ctx.props().msg.is_self {
            html!(<img class="avatar" src={&self.avatar} />)
        } else {
            html!(<img class="avatar pointer" src={&self.avatar} onclick={_avatar_click} />)
        };

        // context menu
        let mut context_menu = html!();
        if self.show_context_menu {
            context_menu = html! {
                <MsgRightClick
                    x={self.context_menu_pos.0}
                    y={self.context_menu_pos.1}
                    close={ctx.link().callback( |_|MsgItemMsg::CloseContextMenu)}
                    delete={ctx.link().callback(|_|MsgItemMsg::DeleteItem)}
                    />
            }
        }
        html! {
            <>
            {friend_card}
            {context_menu}
            <div class={classes} id={id.to_string()} >
                <div class="msg-item-avatar">
                    {avatar}
                </div>
                {content}
                {send_status}
            </div>
            </>
        }
    }
}
impl MsgItem {
    fn voice_in_msg_icon(&self) -> Html {
        html! {
            <div id="voice-in-msg-icon" ref={self.audio_icon_node.clone()}>
                <div style="height: .3rem; "></div>
                <div style="height: .4rem; "></div>
                <div style="height: .9rem; "></div>
                <div style="height: .5rem; "></div>
                <div style="height: .2rem; "></div>
            </div>
        }
    }

    fn play_audio_animation(&self) {
        if let Some(div) = self.audio_icon_node.cast::<HtmlDivElement>() {
            for index in 0..div.child_element_count() {
                div.child_nodes().get(index).map(|node| {
                    node.dyn_into::<HtmlDivElement>().map(|div| {
                        let _ = div.style().remove_property("animation");
                        // reset style
                        div.offset_width();
                        let _ = div.style().set_property(
                            "animation",
                            format!("voice-play .4s linear {}s", index as f32 / 10. + 0.1).as_str(),
                        );
                    })
                });
            }
        }
    }
}
