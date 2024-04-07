use std::cmp::Ordering;
use std::rc::Rc;

use futures_channel::oneshot;
use gloo::timers::callback::Timeout;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    ClipboardEvent, DataTransferItem, DataTransferItemList, File, FileReader, HtmlElement,
    HtmlInputElement, HtmlTextAreaElement, Response,
};
use yew::platform::spawn_local;
use yew::prelude::*;

use crate::db;
use crate::icons::{CloseIcon, ImageIcon};
use crate::model::message::{GroupMsg, InviteMsg, InviteType, Msg};
use crate::model::RightContentType;
use crate::{
    components::right::emoji::EmojiSpan,
    icons::{FileIcon, PhoneIcon, SmileIcon, VideoIcon},
    model::message::Message,
    model::ContentType,
    pages::RecSendMessageState,
};

use super::emoji::{get_emojis, Emoji};

/// 右侧发送组件
/// 总体两排组件布局
/// 第一排为表情、文件、音视频按钮
/// 第二排为输入框
pub struct Sender {
    is_empty_warn_needed: bool,
    timer: Option<Timeout>,
    emoji_list: Vec<Emoji>,
    show_emoji: bool,
    sender_ref: NodeRef,
    input_ref: NodeRef,
    file_input_ref: NodeRef,
    emoji_wrapper_ref: NodeRef,
    show_file_sender: bool,
    rec_send_msg: Rc<RecSendMessageState>,
    _conv_listener: ContextHandle<Rc<RecSendMessageState>>,
    call_state: Rc<RecSendMessageState>,
    _call_listener: ContextHandle<Rc<RecSendMessageState>>,
    file_list: Vec<FileListItem>,
}

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
    None,
    FileOnload(String, ContentType, JsValue),
    OnEnterKeyDown(KeyboardEvent),
    OnPaste(Event),
    CloseFileSender,
    DeleteFileInFileSender(String),
    SendVideoCall,
    SendAudioCall,
}

#[derive(Properties, PartialEq, Debug)]
pub struct SenderProps {
    pub friend_id: AttrValue,
    pub conv_type: RightContentType,
    pub cur_user_id: AttrValue,
    pub disable: bool,
    pub on_file_send: Callback<Message>,
}

impl Sender {
    // fixme need to wait message store success
    fn store_message(&self, ctx: &Context<Self>, mut msg: Message) {
        let conv_type = ctx.props().conv_type.clone();
        spawn_local(async move {
            match conv_type {
                RightContentType::Friend => {
                    db::messages().await.add_message(&mut msg).await.unwrap();
                }
                RightContentType::Group => {
                    db::group_msgs().await.put(&msg).await.unwrap();
                }
                _ => {}
            }
        });
    }

    fn send_msg(&self, ctx: &Context<Self>, msg: Message) {
        match ctx.props().conv_type {
            RightContentType::Friend => {
                self.rec_send_msg.send_msg_event.emit(Msg::Single(msg));
            }
            RightContentType::Group => {
                self.rec_send_msg
                    .send_msg_event
                    .emit(Msg::Group(GroupMsg::Message(msg)));
            }
            _ => {}
        }
    }
    fn send_file(&self, ctx: &Context<Self>, file: File) {
        let mut content_type = ContentType::File;

        ctx.link().send_future(async move {
            let file_name = upload_file(file.clone())
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
            SenderMsg::FileOnload(file_name, content_type, file_content)
        });
    }
}

async fn upload_file(file: File) -> Result<String, JsValue> {
    use web_sys::FormData;

    let form = FormData::new().unwrap();
    form.append_with_blob("file", &file).unwrap();

    // 创建请求体
    let mut opts = web_sys::RequestInit::new();
    opts.method("POST");
    opts.body(Some(&form));

    // 创建请求
    let url = "/api/file/upload";
    let request = web_sys::Request::new_with_str_and_init(url, &opts).unwrap();

    // 发送网络请求
    let window = web_sys::window().unwrap();
    let request_promise = window.fetch_with_request(&request);
    let res: Response = JsFuture::from(request_promise).await?.dyn_into()?;
    let text = JsFuture::from(res.text().unwrap()).await.unwrap();

    Ok(text.as_string().unwrap())
}

impl Component for Sender {
    type Message = SenderMsg;

    type Properties = SenderProps;

    fn create(ctx: &Context<Self>) -> Self {
        let (conv_state, _conv_listener) = ctx
            .link()
            .context(ctx.link().callback(|_| SenderMsg::None))
            .expect("needed to get context");
        let (call_state, _call_listener) = ctx
            .link()
            .context(ctx.link().callback(|_| SenderMsg::None))
            .expect("needed to get context");

        // 加载表情
        Self {
            is_empty_warn_needed: false,
            timer: None,
            emoji_list: get_emojis(),
            show_emoji: false,
            input_ref: NodeRef::default(),
            file_input_ref: NodeRef::default(),
            sender_ref: NodeRef::default(),
            emoji_wrapper_ref: NodeRef::default(),
            show_file_sender: false,
            rec_send_msg: conv_state,
            _conv_listener,
            call_state,
            _call_listener,
            file_list: vec![],
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SenderMsg::SendText => {
                let input = self.input_ref.cast::<HtmlTextAreaElement>().unwrap();
                let content: AttrValue = input.value().into();
                // 如果为空那么 提示不能发送空消息
                if content.is_empty() {
                    self.is_empty_warn_needed = true;
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
                    local_id: nanoid::nanoid!().into(),
                    server_id: AttrValue::default(),
                    send_id,
                    friend_id,
                    content_type: ContentType::Text,
                    content: content.clone(),
                    create_time: send_time,
                    is_read: true,
                    is_self: true,
                    file_content: AttrValue::default(),
                };
                self.store_message(ctx, msg.clone());
                self.send_msg(ctx, msg);
                // 清空输入框
                input.set_value("");
                true
            }
            SenderMsg::CleanEmptyMsgWarn => {
                self.is_empty_warn_needed = false;
                self.timer = None;
                true
            }
            SenderMsg::SendEmoji(emoji) => {
                // 存储消息、发送消息
                let friend_id = ctx.props().friend_id.clone();
                let time = chrono::Local::now().timestamp_millis();
                let send_id = ctx.props().cur_user_id.clone();
                let msg = Message {
                    id: 0,
                    local_id: nanoid::nanoid!().into(),
                    server_id: AttrValue::default(),
                    send_id,
                    friend_id,
                    content_type: ContentType::Emoji,
                    content: emoji.url.clone(),
                    create_time: time,
                    is_read: true,
                    is_self: true,
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
                let time = chrono::Local::now().timestamp_millis();
                let file_content = if let Some(file_content) = file_content.as_string() {
                    file_content.into()
                } else {
                    "".into()
                };

                let msg = Message {
                    id: 0,
                    local_id: nanoid::nanoid!().into(),
                    server_id: AttrValue::default(),
                    content: file_name.clone().into(),
                    is_self: true,
                    is_read: false,
                    create_time: time,
                    friend_id: ctx.props().friend_id.clone(),
                    send_id: ctx.props().cur_user_id.clone(),
                    content_type,
                    file_content,
                };

                self.store_message(ctx, msg.clone());
                self.send_msg(ctx, msg);
                true
            }
            SenderMsg::None => false,
            SenderMsg::OnEnterKeyDown(event) => {
                if event.shift_key() {
                    if event.key() == "Enter" {
                        event.prevent_default();
                        // log::debug!("press key is :{:?}", event.key());
                        let textarea: HtmlTextAreaElement = self.input_ref.cast().unwrap();
                        let start = textarea.selection_start().unwrap().unwrap() as usize;
                        let end = textarea.selection_end().unwrap().unwrap() as usize;
                        let mut value = textarea.value();
                        let v: Vec<(usize, char)> = value.char_indices().collect();
                        let start_index = v[start].0;
                        // log::debug!("v: {:?}; start: {}, end: {}", &v, start, end);
                        match end.cmp(&value.chars().count()) {
                            Ordering::Equal => value.push('\n'),
                            Ordering::Less => {
                                let end_index = v[end].0;
                                // log::debug!("end index: {}",end_index);
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
                    }
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
                            // 其他文件
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
                self.call_state.call_event.emit(InviteMsg {
                    msg_id: nanoid::nanoid!().into(),
                    create_time: chrono::Local::now().timestamp_millis(),
                    friend_id: ctx.props().friend_id.clone(),
                    send_id: ctx.props().cur_user_id.clone(),
                    invite_type: InviteType::Video,
                });
                false
            }
            SenderMsg::SendAudioCall => {
                self.call_state.call_event.emit(InviteMsg {
                    msg_id: nanoid::nanoid!().into(),
                    create_time: chrono::Local::now().timestamp_millis(),
                    friend_id: ctx.props().friend_id.clone(),
                    send_id: ctx.props().cur_user_id.clone(),
                    invite_type: InviteType::Audio,
                });
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // spawn disable layer
        let mut disable = html!();
        if ctx.props().disable {
            let message = match ctx.props().conv_type {
                RightContentType::Friend => "对方开启了好友验证，请先通过验证",
                RightContentType::Group => "群聊已经解散",
                _ => "暂时无法发送消息",
            };
            disable = html! {
                <div class="sender-disabled">
                    {message}
                </div>
            }
        }
        // 绘制警告tip
        let mut warn = html!();
        if self.is_empty_warn_needed {
            warn = html! {
                <span class="empty-msg-tip">
                    {"发送内容不能为空"}
                </span>
            }
        }

        let mut emojis = html!();
        if self.show_emoji {
            let callback = &ctx.link().callback(SenderMsg::SendEmoji);
            let onblur = &ctx.link().callback(move |_| SenderMsg::ShowEmoji);
            emojis = html! {
                <div class="emoji-wrapper" tabindex="1" ref={self.emoji_wrapper_ref.clone()} {onblur}>
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
                            {"确定"}
                        </button>
                        <button {onclick} >
                            {"取消"}
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
        html! {
            <>
            {emojis}
            {file_sender}
            <div class="sender" ref={self.sender_ref.clone()}>
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
                        <span onclick={audio_click}>
                            <PhoneIcon />
                        </span>
                        <span onclick={video_click} >
                            <VideoIcon />
                        </span>
                    </div>
                </div>
                <div class="msg-input-wrapper">
                    <textarea class="msg-input" ref={self.input_ref.clone()} {onpaste} {onkeydown} /* contenteditable="true" */>
                    </textarea>
                    {warn}
                    <button class="send-btn"
                        onclick={ctx.link().callback(|_| SenderMsg::SendText)}>
                        {"发送"}
                    </button>
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
        if !self.show_emoji && !ctx.props().disable {
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
