use log::error;
use sandcat_sdk::{
    db,
    model::{message::Message, ContentType},
};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlElement;
use yew::prelude::*;

pub struct RelatedMsg {
    text_node: NodeRef,
    show_img_preview: bool,
    msg: Option<Message>,
    show_complete_text: bool,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct Props {
    pub local_id: AttrValue,
    pub nickname: AttrValue,
}

pub enum Msg {
    PreviewImg,
    ShowRelated(Message),
    ShowText,
}

impl Component for RelatedMsg {
    type Message = Msg;

    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let local_id = ctx.props().local_id.clone();
        let ctx = ctx.link().clone();
        spawn_local(async move {
            if let Ok(Some(msg)) = db::db_ins().messages.get_msg_by_local_id(&local_id).await {
                ctx.send_message(Self::Message::ShowRelated(msg));
            } else {
                error!("related msg not found");
            }
        });

        Self {
            text_node: NodeRef::default(),
            show_img_preview: false,
            msg: None,
            show_complete_text: false,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::PreviewImg => {
                self.show_img_preview = !self.show_img_preview;
            }
            Msg::ShowRelated(msg) => {
                self.msg = Some(msg);
            }
            Msg::ShowText => {
                self.show_complete_text = !self.show_complete_text;
            }
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if self.msg.is_none() {
            return html! {};
        }
        let msg = self.msg.as_ref().unwrap();
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
                let mut preview_text = html!();
                if self.show_complete_text {
                    preview_text = html! {
                        <div  class="preview-text box-shadow"
                            tabindex="0"
                            ref={self.text_node.clone()}
                            onblur={ctx.link().callback(|_|Self::Message::ShowText)}>
                            {html_content.clone()}
                        </div>
                    }
                }
                let onclick = ctx.link().callback(|_| Self::Message::ShowText);
                html! {
                <>
                    {preview_text}
                    <div class="related-text-msg" {onclick}>
                        {format!("{}: ",ctx.props().nickname)}
                        {html_content}
                    </div>
                </>
                }
            }
            ContentType::Image => {
                let img_url = if msg.file_content.is_empty() {
                    let full_original = &msg.content;
                    let file_name_prefix =
                        full_original.split("||").next().unwrap_or(full_original);
                    AttrValue::from(format!("/api/file/get/{}", file_name_prefix))
                } else {
                    msg.file_content.clone()
                };
                let src = img_url.clone();
                let onclick = ctx
                    .link()
                    .callback(move |_: MouseEvent| Self::Message::PreviewImg);
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
                    <div class="img-mask">
                    </div>
                    {format!("{}: ",ctx.props().nickname)}
                    <img class="msg-item-img" src={img_url} {onclick}/>
                </>
                }
            }
            ContentType::Video => html! {
                <video class="msg-item-video">
                    <source src={&msg.content} type="video/mp4" />
                </video>
            },
            ContentType::File => {
                let full_original = msg.content.clone();
                let mut parts = full_original.split("||");
                let file_name_prefix = parts.next().unwrap_or(&full_original).to_string();
                let file_name = parts.next().unwrap_or(&full_original).to_string();

                html! {
                    <a href={file_name_prefix} download="" class="msg-item-file-name">
                        {file_name}
                    </a>
                }
            }
            ContentType::Emoji => {
                html! {
                    <img class="emoji" src={msg.content.clone()} />
                }
            }
            _ => html!(),
        };
        html! {
            <div class="related-msg related-msg-background pointer">
                {content}
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if let Some(node) = self.text_node.cast::<HtmlElement>() {
            let _ = node.focus();
        }
    }
}
