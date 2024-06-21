mod component;
mod emoji;
pub use component::*;
use log::error;

use fluent::FluentBundle;
use fluent::FluentResource;
use futures_channel::oneshot;
use gloo::timers::callback::Timeout;
use gloo::utils::window;
use utils::tr;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::File;
use web_sys::FileReader;
use web_sys::HtmlElement;
use web_sys::HtmlTextAreaElement;
use yew::html;
use yew::AttrValue;
use yew::Context;
use yew::Html;
use yew::NodeRef;
use yewdux::Dispatch;

use crate::constant::CANCEL;
use crate::constant::DISABLED;
use crate::constant::GROUP_DISMISSED;
use crate::constant::SEND;
use crate::constant::SUBMIT;
use crate::constant::VERIFY_NEEDED;
use crate::right::sender::emoji::EmojiPanel;
use icons::{CloseIcon, FileIcon, ImageIcon, PhoneIcon, VideoIcon};
use sandcat_sdk::api;
use sandcat_sdk::db;
use sandcat_sdk::model::message::GroupMsg;
use sandcat_sdk::model::message::Message;
use sandcat_sdk::model::message::Msg;
use sandcat_sdk::model::message::SendStatus;
use sandcat_sdk::model::voice::Voice;
use sandcat_sdk::model::ContentType;
use sandcat_sdk::model::RightContentType;
use sandcat_sdk::state::MobileState;
use sandcat_sdk::state::SendAudioMsgState;
use sandcat_sdk::state::SendMessageState;

use super::emoji::Emoji;

const INPUT_MAX_LEN: usize = 5000;
const NO_EMPTY_WARN: &str = "no_empty";
const INPUT_MAX_LEN_WARN: &str = "input_max_len";
pub struct FileListItem {
    file: File,
    file_type: FileType,
}

pub enum FileType {
    Image,
    File,
}

/// 右侧发送组件
/// 总体两排组件布局
/// 第一排为表情、文件、音视频按钮
/// 第二排为输入框
pub struct Sender {
    is_warn_needed: bool,
    warn_msg: String,
    timer: Option<Timeout>,
    show_emoji: bool,
    sender_ref: NodeRef,
    input_ref: NodeRef,
    file_input_ref: NodeRef,
    show_file_sender: bool,
    i18n: FluentBundle<FluentResource>,
    file_list: Vec<FileListItem>,
    is_mobile: bool,
    enter_key_down: i64,
    is_key_down: bool,
    is_voice_mode: bool,
}

impl Sender {
    fn get_platform(&self) -> i32 {
        if self.is_mobile {
            MobileState::Mobile as i32
        } else {
            MobileState::Desktop as i32
        }
    }

    fn send_emoji(&self, ctx: &Context<Self>, emoji: Emoji) -> bool {
        if emoji.is_inline {
            // insert to the textarea
            self.insert_character_before_cursor(emoji.url.clone());
            return true;
        }
        // 存储消息、发送消息
        let friend_id = ctx.props().friend_id.clone();
        let time = chrono::Utc::now().timestamp_millis();
        let send_id = ctx.props().cur_user_id.clone();
        let msg = Message {
            local_id: nanoid::nanoid!().into(),
            server_id: AttrValue::default(),
            send_id,
            friend_id,
            content_type: ContentType::Emoji,
            content: emoji.url.clone().into(),
            create_time: time,
            is_read: 1,
            is_self: true,
            platform: self.get_platform(),
            send_status: SendStatus::Sending,
            avatar: ctx.props().avatar.clone(),
            nickname: ctx.props().nickname.clone(),
            ..Default::default()
        };
        self.store_send_msg(ctx, msg);
        true
    }

    /// insert character before cursor like emoji and \n
    /// use utf16
    fn insert_character_before_cursor(&self, c: String) {
        let textarea: HtmlTextAreaElement = self.input_ref.cast().unwrap();
        let value = textarea.value();
        let mut utf16_value: Vec<u16> = value.encode_utf16().collect();

        // get the cursor position
        let start = textarea.selection_start().unwrap().unwrap() as usize;
        let end = textarea.selection_end().unwrap().unwrap() as usize;

        let emoji_utf16: Vec<u16> = c.encode_utf16().collect();
        let emoji_utf16_len = emoji_utf16.len();

        // insert new character
        utf16_value.splice(start..end, emoji_utf16);

        // convert back to string
        let new_value = String::from_utf16(&utf16_value).unwrap();

        textarea.set_value(&new_value);

        // update cursor position
        let new_cursor_position = if &c == "\n" {
            start + 1
        } else {
            start + emoji_utf16_len
        };

        textarea
            .set_selection_start(Some(new_cursor_position as u32))
            .unwrap();
        textarea
            .set_selection_end(Some(new_cursor_position as u32))
            .unwrap();
    }

    fn handle_new_line(&self, ctx: &Context<Self>) {
        let textarea: HtmlTextAreaElement = self.input_ref.cast().unwrap();
        self.insert_character_before_cursor("\n".to_string());
        // handle textarea height
        if self.is_mobile {
            ctx.link().send_message(SenderMsg::OnTextInput);
        }

        // scroll to bottom
        let style = window().get_computed_style(&textarea).unwrap().unwrap();
        let padding_bottom = style
            .get_property_value("padding-bottom")
            .unwrap_or_default();
        let padding_bottom = padding_bottom
            .trim_end_matches("px")
            .parse::<i32>()
            .unwrap_or(8);

        textarea.set_scroll_top(textarea.scroll_height() + padding_bottom);
    }

    fn store_send_msg(&self, ctx: &Context<Self>, msg: Message) {
        let conv_type = ctx.props().conv_type.clone();
        spawn_local(async move {
            Self::store_send_msg_(conv_type, msg).await;
        });
    }

    async fn store_send_msg_(conv_type: RightContentType, mut msg: Message) {
        match conv_type {
            RightContentType::Friend => {
                db::db_ins().messages.add_message(&mut msg).await.unwrap();
                Dispatch::<SendMessageState>::global().reduce_mut(|s| s.msg = Msg::Single(msg));
            }
            RightContentType::Group => {
                db::db_ins().group_msgs.put(&msg).await.unwrap();
                Dispatch::<SendMessageState>::global()
                    .reduce_mut(|s| s.msg = Msg::Group(GroupMsg::Message(msg)));
            }
            _ => {}
        }
    }

    fn get_file_sender_html(&self, ctx: &Context<Self>) -> Html {
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
                        {tr!(self.i18n, SUBMIT)}
                    </button>
                    <button {onclick} >
                        {tr!(self.i18n, CANCEL)}
                    </button>
                </div>
            </div>
        }
    }

    // todo upload file by behind task and update the upload state
    fn send_file(&self, ctx: &Context<Self>, file: File) {
        let mut content_type = ContentType::File;

        ctx.link().send_future(async move {
            let file_name_src = file.name();
            let file_name = api::file()
                .upload_file(&file)
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

    fn send_text(&mut self, ctx: &Context<Self>) -> bool {
        if let Some(input) = self.input_ref.cast::<HtmlTextAreaElement>() {
            let value = input.value();
            let content: AttrValue = value.trim().to_string().into();
            // 如果为空那么 提示不能发送空消息
            if content.is_empty() {
                self.is_warn_needed = true;
                self.warn_msg = NO_EMPTY_WARN.to_string();
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
                self.warn_msg = INPUT_MAX_LEN_WARN.to_string();
                // get focus
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
            let platform = self.get_platform();
            let msg = Message {
                local_id: nanoid::nanoid!().into(),
                server_id: AttrValue::default(),
                send_id,
                friend_id,
                content_type: ContentType::Text,
                content: content.clone(),
                create_time: send_time,
                is_read: 1,
                is_self: true,
                platform,
                send_status: SendStatus::Sending,
                avatar: ctx.props().avatar.clone(),
                nickname: ctx.props().nickname.clone(),
                ..Default::default()
            };
            self.store_send_msg(ctx, msg);
            // clean the input
            input.set_value("");
            if self.is_mobile {
                // reset the mobile textarea height
                input.style().set_property("height", "auto").unwrap();
            }
        }
        true
    }

    fn handle_input(&self) -> bool {
        let textarea: HtmlTextAreaElement = self.input_ref.cast().unwrap();
        let document = window().document().unwrap();
        let html = document
            .document_element()
            .unwrap()
            .dyn_into::<HtmlElement>()
            .unwrap();
        let html_style = window().get_computed_style(&html).unwrap().unwrap();

        let font_size = html_style
            .get_property_value("font-size")
            .unwrap_or_default();
        let font_size = font_size
            .trim_end_matches("px")
            .parse::<f64>()
            .unwrap_or(16.0);

        let min_height = 1.0 * font_size;
        let max_height = 7.0 * font_size;

        // 重置高度以获得准确的scrollHeight
        textarea.style().set_property("height", "auto").unwrap();

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

    fn send_voice_msg(&self, ctx: &Context<Self>, voice: Voice) {
        let conv_type = ctx.props().conv_type.clone();
        let friend_id = ctx.props().friend_id.clone();
        let avatar = ctx.props().avatar.clone();
        let nickname = ctx.props().nickname.clone();
        let user_id = ctx.props().cur_user_id.clone();
        let platform = self.get_platform();
        spawn_local(async move {
            if let Err(e) = db::db_ins().voices.save(&voice).await {
                error!("save voice error:{:?}", e);
            }

            let time = chrono::Utc::now().timestamp_millis();
            let mut msg = Message {
                local_id: voice.local_id.into(),
                is_self: true,
                create_time: time,
                friend_id,
                send_id: user_id,
                is_read: 1,
                content_type: ContentType::Audio,
                platform,
                send_status: SendStatus::Sending,
                audio_duration: voice.duration,
                audio_downloaded: true,
                avatar,
                nickname,
                ..Default::default()
            };
            Dispatch::<SendAudioMsgState>::global().set(SendAudioMsgState { msg: msg.clone() });

            // send to file server
            let name = match api::file().upload_voice(&voice.data).await {
                Ok(name) => name,
                Err(e) => {
                    error!("send to file server error:{:?}", e);
                    return;
                }
            };
            msg.content = name.into();
            // send to message server
            Self::store_send_msg_(conv_type, msg).await;
        });
    }

    fn get_sender_classes(
        &self,
        ctx: &Context<Self>,
        is_mobile: bool,
    ) -> (&'static str, &'static str, &'static str, Html) {
        if is_mobile {
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
                    <button class="send-btn" onclick={ctx.link().callback(|_| SenderMsg::SendText)}>
                        {tr!(self.i18n, SEND)}
                    </button>
                ),
            )
        }
    }

    fn get_disable_html(&self, ctx: &Context<Self>) -> Html {
        if ctx.props().disable {
            let message = match ctx.props().conv_type {
                RightContentType::Friend => tr!(self.i18n, VERIFY_NEEDED),
                RightContentType::Group => tr!(self.i18n, GROUP_DISMISSED),
                _ => tr!(self.i18n, DISABLED),
            };
            html!(<div class="sender-disabled">{message}</div>)
        } else {
            html!()
        }
    }

    fn get_phone_call_icons(&self, ctx: &Context<Self>, conv_type: &RightContentType) -> Html {
        if *conv_type == RightContentType::Friend {
            html!(
                <>
                    <span onclick={ctx.link().callback(|_| SenderMsg::SendAudioCall)}>
                        <PhoneIcon />
                    </span>
                    <span onclick={ctx.link().callback(|_| SenderMsg::SendVideoCall)}>
                        <VideoIcon />
                    </span>
                </>
            )
        } else {
            html!()
        }
    }

    // Extract the emoji panel logic
    fn get_emoji_panel(&self, ctx: &Context<Self>) -> Html {
        if self.show_emoji {
            let callback = ctx.link().callback(SenderMsg::SendEmoji);
            let onblur = ctx.link().callback(|_| SenderMsg::ShowEmoji);
            html!(<EmojiPanel send={callback} close={onblur}/>)
        } else {
            html!()
        }
    }
}
