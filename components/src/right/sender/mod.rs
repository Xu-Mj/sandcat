mod component;
pub use component::*;

use std::cmp::Ordering;

use fluent::FluentBundle;
use fluent::FluentResource;
use futures_channel::oneshot;
use gloo::timers::callback::Timeout;
use gloo::utils::window;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::File;
use web_sys::FileReader;
use web_sys::HtmlElement;
use web_sys::HtmlTextAreaElement;
use yew::AttrValue;
use yew::Context;
use yew::NodeRef;
use yewdux::Dispatch;

use sandcat_sdk::api;
use sandcat_sdk::db;
use sandcat_sdk::model::message::GroupMsg;
use sandcat_sdk::model::message::Message;
use sandcat_sdk::model::message::Msg;
use sandcat_sdk::model::message::SendStatus;
use sandcat_sdk::model::ContentType;
use sandcat_sdk::model::RightContentType;
use sandcat_sdk::state::MobileState;
use sandcat_sdk::state::SendMessageState;

use super::emoji::Emoji;

const INPUT_MAX_LEN: usize = 5000;

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
    fn handle_new_line(&self, ctx: &Context<Self>) -> bool {
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

        // handle textarea height
        ctx.link().send_message(SenderMsg::OnTextInput);

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
        false
    }

    // fixme need to wait message store success
    fn store_send_msg(&self, ctx: &Context<Self>, mut msg: Message) {
        let conv_type = ctx.props().conv_type.clone();
        spawn_local(async move {
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
        });
    }

    // fn send_msg(&self, ctx: &Context<Self>, msg: Message) {
    //     log::debug!("send message state in sender");
    //     match ctx.props().conv_type {
    //         RightContentType::Friend => {
    //             Dispatch::<SendMessageState>::global().reduce_mut(|s| s.msg = Msg::Single(msg));
    //         }
    //         RightContentType::Group => {
    //             Dispatch::<SendMessageState>::global()
    //                 .reduce_mut(|s| s.msg = Msg::Group(GroupMsg::Message(msg)));
    //         }
    //         _ => {}
    //     }
    // }

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
        log::info!(
            "send text, platform: {:?}, {}",
            self.get_platform(),
            self.is_mobile
        );
        if let Some(input) = self.input_ref.cast::<HtmlTextAreaElement>() {
            let content: AttrValue = input.value().into();
            // 如果为空那么 提示不能发送空消息
            if content.is_empty() || content.trim().is_empty() {
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
                ..Default::default()
            };
            self.store_send_msg(ctx, msg);
            // self.send_msg(ctx, msg);
            // 清空输入框
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
}
