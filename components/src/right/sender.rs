use std::cmp::Ordering;

use fluent::{FluentBundle, FluentResource};
use futures_channel::oneshot;
use gloo::timers::callback::Timeout;
use gloo::utils::window;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{
    ClipboardEvent, DataTransferItem, DataTransferItemList, File, FileReader, HtmlElement,
    HtmlInputElement, HtmlTextAreaElement,
};
use yew::platform::spawn_local;
use yew::prelude::*;
use yewdux::Dispatch;

use i18n::{en_us, zh_cn, LanguageType};
use icons::{CloseIcon, ImageIcon};
use icons::{FileIcon, PhoneIcon, SmileIcon, VideoIcon};
use sandcat_sdk::api;
use sandcat_sdk::db;
use sandcat_sdk::model::message::{GroupMsg, InviteMsg, InviteType, Msg, SendStatus};
use sandcat_sdk::model::RightContentType;
use sandcat_sdk::state::{MobileState, SendCallState};
use sandcat_sdk::{model::message::Message, model::ContentType, state::SendMessageState};
use utils::tr;

use super::emoji::{get_emojis, Emoji};
use crate::right::emoji::EmojiSpan;

/// 右侧发送组件
/// 总体两排组件布局
/// 第一排为表情、文件、音视频按钮
/// 第二排为输入框
pub struct Sender {
    is_warn_needed: bool,
    warn_msg: String,
    timer: Option<Timeout>,
    emoji_list: Vec<Emoji>,
    show_emoji: bool,
    sender_ref: NodeRef,
    input_ref: NodeRef,
    file_input_ref: NodeRef,
    emoji_wrapper_ref: NodeRef,
    show_file_sender: bool,
    i18n: FluentBundle<FluentResource>,
    file_list: Vec<FileListItem>,
    is_mobile: bool,
}

const INPUT_MAX_LEN: usize = 5000;
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
    OnEnterKeyDown(KeyboardEvent),
    OnPaste(Event),
    CloseFileSender,
    DeleteFileInFileSender(String),
    SendVideoCall,
    SendAudioCall,
    OnTextInput,
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

impl Sender {
    // fixme need to wait message store success
    fn store_message(&self, ctx: &Context<Self>, mut msg: Message) {
        let conv_type = ctx.props().conv_type.clone();
        spawn_local(async move {
            match conv_type {
                RightContentType::Friend => {
                    db::db_ins().messages.add_message(&mut msg).await.unwrap();
                }
                RightContentType::Group => {
                    db::db_ins().group_msgs.put(&msg).await.unwrap();
                }
                _ => {}
            }
        });
    }

    fn send_msg(&self, ctx: &Context<Self>, msg: Message) {
        log::debug!("send message state in sender");
        match ctx.props().conv_type {
            RightContentType::Friend => {
                Dispatch::<SendMessageState>::global().reduce_mut(|s| s.msg = Msg::Single(msg));
            }
            RightContentType::Group => {
                Dispatch::<SendMessageState>::global()
                    .reduce_mut(|s| s.msg = Msg::Group(GroupMsg::Message(msg)));
            }
            _ => {}
        }
    }
    fn send_file(&self, ctx: &Context<Self>, file: File) {
        let mut content_type = ContentType::File;

        ctx.link().send_future(async move {
            let file_name_src = file.name();
            let file_name = api::file()
                .upload_file(file.clone())
                .await
                .map_err(|err| log::error!("上传文件错误: {:?}", err))
                .unwrap();
            // let file_name = file.name();
            let mut file_content = JsValue::default();
            // 判断文件类型

            // 判断是否是视频类型
            if file.type_() == "video/mp4" {
                content_type = ContentType::Video;
            }
            if file.type_() == "image/png" || file.type_() == "image/jpeg" {
                content_type = ContentType::Image;
                // 读取文件内容
                let file_reader = FileReader::new().expect("create file reader error");
                // 声明一个channel用来获取闭包中的数据
                let (tx, rx) = oneshot::channel();
                let mut tx = Some(tx);
                let reader = file_reader.clone();
                let onload = Closure::wrap(Box::new(move || {
                    tx.take()
                        .unwrap()
                        .send(reader.result().expect("获取文件内容错误"))
                        .expect("文件内容发送失败");
                }) as Box<dyn FnMut()>);
                file_reader.read_as_data_url(&file).expect("文件读取错误");

                file_reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                onload.forget();
                file_content = rx.await.expect("获取文件内容错误");
            }
            let file_name = format!("{}||{}", file_name, file_name_src);
            SenderMsg::FileOnload(file_name, content_type, file_content)
        });
    }
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
            emoji_list: get_emojis(),
            show_emoji: false,
            input_ref: NodeRef::default(),
            file_input_ref: NodeRef::default(),
            sender_ref: NodeRef::default(),
            emoji_wrapper_ref: NodeRef::default(),
            show_file_sender: false,
            i18n,
            file_list: vec![],
            is_mobile: Dispatch::<MobileState>::global().get().is_mobile(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SenderMsg::SendText => {
                let input = self.input_ref.cast::<HtmlTextAreaElement>().unwrap();
                let content: AttrValue = input.value().into();
                // 如果为空那么 提示不能发送空消息
                if content.is_empty() {
                    self.is_warn_needed = true;
                    self.warn_msg.clone_from(&"no_empty".to_string());
                    // 输入框立即获取焦点
                    input.focus().unwrap();
                    // 给提示框添加一个定时器，1s后消失
                    let ctx = ctx.link().clone();
                    self.timer = Some(Timeout::new(1000, move || {
                        ctx.send_message(SenderMsg::CleanEmptyMsgWarn);
                    }));
                    return true;
                }

                if content.chars().count() > INPUT_MAX_LEN {
                    self.is_warn_needed = true;
                    self.warn_msg = "input_max_len".to_string();
                    // 输入框立即获取焦点
                    input.focus().unwrap();
                    // 给提示框添加一个定时器，1s后消失
                    let ctx = ctx.link().clone();
                    self.timer = Some(Timeout::new(1000, move || {
                        ctx.send_message(SenderMsg::CleanEmptyMsgWarn);
                    }));
                    return true;
                }
                // 存储消息、发送消息
                let friend_id = ctx.props().friend_id.clone();
                let send_time = chrono::Local::now().timestamp_millis();

                let send_id = ctx.props().cur_user_id.clone();
                let msg = Message {
                    id: 0,
                    seq: 0,
                    local_id: nanoid::nanoid!().into(),
                    server_id: AttrValue::default(),
                    send_id,
                    friend_id,
                    content_type: ContentType::Text,
                    content: content.clone(),
                    create_time: send_time,
                    is_read: 1,
                    is_self: true,
                    send_time: 0,
                    send_status: SendStatus::Sending,
                    file_content: AttrValue::default(),
                };
                self.store_message(ctx, msg.clone());
                self.send_msg(ctx, msg);
                // 清空输入框
                input.set_value("");
                true
            }
            SenderMsg::CleanEmptyMsgWarn => {
                self.is_warn_needed = false;
                self.timer = None;
                true
            }
            SenderMsg::SendEmoji(emoji) => {
                // 存储消息、发送消息
                let friend_id = ctx.props().friend_id.clone();
                let time = chrono::Utc::now().timestamp_millis();
                let send_id = ctx.props().cur_user_id.clone();
                let msg = Message {
                    id: 0,
                    seq: 0,
                    local_id: nanoid::nanoid!().into(),
                    server_id: AttrValue::default(),
                    send_id,
                    friend_id,
                    content_type: ContentType::Emoji,
                    content: emoji.url.clone(),
                    create_time: time,
                    is_read: 1,
                    is_self: true,
                    send_time: 0,
                    send_status: SendStatus::Sending,
                    file_content: AttrValue::default(),
                };
                self.store_message(ctx, msg.clone());
                self.send_msg(ctx, msg);
                true
            }
            SenderMsg::ShowEmoji => {
                self.show_emoji = !self.show_emoji;
                true
            }
            SenderMsg::SendFileIconClicked => {
                let file_input = self.file_input_ref.cast::<HtmlElement>().unwrap();
                file_input.click();
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
                    id: 0,
                    seq: 0,
                    local_id: nanoid::nanoid!().into(),
                    server_id: AttrValue::default(),
                    content: file_name.clone().into(),
                    is_self: true,
                    is_read: 0,
                    create_time: time,
                    friend_id: ctx.props().friend_id.clone(),
                    send_id: ctx.props().cur_user_id.clone(),
                    content_type,
                    file_content,
                    send_time: 0,
                    send_status: SendStatus::Sending,
                };

                self.store_message(ctx, msg.clone());
                self.send_msg(ctx, msg);
                true
            }
            SenderMsg::OnEnterKeyDown(event) => {
                if event.shift_key() && event.key() == "Enter" {
                    event.prevent_default();
                    let textarea: HtmlTextAreaElement = self.input_ref.cast().unwrap();
                    let mut value = textarea.value();
                    let char_count = value.chars().count();

                    let start = textarea.selection_start().unwrap().unwrap() as usize;
                    let end = textarea.selection_end().unwrap().unwrap() as usize;

                    // 保护性检查以确保start和end不越界
                    if start > char_count || end > char_count || start > end {
                        return false; // 越界，直接返回
                    }

                    let v: Vec<(usize, char)> = value.char_indices().collect();
                    let start_index = v.get(start).map_or(start, |&(i, _)| i);

                    match end.cmp(&char_count) {
                        Ordering::Equal => value.push('\n'),
                        Ordering::Less => {
                            let end_index = v.get(end).map_or(end, |&(i, _)| i);
                            if end_index == start_index {
                                value.insert(start_index, '\n');
                            } else {
                                let selected_text = &value[start_index..end_index];
                                let new_text = "\n";
                                value = value.replacen(selected_text, new_text, 1);
                            }
                        }
                        Ordering::Greater => {}
                    };

                    textarea.set_value(&value);
                    textarea
                        .set_selection_start(Some((start + 1) as u32))
                        .unwrap();
                    textarea
                        .set_selection_end(Some((start + 1) as u32))
                        .unwrap();
                    textarea.set_scroll_top(textarea.scroll_height());
                    return false;
                }
                if event.key() == "Enter" {
                    event.prevent_default();

                    ctx.link().send_message(SenderMsg::SendText);
                }
                false
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
                    }
                });
                false
            }
            SenderMsg::OnTextInput => {
                let textarea: HtmlTextAreaElement = self.input_ref.cast().unwrap();
                // textarea.style().set_property("height", "auto").unwrap();
                let document = window().document().unwrap();
                let html = document
                    .document_element()
                    .unwrap()
                    .dyn_into::<HtmlElement>()
                    .unwrap();
                let html_style = window().get_computed_style(&html).unwrap().unwrap();
                // let html_style = html.style();

                let font_size = html_style
                    .get_property_value("font-size")
                    .unwrap_or_default();
                log::debug!("font-size: {}", font_size);
                let font_size = font_size
                    .trim_end_matches("px")
                    .parse::<f64>()
                    .unwrap_or(16.0); // 默认字体大小为16px

                // 根据实际使用情况调整min_height和max_height
                let min_height = 1.0 * font_size; // 假设最小高度为1rem
                let max_height = 5.0 * font_size; // 假设最大高度为5rem

                textarea.style().set_property("height", "auto").unwrap(); // 重置高度以获得准确的scrollHeight

                let scroll_height = textarea.scroll_height() as f64;
                if scroll_height > max_height {
                    textarea
                        .style()
                        .set_property("height", &format!("{}px", max_height))
                        .unwrap();
                    textarea.style().set_property("overflow-y", "auto").unwrap();
                } else {
                    textarea
                        .style()
                        .set_property("height", &format!("{}px", scroll_height.max(min_height)))
                        .unwrap();
                    textarea
                        .style()
                        .set_property("overflow-y", "hidden")
                        .unwrap();
                }

                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let (sender_class, emoji_class, input_class, warn_class, send_btn) =
            match *Dispatch::<MobileState>::global().get() {
                MobileState::Desktop => (
                    "sender sender-size",
                    "emoji-wrapper emoji-wrapper-size",
                    "msg-input msg-input-size",
                    "empty-msg-tip box-shadow",
                    html!(
                        <button class="send-btn"
                            onclick={ctx.link().callback(|_| SenderMsg::SendText)}>
                            {tr!(self.i18n, "send")}
                        </button>),
                ),
                MobileState::Mobile => (
                    "sender",
                    "emoji-wrapper emoji-wrapper-size-mobile",
                    "msg-input msg-input-size-mobile",
                    "empty-msg-tip-mobile box-shadow",
                    html!(),
                ),
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
        if self.show_emoji {
            let callback = &ctx.link().callback(SenderMsg::SendEmoji);
            let onblur = &ctx.link().callback(move |_| SenderMsg::ShowEmoji);
            emojis = html! {
                <div class={emoji_class} tabindex="-1" ref={self.emoji_wrapper_ref.clone()} {onblur}>
                    {
                        self.emoji_list.iter()
                        .map(|emoji| {html! (<EmojiSpan emoji={emoji.clone()} onclick={callback} />)})
                        .collect::<Html>()
                    }
                </div>
            }
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

        let textarea = if self.is_mobile {
            html! {
                <textarea class={input_class}
                    ref={self.input_ref.clone()}
                    oninput={ctx.link().callback(|_|SenderMsg::OnTextInput)}
                    {onpaste}
                    {onkeydown}>
                </textarea>
            }
        } else {
            html! {
                <textarea class={input_class}
                    ref={self.input_ref.clone()}
                    {onpaste}
                    {onkeydown}>
                </textarea>
            }
        };
        html! {
            <>
            {emojis}
            {file_sender}
            <div class={sender_class} ref={self.sender_ref.clone()}>
                // 滑块
                // <div class="sender-resizer" ref={self.resider_ref.clone()} ></div>
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
                    </div>
                    <div class="send-bar-right" >
                        {phone_call}
                    </div>
                </div>
                <div class="msg-input-wrapper">
                    {textarea}
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
            self.input_ref
                .cast::<HtmlElement>()
                .unwrap()
                .blur()
                .unwrap();
            return;
        }
        if !self.show_emoji && !ctx.props().disable && !self.is_mobile {
            self.input_ref
                .cast::<HtmlElement>()
                .unwrap()
                .focus()
                .unwrap();
        }
        if self.show_emoji {
            let wrapper = self.emoji_wrapper_ref.cast::<HtmlElement>().unwrap();
            // 设置表情面板位置
            let sender = self.sender_ref.cast::<HtmlElement>().unwrap();
            let gap = ".5rem";
            wrapper
                .style()
                .set_property(
                    "bottom",
                    format!("calc({}px + {})", sender.client_height(), gap).as_str(),
                )
                .unwrap();
            wrapper.focus().unwrap();
        }
    }
}
