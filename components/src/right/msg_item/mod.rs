mod component;
mod related_msg;
use component::*;

use fluent::{FluentBundle, FluentResource};
use gloo::timers::callback::Timeout;
use nanoid::nanoid;
use utils::tr;
use wasm_bindgen::JsCast;
use web_sys::HtmlDivElement;
use yew::prelude::*;
use yewdux::Dispatch;

use i18n::{en_us, zh_cn, LanguageType};
use icons::{CloseIcon, ExclamationIcon, MsgLoadingIcon, MsgPhoneIcon, VideoRecordIcon};
use sandcat_sdk::db;
use sandcat_sdk::model::file_msg::FileMsg;
use sandcat_sdk::model::friend::Friend;
use sandcat_sdk::model::message::{InviteMsg, InviteType, Message, SendStatus};
use sandcat_sdk::model::ContentType;
use sandcat_sdk::model::RightContentType;
use sandcat_sdk::state::{I18nState, MobileState, Notify, SendCallState};

use crate::get_platform;

pub struct MsgItem {
    avatar: AttrValue,
    nickname: AttrValue,
    show_img_preview: bool,
    show_friend_card: bool,
    show_friendlist: bool,
    /// timeout for sending message
    timeout: Option<Timeout>,
    show_send_fail: bool,
    show_sending: bool,
    pointer: (i32, i32),
    friend_info: Option<Friend>,
    text_node: NodeRef,
    audio_icon_node: NodeRef,
    /// if timeout then show downloading icon
    show_audio_download_timer: Option<Timeout>,
    /// if timeout then show download timeout icon
    audio_download_timeout: Option<Timeout>,
    download_stage: AudioDownloadStage,
    i18n: Option<FluentBundle<FluentResource>>,
    /// right click menu
    show_context_menu: bool,
    /// hold right click item position
    context_menu_pos: (i32, i32),
    show_video_palyer: bool,
}

enum AudioDownloadStage {
    // component rendered < 200ms
    Hidden,
    Downloading,
    Timeout,
}

impl MsgItem {
    pub fn new(ctx: &Context<Self>) -> Self {
        // query data by conv type
        if ctx.props().conv_type == RightContentType::Group && !ctx.props().msg.is_self {
            let friend_id = ctx.props().msg.send_id.clone();
            let group_id = ctx.props().msg.friend_id.clone();
            log::debug!("query group member: {:?}, {:?}", friend_id, group_id);
            ctx.link().send_future(async move {
                let member = db::db_ins()
                    .group_members
                    .get_by_group_id_and_friend_id(group_id.as_str(), friend_id.as_str())
                    .await
                    .unwrap()
                    .unwrap();
                MsgItemMsg::QueryGroupMember(member.avatar, member.group_name)
            });
        }

        let avatar = ctx.props().avatar.clone();
        let nickname = ctx.props().nickname.clone();
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
            let res = match I18nState::get().lang {
                LanguageType::ZhCN => zh_cn::MSG_ITEM,
                LanguageType::EnUS => en_us::MSG_ITEM,
            };
            i18n = Some(utils::create_bundle(res));
        }

        Self {
            timeout,
            show_img_preview: false,
            show_video_palyer: false,
            show_friend_card: false,
            show_friendlist: false,
            avatar,
            nickname,
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

    fn get_call_hint(&self, ctx: &Context<Self>) -> String {
        let full_original = ctx.props().msg.content.clone();
        let mut parts = full_original.split("||");
        if parts.clone().count() < 2 {
            tr!(self.i18n.as_ref().unwrap(), &full_original)
        } else {
            let prefix = parts.next().unwrap_or(&full_original).to_string();
            let duration = parts.next().unwrap_or(&full_original).to_string();

            format!("{} {}", tr!(self.i18n.as_ref().unwrap(), &prefix), duration)
        }
    }

    fn make_call(&self, ctx: &Context<Self>, invite_type: InviteType) {
        Dispatch::<SendCallState>::global().reduce_mut(|s| {
            s.msg = InviteMsg {
                local_id: nanoid!().into(),
                server_id: AttrValue::default(),
                send_id: ctx.props().user_id.clone(),
                friend_id: ctx.props().friend_id.clone(),
                create_time: chrono::Utc::now().timestamp_millis(),
                invite_type,
                platform: get_platform(MobileState::is_mobile()),
                avatar: ctx.props().avatar.clone(),
                nickname: ctx.props().msg.nickname.clone(),
            }
        });
    }

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

    fn get_content(
        &self,
        ctx: &Context<Self>,
        msg: &Message,
        oncontextmenu: Option<Callback<MouseEvent>>,
        mut msg_content_classes: Classes,
    ) -> Html {
        let msg_type = msg.content_type;

        let content = match msg_type {
            ContentType::Text => {
                let content_lines: Vec<_> = msg.content.split('\n').collect();
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
                let onclick = ctx.link().callback(|_| MsgItemMsg::PreviewImg);
                get_img_html(msg, oncontextmenu, self.show_img_preview, onclick, None)
            }
            ContentType::Video => {
                let file = FileMsg::from(&msg.content);
                let src = format!("/api/file/get/{}", file.server_name);

                let onclick = ctx.link().callback(|event: MouseEvent| {
                    event.stop_propagation();
                    MsgItemMsg::ShowVideoPlayer
                });
                let mut video_player = html!();
                if self.show_video_palyer {
                    video_player = html! {
                        <div class="video-player" onclick={onclick.clone()} >
                            <span onclick={onclick.clone()}><CloseIcon/></span>
                            <video controls={true}>
                                <source src={src.clone()} type="video/mp4" />
                            </video>
                        </div>
                    };
                }
                html! {
                    <>
                    {video_player}
                    <div class="msg-item-content" {oncontextmenu} {onclick}>
                        <video class="msg-item-video">
                            <source {src} type="video/mp4" />
                        </video>
                    </div>
                    </>
                }
            }
            ContentType::File => get_file_html(msg, msg_content_classes.to_string(), oncontextmenu),
            ContentType::Emoji => {
                html! {
                    <div class="msg-item-emoji" {oncontextmenu}>
                        <img class="emoji" alt="emoji" src={&msg.content} />
                    </div>
                }
            }
            ContentType::VideoCall => {
                let onclick = ctx.link().callback(|_| MsgItemMsg::CallVideo);
                let text = self.get_call_hint(ctx);
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
                let text = self.get_call_hint(ctx);
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
                // if audio download success, the msg.audio_downloaded will be true
                let (icon, onclick) = if msg.audio_downloaded {
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

                let duration = msg.audio_duration;
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
        content
    }
}

fn get_file_html(
    msg: &Message,
    class: String,
    oncontextmenu: Option<Callback<MouseEvent>>,
) -> Html {
    let file = FileMsg::from(&msg.content);

    let platform = if msg.platform == 0 {
        "Desktop"
    } else {
        "Mobile"
    };

    let href = format!("/api/file/get/{}", file.server_name);
    html! {
        <div {class} {oncontextmenu} >
            <a {href} download="" class="msg-item-file-name">
                <div>
                    <p>{&file.name}</p>
                    <p>{&file.get_size()}</p>
                </div>
                {file.ext.get_icon()}
            </a>
            <div class="msg-item-platform">{platform}</div>
        </div>
    }
}

fn get_img_html(
    msg: &Message,
    oncontextmenu: Option<Callback<MouseEvent>>,
    show_preview: bool,
    onclick: Callback<MouseEvent>,
    nickname: Option<String>,
) -> Html {
    let img_url = if msg.file_content.is_empty() {
        let file = FileMsg::from(&msg.content);
        &AttrValue::from(format!("/api/file/get/{}", file.server_name))
    } else {
        &msg.file_content.clone()
    };

    let src = img_url.clone();

    let img_preview = if show_preview {
        html! {
            <div class="img-preview pointer" onclick={onclick.clone()}>
                <img alt="preview-img" {src} />
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
            {nickname}
            <img class="msg-item-img" alt="image" src={img_url} {onclick}/>
        </div>
    </>
    }
}
