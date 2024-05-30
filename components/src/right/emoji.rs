// 暂时先写死

use yew::prelude::*;

pub fn get_emojis() -> Vec<Emoji> {
    vec![
        Emoji {
            id: 1,
            name: "sun".into(),
            url: "/images/emoji/sun.gif".into(),
            is_inline: false,
        },
        Emoji {
            id: 2,
            name: "poop".into(),
            url: "/images/emoji/poop.gif".into(),
            is_inline: false,
        },
        Emoji {
            id: 3,
            name: "thumbup".into(),
            url: "/images/emoji/thumbup.gif".into(),
            is_inline: false,
        },
        Emoji {
            id: 4,
            name: "se".into(),
            url: "/images/emoji/se.gif".into(),
            is_inline: false,
        },
        Emoji {
            id: 5,
            name: "smile".into(),
            url: "/images/emoji/smile.gif".into(),
            is_inline: false,
        },
        Emoji {
            id: 6,
            name: "fist".into(),
            url: "/images/emoji/fist.gif".into(),
            is_inline: false,
        },
        Emoji {
            id: 7,
            name: "xieyan".into(),
            url: "/images/emoji/xieyan.gif".into(),
            is_inline: false,
        },
        Emoji {
            id: 8,
            name: "ghost".into(),
            url: "/images/emoji/ghost.gif".into(),
            is_inline: false,
        },
        Emoji {
            id: 9,
            name: "angry".into(),
            url: "/images/emoji/angry.gif".into(),
            is_inline: false,
        },
        Emoji {
            id: 10,
            name: "thumbdown".into(),
            url: "/images/emoji/thumbdown.gif".into(),
            is_inline: false,
        },
    ]
}

#[derive(Debug, Clone, PartialEq)]
pub struct Emoji {
    pub id: usize,
    pub name: String,
    pub url: String,
    pub is_inline: bool,
}

pub struct EmojiSpan {}

pub enum EmojiSpanMsg {
    SendEmoji,
}

#[derive(Properties, Clone, PartialEq)]
pub struct EmojiProps {
    pub emoji: Emoji,
    pub onclick: Callback<Emoji>,
    pub is_unicode: bool,
}

impl Component for EmojiSpan {
    type Message = EmojiSpanMsg;
    type Properties = EmojiProps;

    fn create(_ctx: &Context<Self>) -> Self {
        EmojiSpan {}
    }

    fn update(&mut self, ctx: &Context<Self>, _msg: Self::Message) -> bool {
        match _msg {
            EmojiSpanMsg::SendEmoji => {
                let emoji = ctx.props().emoji.clone();
                ctx.props().onclick.emit(emoji);
            }
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let emoji = &ctx.props().emoji;
        let onclick = ctx.link().callback(move |_| EmojiSpanMsg::SendEmoji);
        let content = if ctx.props().is_unicode {
            html!({ &emoji.url })
        } else {
            html!(<img src={emoji.url.clone()} />)
        };
        html! {
        <span class="emoji"
            {onclick}
            title={emoji.name.clone()}
            >
            {content}
        </span>}
    }
}

pub const EMOJI_UNICODE: &str = "😀😃😄😁😆😅🤣😂🙂🙃🫠😉
    😊😇😕🫤😟🙁😮😯😲😳🥺🥹😦😧😨😰😥😢😭😱😖😣
    😞😓😩😫🥱😤😡😠🤬😈👿💀💩🤡👹👺👻👽👾🤖😺😸
    😹😻😼😽🙀😿😾🙈🙉🙊💌💘💝💖💗💓💞💕😍🤩😘😗
    😚😙🥲😋😛😜🤪😝🤑🤗🤭🫢🫣🤫🤔🫡🤐🤨😐😑😶🫥😶‍🌫️
    😏😒🙄😬😮‍💨🤥🫨😔😪🤤😴😷🤒🤕🤢🤮🤧🥵🥶🥴😵😵‍💫🤯
    🤠🥳🥸😎🤓🧐";

pub fn get_unicode_emojis() -> Vec<Emoji> {
    let mut emojis = vec![];
    for c in EMOJI_UNICODE.chars() {
        if c.is_whitespace() {
            continue;
        }
        emojis.push(Emoji {
            id: emojis.len(),
            name: c.to_string(),
            url: c.to_string(),
            is_inline: true,
        });
    }
    emojis
}
