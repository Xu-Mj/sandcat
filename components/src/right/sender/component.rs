use std::rc::Rc;

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{
    ClipboardEvent, DataTransferItem, DataTransferItemList, HtmlElement, HtmlInputElement,
    HtmlTextAreaElement,
};
use yew::prelude::*;
use yewdux::Dispatch;

use i18n::{en_us, zh_cn, LanguageType};
use icons::{CloseIcon, FileIcon, KeyboardIcon, SmileIcon, VoiceIcon};
use sandcat_sdk::model::voice::Voice;
use sandcat_sdk::{
    model::message::{InviteMsg, InviteType, Message, SendStatus},
    model::{ContentType, RightContentType},
    state::{MobileState, RelatedMsgState, SendCallState},
};
use utils::tr;

use crate::right::emoji::Emoji;
use crate::right::recorder::Recorder;
use crate::right::sender::INPUT_MAX_LEN;

use super::{FileListItem, FileType, Sender};

pub enum SenderMsg {
    SendText,
    CleanEmptyMsgWarn,
    SendEmoji(Emoji),
    ShowEmoji,
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
    RelatedMsgStateChanged(Rc<RelatedMsgState>),
    DelRelatMsg,
}

#[derive(Properties, PartialEq, Debug)]
pub struct SenderProps {
    pub friend_id: AttrValue,
    pub conv_type: RightContentType,
    pub cur_user_id: AttrValue,
    pub avatar: AttrValue,
    pub nickname: AttrValue,
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

        // listen related message state
        let _related_msg_state = Dispatch::global()
            .subscribe_silent(ctx.link().callback(SenderMsg::RelatedMsgStateChanged));
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
            is_mobile: MobileState::is_mobile(),
            enter_key_down: 0,
            is_key_down: false,
            is_voice_mode: false,
            related_msg: None,
            _related_msg_state,
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
                        avatar: ctx.props().avatar.clone(),
                        nickname: ctx.props().nickname.clone(),
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
                        avatar: ctx.props().avatar.clone(),
                        nickname: ctx.props().nickname.clone(),
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
                    self.is_key_down = true;
                    self.enter_key_down = chrono::Utc::now().timestamp_millis();
                    event.prevent_default();
                }
                false
            }
            SenderMsg::VoiceIconClicked => {
                // keyboard clicked
                if self.is_mobile && self.is_voice_mode {
                    if let Some(input) = self.input_ref.cast::<HtmlTextAreaElement>() {
                        // ignore error
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
                self.send_voice_msg(ctx, voice);
                false
            }
            SenderMsg::RelatedMsgStateChanged(state) => {
                if state.msg.local_id.is_empty() {
                    self.related_msg = None;
                    return false;
                }
                self.related_msg = Some((
                    state.nickname.clone(),
                    state.msg.local_id.clone(),
                    state.msg.content.clone(),
                ));
                true
            }
            SenderMsg::DelRelatMsg => {
                self.related_msg = None;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let (sender_class, input_class, warn_class, send_btn) =
            self.get_sender_classes(ctx, self.is_mobile);

        let disable_html = self.get_disable_html(ctx);

        let warn_html = if self.is_warn_needed {
            let msg = if self.warn_msg == "input_max_len" {
                format!("{} {}", tr!(self.i18n, &self.warn_msg), INPUT_MAX_LEN)
            } else {
                tr!(self.i18n, &self.warn_msg)
            };
            html!(<span class={warn_class}>{msg}</span>)
        } else {
            html!()
        };

        let emojis = self.get_emoji_panel(ctx);

        let file_sender_html = if self.show_file_sender {
            self.get_file_sender_html(ctx)
        } else {
            html!()
        };

        let phone_call_icons = self.get_phone_call_icons(ctx, &ctx.props().conv_type);

        let oninput = self
            .is_mobile
            .then(|| ctx.link().callback(|_| SenderMsg::OnTextInput));
        let voice_icon_html = if self.is_voice_mode {
            html!(<KeyboardIcon />)
        } else {
            html!(<VoiceIcon />)
        };

        let recorder_html = if self.is_voice_mode {
            html!(<Recorder send_voice={ctx.link().callback(SenderMsg::SendVoice)} />)
        } else {
            html!()
        };

        // related message
        let mut related_msg_html = html!();

        if let Some((ref nickname, _, ref content)) = self.related_msg {
            related_msg_html = html! {
                <p class="related-msg">
                    {&nickname}{":"}{&content}
                    <span onclick={ctx.link().callback(|_|SenderMsg::DelRelatMsg)}><CloseIcon/></span> </p>
            };
        }

        html! {
            <>
                {file_sender_html}
                <div class={sender_class} ref={self.sender_ref.clone()} >
                    {emojis}
                    <div class="send-bar">
                        <div class="send-bar-left">
                            <span onclick={ctx.link().callback(|event: MouseEvent| {
                                event.stop_propagation();
                                SenderMsg::ShowEmoji
                            })}>
                                <SmileIcon />
                            </span>
                            <span>
                                <input type="file" hidden=true ref={self.file_input_ref.clone()}
                                    onchange={ctx.link().callback(SenderMsg::FileInputChanged)} />
                                <span onclick={ctx.link().callback(|_| SenderMsg::SendFileIconClicked)}>
                                    <FileIcon />
                                </span>
                            </span>
                            <span onclick={ctx.link().callback(|_| SenderMsg::VoiceIconClicked)}>
                                {voice_icon_html}
                            </span>
                        </div>
                        <div class="send-bar-right">
                            {phone_call_icons}
                        </div>
                    </div>
                    <div class="msg-input-wrapper">
                        {recorder_html}
                        <textarea class={input_class}
                            ref={self.input_ref.clone()}
                            {oninput}
                            onpaste={ctx.link().callback(SenderMsg::OnPaste)}
                            onkeydown={ctx.link().callback(SenderMsg::OnEnterKeyDown)}
                            onkeyup={ctx.link().callback(SenderMsg::OnEnterKeyUp)}>
                        </textarea>
                        // sender footer contains related message, warn message and send button
                        <div class="sender-footer">
                            {warn_html}
                            {related_msg_html}
                            {send_btn}
                        </div>
                    </div>
                    {disable_html}
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

        if !ctx.props().disable && !self.is_mobile && !self.is_voice_mode {
            self.input_ref
                .cast::<HtmlElement>()
                .map(|input| input.focus());
        }
    }
}
