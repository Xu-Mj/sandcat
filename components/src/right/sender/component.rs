use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    ClipboardEvent, DataTransferItem, DataTransferItemList, File, HtmlElement, HtmlInputElement,
    HtmlTextAreaElement,
};
use yew::prelude::*;
use yewdux::Dispatch;

use i18n::{en_us, zh_cn, LanguageType};
use icons::{CloseIcon, ImageIcon, KeyboardIcon};
use icons::{FileIcon, PhoneIcon, SmileIcon, VideoIcon, VoiceIcon};

use sandcat_sdk::model::message::{InviteMsg, InviteType, SendStatus};
use sandcat_sdk::model::voice::Voice;
use sandcat_sdk::model::RightContentType;
use sandcat_sdk::state::{MobileState, SendCallState};
use sandcat_sdk::{model::message::Message, model::ContentType};
use utils::tr;

use crate::right::emoji::Emoji;
use crate::right::recorder::Recorder;
use crate::right::sender::emoji::EmojiPanel;
use crate::right::sender::INPUT_MAX_LEN;

use super::Sender;

pub struct FileListItem {
    file: File,
    file_type: FileType,
}

pub enum FileType {
    Image,
    File,
}

pub enum SenderMsg {
    SendText,
    CleanEmptyMsgWarn,
    SendEmoji(Emoji),
    ShowEmoji,
    // SenderResize(MouseEvent),
    SendFileIconClicked,
    FileInputChanged(Event),
    SendFile,
    FileOnload(String, ContentType, JsValue),
    OnEnterKeyUp(KeyboardEvent),
    OnEnterKeyDown(KeyboardEvent),
    OnPaste(Event),
    CloseFileSender,
    DeleteFileInFileSender(String),
    SendVideoCall,
    SendAudioCall,
    OnTextInput,
    VoiceIconClicked,
    SendVoice(Voice),
}

#[derive(Properties, PartialEq, Debug)]
pub struct SenderProps {
    pub friend_id: AttrValue,
    pub conv_type: RightContentType,
    pub cur_user_id: AttrValue,
    pub disable: bool,
    pub lang: LanguageType,
    pub on_file_send: Callback<Message>,
}

impl Component for Sender {
    type Message = SenderMsg;

    type Properties = SenderProps;

    fn create(ctx: &Context<Self>) -> Self {
        let res = match ctx.props().lang {
            LanguageType::ZhCN => zh_cn::SENDER,
            LanguageType::EnUS => en_us::SENDER,
        };
        let i18n = utils::create_bundle(res);
        // 加载表情
        Self {
            is_warn_needed: false,
            warn_msg: String::new(),
            timer: None,
            show_emoji: false,
            input_ref: NodeRef::default(),
            file_input_ref: NodeRef::default(),
            sender_ref: NodeRef::default(),
            show_file_sender: false,
            i18n,
            file_list: vec![],
            is_mobile: Dispatch::<MobileState>::global().get().is_mobile(),
            enter_key_down: 0,
            is_key_down: false,
            is_voice_mode: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SenderMsg::SendText => self.send_text(ctx),
            SenderMsg::CleanEmptyMsgWarn => {
                self.is_warn_needed = false;
                self.timer = None;
                true
            }
            SenderMsg::SendEmoji(emoji) => self.send_emoji(ctx, emoji),
            SenderMsg::ShowEmoji => {
                self.show_emoji = !self.show_emoji;
                true
            }
            SenderMsg::SendFileIconClicked => {
                if let Some(file_input) = self.file_input_ref.cast::<HtmlElement>() {
                    file_input.click();
                }
                false
            }
            SenderMsg::FileInputChanged(event) => {
                let file_input: HtmlInputElement = event.target().unwrap().dyn_into().unwrap();
                let file_list = file_input.files();
                if let Some(file_list) = file_list {
                    let file_list = file_list;
                    for i in 0..file_list.length() {
                        if let Some(file) = file_list.get(i) {
                            self.send_file(ctx, file);
                        }
                    }
                }
                false
            }
            SenderMsg::SendFile => {
                for item in &self.file_list {
                    self.send_file(ctx, item.file.clone());
                }
                self.file_list = vec![];
                self.show_file_sender = false;
                false
            }
            SenderMsg::FileOnload(file_name, content_type, file_content) => {
                let time = chrono::Utc::now().timestamp_millis();
                let file_content = if let Some(file_content) = file_content.as_string() {
                    file_content.into()
                } else {
                    "".into()
                };

                let msg = Message {
                    local_id: nanoid::nanoid!().into(),
                    content: file_name.clone().into(),
                    is_self: true,
                    create_time: time,
                    friend_id: ctx.props().friend_id.clone(),
                    send_id: ctx.props().cur_user_id.clone(),
                    is_read: 1,
                    content_type,
                    file_content,
                    platform: self.get_platform(),
                    send_status: SendStatus::Sending,
                    ..Default::default()
                };

                self.store_send_msg(ctx, msg);
                // self.send_msg(ctx, msg);
                true
            }

            SenderMsg::OnPaste(event) => {
                let event1: ClipboardEvent = event.clone().dyn_into().unwrap();
                let data = event1.clipboard_data().unwrap();

                let items: DataTransferItemList = data.items();
                for i in 0..items.length() {
                    let item: DataTransferItem = items.get(i).unwrap();

                    if item.kind() == "file" {
                        if item.type_().starts_with("image") {
                            let file = item.get_as_file().unwrap();
                            if let Some(file) = file {
                                self.file_list.push(FileListItem {
                                    file,
                                    file_type: FileType::Image,
                                });
                            }
                        } else {
                            // other type file
                            let file = item.get_as_file().unwrap();
                            if let Some(file) = file {
                                self.file_list.push(FileListItem {
                                    file,
                                    file_type: FileType::File,
                                });
                            }
                        }
                        self.show_file_sender = true;
                    }
                }
                true
            }
            SenderMsg::CloseFileSender => {
                self.show_file_sender = false;
                self.file_list = vec![];
                true
            }
            SenderMsg::DeleteFileInFileSender(name) => {
                if let Some(pos) = self
                    .file_list
                    .iter()
                    .position(|item| item.file.name() == name)
                {
                    self.file_list.remove(pos);
                }
                true
            }
            SenderMsg::SendVideoCall => {
                Dispatch::<SendCallState>::global().reduce_mut(|s| {
                    s.msg = InviteMsg {
                        local_id: nanoid::nanoid!().into(),
                        server_id: AttrValue::default(),
                        create_time: chrono::Utc::now().timestamp_millis(),
                        friend_id: ctx.props().friend_id.clone(),
                        send_id: ctx.props().cur_user_id.clone(),
                        invite_type: InviteType::Video,
                        platform: self.get_platform(),
                    }
                });
                false
            }
            SenderMsg::SendAudioCall => {
                Dispatch::<SendCallState>::global().reduce_mut(|s| {
                    s.msg = InviteMsg {
                        local_id: nanoid::nanoid!().into(),
                        server_id: AttrValue::default(),
                        create_time: chrono::Utc::now().timestamp_millis(),
                        friend_id: ctx.props().friend_id.clone(),
                        send_id: ctx.props().cur_user_id.clone(),
                        invite_type: InviteType::Audio,
                        platform: self.get_platform(),
                    }
                });
                false
            }
            SenderMsg::OnTextInput => self.handle_input(),
            SenderMsg::OnEnterKeyUp(event) => {
                // handle mobile enter key long press event
                if event.key() != "Enter" {
                    return false;
                }
                let need_new_line = self.enter_key_down != 0
                    && chrono::Utc::now().timestamp_millis() - self.enter_key_down > 500;
                log::debug!(
                    "need_new_line: {}, event.shift_key: {}",
                    need_new_line,
                    event.shift_key()
                );

                self.is_key_down = false;

                if event.shift_key() || need_new_line {
                    self.enter_key_down = 0;
                    event.prevent_default();
                    self.handle_new_line(ctx);
                    return false;
                }

                ctx.link().send_message(SenderMsg::SendText);

                false
            }
            SenderMsg::OnEnterKeyDown(event) => {
                if event.key() == "Enter" && !self.is_key_down {
                    self.is_key_down = true; // 置标志变量为 true
                    self.enter_key_down = chrono::Utc::now().timestamp_millis(); // 记录按键按下时间
                    event.prevent_default(); // 阻止默认行为
                }
                false
            }
            SenderMsg::VoiceIconClicked => {
                // keyboard clicked
                if self.is_mobile && self.is_voice_mode {
                    if let Some(input) = self.input_ref.cast::<HtmlTextAreaElement>() {
                        // igonre error
                        let _ = input.style().remove_property("display");
                        let _ = input.focus();
                    }
                } else if self.is_mobile && !self.is_voice_mode {
                    self.input_ref
                        .cast::<HtmlTextAreaElement>()
                        .map(|input| input.style().set_property("display", "none"));
                }
                self.is_voice_mode = !self.is_voice_mode;
                true
            }
            SenderMsg::SendVoice(voice) => {
                log::debug!("send voice");
                Self::send_voice_msg(
                    self.get_platform(),
                    ctx.props().friend_id.clone(),
                    ctx.props().cur_user_id.clone(),
                    voice,
                    ctx.props().conv_type.clone(),
                );
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let (sender_class, input_class, warn_class, send_btn) = if self.is_mobile {
            (
                "sender",
                "msg-input msg-input-size-mobile",
                "empty-msg-tip-mobile box-shadow",
                html!(),
            )
        } else {
            (
                "sender sender-size",
                "msg-input msg-input-size",
                "empty-msg-tip box-shadow",
                html!(
                    <button class="send-btn"
                        onclick={ctx.link().callback(|_| SenderMsg::SendText)}>
                        {tr!(self.i18n, "send")}
                    </button>),
            )
        };
        // spawn disable layer
        let mut disable = html!();
        if ctx.props().disable {
            let message = match ctx.props().conv_type {
                RightContentType::Friend => tr!(self.i18n, "verify_needed"),
                RightContentType::Group => tr!(self.i18n, "group_dismissed"),
                _ => tr!(self.i18n, "disabled"),
            };
            disable = html! {
                <div class="sender-disabled">
                    {message}
                </div>
            }
        }
        // spawn warn tip
        let mut warn = html!();
        if self.is_warn_needed {
            let msg = if self.warn_msg == "input_max_len" {
                format!("{} {}", tr!(self.i18n, &self.warn_msg), INPUT_MAX_LEN)
            } else {
                tr!(self.i18n, &self.warn_msg)
            };
            warn = html! {
                <span class={warn_class}>
                    {msg}
                </span>
            }
        }

        let mut emojis = html!();
        if !self.show_emoji {
            let callback = &ctx.link().callback(SenderMsg::SendEmoji);
            let onblur = &ctx.link().callback(move |_| SenderMsg::ShowEmoji);
            // emojis = html! {
            //     <div class={emoji_class} tabindex="-1" ref={self.emoji_wrapper_ref.clone()} {onblur}>
            //         {
            //             self.emoji_list.iter()
            //             .map(|emoji| {html! (<EmojiSpan emoji={emoji.clone()} onclick={callback} />)})
            //             .collect::<Html>()
            //         }
            //     </div>
            // }
            emojis = html!(<EmojiPanel send={callback} close={onblur}/>);
        }

        // 文件发送窗口
        let file_sender = if self.show_file_sender {
            let content = self
                .file_list
                .iter()
                .map(|item| {
                    let filename = item.file.name();
                    let close = ctx
                        .link()
                        .callback(move |_| SenderMsg::DeleteFileInFileSender(filename.clone()));
                    match item.file_type {
                        FileType::Image => {
                            html! {
                                 <div class="file-sender-item" key={item.file.name()}>
                                    <ImageIcon />
                                    <span class="file-sender-name">
                                        {item.file.name()}
                                    </span>
                                    <CloseIcon />
                                </div>
                            }
                        }
                        FileType::File => {
                            html! {
                                <div class="file-sender-item" key={item.file.name()}>
                                    <FileIcon />
                                    <span class="file-sender-name">
                                        {item.file.name()}
                                    </span>
                                    <span onclick={close} >
                                        <CloseIcon />
                                    </span>
                                </div>
                            }
                        }
                    }
                })
                .collect::<Html>();
            let onclick = ctx.link().callback(|_| SenderMsg::CloseFileSender);
            let send = ctx.link().callback(|_| SenderMsg::SendFile);
            html! {
                <div class="file-sender">
                    <div class="file-sender-content" >
                        {content}
                    </div>
                    <div class="file-sender-footer">
                        <button onclick={send} >
                            {tr!(self.i18n, "submit")}
                        </button>
                        <button {onclick} >
                            {tr!(self.i18n, "cancel")}
                        </button>
                    </div>
                </div>
            }
        } else {
            html! {}
        };

        let onkeydown = ctx.link().callback(SenderMsg::OnEnterKeyDown);
        let onkeyup = ctx.link().callback(SenderMsg::OnEnterKeyUp);
        let onpaste = ctx.link().callback(SenderMsg::OnPaste);
        let video_click = ctx.link().callback(|_| SenderMsg::SendVideoCall);
        let audio_click = ctx.link().callback(|_| SenderMsg::SendAudioCall);

        let mut phone_call = html!();
        if ctx.props().conv_type == RightContentType::Friend {
            phone_call = html! {
                <>
                    <span onclick={audio_click}>
                        <PhoneIcon />
                    </span>
                    <span onclick={video_click} >
                        <VideoIcon />
                    </span>
                </>
            }
        }

        let oninput = if self.is_mobile {
            Some(ctx.link().callback(|_| SenderMsg::OnTextInput))
        } else {
            None
        };

        // voice icon
        let voice_icon = if self.is_voice_mode {
            html!(<KeyboardIcon />)
        } else {
            html!(<VoiceIcon />)
        };

        let mut recorder = html!();
        if self.is_voice_mode {
            recorder = html!(<Recorder send_voice={ctx.link().callback(SenderMsg::SendVoice)} />);
        }
        let voice_icon_click = ctx.link().callback(|_| SenderMsg::VoiceIconClicked);

        html! {
            <>
            {file_sender}
            <div class={sender_class} ref={self.sender_ref.clone()}>
            {emojis}
                <div class="send-bar">
                    <div class="send-bar-left">
                        <span onclick={ctx.link().callback(move |_| SenderMsg::ShowEmoji)}>
                            <SmileIcon />
                        </span>
                        <span >
                            <input type="file" hidden={true} ref={self.file_input_ref.clone()}
                                onchange={ctx.link().callback(SenderMsg::FileInputChanged)}/>
                            <span onclick={ctx.link().callback(|_| SenderMsg::SendFileIconClicked)}>
                                <FileIcon />
                            </span>
                        </span>
                        <span onclick={voice_icon_click}>
                            {voice_icon}
                        </span>
                    </div>
                    <div class="send-bar-right" >
                        {phone_call}
                    </div>
                </div>
                <div class="msg-input-wrapper">
                    {recorder}
                    <textarea class={input_class}
                        ref={self.input_ref.clone()}
                        {oninput}
                        {onpaste}
                        {onkeydown}
                        {onkeyup}>
                    </textarea>
                    {warn}
                    {send_btn}
                </div>
                {disable}
            </div>
            </>

        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        if ctx.props().disable {
            let _ = self
                .input_ref
                .cast::<HtmlElement>()
                .map(|input| input.blur());
            return;
        }
        if
        /* !self.show_emoji &&  */
        !ctx.props().disable && !self.is_mobile && !self.is_voice_mode {
            self.input_ref
                .cast::<HtmlElement>()
                .map(|input| input.focus());
        }
        if self.show_emoji {
            // let wrapper = self.emoji_wrapper_ref.cast::<HtmlElement>().unwrap();
            // // 设置表情面板位置
            // let sender = self.sender_ref.cast::<HtmlElement>().unwrap();
            // let gap = ".5rem";
            // wrapper
            //     .style()
            //     .set_property(
            //         "bottom",
            //         format!("calc({}px + {})", sender.client_height(), gap).as_str(),
            //     )
            //     .unwrap();
            // let _ = wrapper.focus();
        }
    }
}
