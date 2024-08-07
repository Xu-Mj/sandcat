use indexmap::IndexMap;
use web_sys::HtmlDivElement;
use yew::prelude::*;

use icons::{BiggerIcon, SmileIcon};
use sandcat_sdk::state::MobileState;

use crate::right::emoji::{get_emojis, get_unicode_emojis, Emoji, EmojiSpan};
pub struct EmojiPanel {
    node: NodeRef,
    data: IndexMap<String, EmojiType>,
    current_type: String,
    is_mobile: bool,
}

#[derive(Clone, PartialEq, Properties)]
pub struct EmojiPanelProps {
    pub send: Callback<Emoji>,
    pub close: Callback<()>,
}

pub enum EmojiPanelMsg {
    Send(Emoji),
    ChangeEmojiType(String),
    Close,
}

const BIGGER_EMOJI: &str = "bigger_emoji";
const UNICODE_EMOJI: &str = "unicode_emoji";

impl Component for EmojiPanel {
    type Message = EmojiPanelMsg;

    type Properties = EmojiPanelProps;

    fn create(_ctx: &Context<Self>) -> Self {
        // load emoji data
        let current_type = EmojiType::new(
            UNICODE_EMOJI.to_owned(),
            UNICODE_EMOJI.to_owned(),
            false,
            true,
            None,
            get_unicode_emojis(),
        );

        let bigger = EmojiType::new(
            BIGGER_EMOJI.to_owned(),
            BIGGER_EMOJI.to_owned(),
            false,
            false,
            None,
            get_emojis(),
        );

        let data = IndexMap::from([
            (BIGGER_EMOJI.to_owned(), bigger),
            (UNICODE_EMOJI.to_owned(), current_type.clone()),
        ]);
        // todo add user emoji

        let is_mobile = MobileState::is_mobile();
        Self {
            node: NodeRef::default(),
            data,
            current_type: UNICODE_EMOJI.to_owned(),
            is_mobile,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            EmojiPanelMsg::Send(emoji) => {
                ctx.props().send.emit(emoji);
            }
            EmojiPanelMsg::ChangeEmojiType(t) => {
                self.current_type = t;
                return true;
            }
            EmojiPanelMsg::Close => {
                ctx.props().close.emit(());
            }
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // up panel --> emoji
        // down panel --> emoji type
        let send = &ctx.link().callback(EmojiPanelMsg::Send);
        let mut up = html!();
        if let Some(emoji_type) = self.data.get(&self.current_type) {
            up = emoji_type
                .emojis
                .iter()
                .map(|emoji| html! (<EmojiSpan emoji={emoji.clone()} onclick={send} is_unicode={emoji_type.is_inline}/>))
                .collect::<Html>();
        }

        let mut up_class = classes!("emoji-panel-up");
        if self.current_type == UNICODE_EMOJI {
            up_class.push("unicode-emoji-wrapper");
        } else {
            up_class.push("emoji-wrapper");
        }

        let class = if self.is_mobile {
            "emoji-panel emoji-panel-size-mobile"
        } else {
            "emoji-panel emoji-panel-size"
        };
        let on_unicode_click = ctx
            .link()
            .callback(|_| EmojiPanelMsg::ChangeEmojiType(UNICODE_EMOJI.to_owned()));
        let on_bigger_click = ctx
            .link()
            .callback(|_| EmojiPanelMsg::ChangeEmojiType(BIGGER_EMOJI.to_owned()));
        let onblur = ctx.link().callback(|_| EmojiPanelMsg::Close);
        html! {
            <div tabindex="0" ref={self.node.clone()}  {class} {onblur}>
                <div class={up_class}>
                    {up}
                </div>
                <div class="emoji-panel-footer">
                    <span onclick={on_unicode_click}><SmileIcon /> </span>
                    <span onclick={on_bigger_click}><BiggerIcon /></span>
                    // todo render user emoji
                </div>
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if !first_render {
            return;
        }
        if let Some(node) = self.node.cast::<HtmlDivElement>() {
            // let onclose = ctx.props().close.clone();
            // let node = node.clone();
            // // register click event to document
            // let func = Closure::wrap(Box::new(move |event: MouseEvent| {
            //     if let Some(target) = event.target() {
            //         let target_node = target.dyn_into::<web_sys::Node>().unwrap();
            //         if !node.contains(Some(&target_node)) {
            //             onclose.emit(());
            //             // remove onclick event
            //             document().set_onclick(None);
            //         }
            //     }
            // }) as Box<dyn FnMut(MouseEvent)>);
            // document().set_onclick(Some(func.as_ref().unchecked_ref()));
            // self.click_closure = Some(func);
            node.focus().unwrap();
        }
    }
}

#[allow(dead_code)]
#[derive(Clone)]
struct EmojiType {
    pub id: String,
    pub name: String,
    pub is_user: bool,
    pub is_inline: bool,
    pub emoji_url: Option<String>,
    pub emojis: Vec<Emoji>,
}

impl EmojiType {
    pub fn new(
        id: String,
        name: String,
        is_user: bool,
        is_inline: bool,
        emoji_url: Option<String>,
        emojis: Vec<Emoji>,
    ) -> Self {
        Self {
            id,
            name,
            is_user,
            is_inline,
            emoji_url,
            emojis,
        }
    }
}
