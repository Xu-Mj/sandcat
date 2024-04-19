use fluent::{FluentBundle, FluentResource};
use gloo::utils::window;
use web_sys::HtmlDivElement;
use yew::prelude::*;

use crate::{
    components::action::Action,
    i18n::{en_us, zh_cn, LanguageType},
    model::{user::UserWithMatchType, RightContentType},
    tr, utils,
};

#[derive(Default)]
pub struct FriendCard {
    friend: UserWithMatchType,
    node_ref: NodeRef,
    i18n: FluentBundle<FluentResource>,
}

pub enum FriendCardMsg {
    Destroy,
}

#[derive(Properties, Clone, PartialEq)]
pub struct FriendCardProps {
    // container: Element,
    pub friend_info: UserWithMatchType,
    pub user_id: AttrValue,
    pub lang: LanguageType,
    pub close: Callback<()>,
    pub is_self: bool,
    pub x: i32,
    pub y: i32,
}

impl Component for FriendCard {
    type Message = FriendCardMsg;

    type Properties = FriendCardProps;

    fn create(ctx: &Context<Self>) -> Self {
        let friend = ctx.props().friend_info.clone();
        let res = match ctx.props().lang {
            LanguageType::ZhCN => zh_cn::FRIEND_CARD,
            LanguageType::EnUS => en_us::FRIEND_CARD,
        };
        let i18n = utils::create_bundle(res);
        Self {
            i18n,
            friend,
            ..Default::default()
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            FriendCardMsg::Destroy => {
                ctx.props().close.emit(());
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div
                class="friend-card box-shadow"
                tabindex="-1"
                ref={self.node_ref.clone()}
                onblur={ctx.link().callback(|_| FriendCardMsg::Destroy)}
                >
                <div class="friend-card-header">
                    <img src={&self.friend.avatar} class="friend-card-avatar"/>
                    <div class="friend-card-info">
                        // <span><b>{&self.friend.remark}</b></span>
                        <span>{tr!(self.i18n, "nickname")}{&self.friend.name}</span>
                        <span>{tr!(self.i18n, "account")}{&self.friend.account}</span>
                        <span>{tr!(self.i18n, "region")}{&self.friend.region.clone().unwrap_or_default()} </span>
                    </div>
                </div>
                <div class="friend-card-body">
                    <Action id={&ctx.props().friend_info.id} conv_type={RightContentType::Friend} lang={ctx.props().lang}/>
                </div>
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        let node = self.node_ref.cast::<HtmlDivElement>().unwrap();
        if first_render {
            // calculate border boundary
            let height = window().inner_height().unwrap().as_f64().unwrap() as i32;
            let width = window().inner_width().unwrap().as_f64().unwrap() as i32;
            let mut x = ctx.props().x;
            let mut y = ctx.props().y;
            if node.client_height() > height - y {
                y = height - node.client_height();
            }
            if node.client_width() > width - x {
                x = width - node.client_width();
            }

            log::debug!("x: {}, y: {}", x, y);
            node.style()
                .set_property("top", format!("{}px", y).as_str())
                .unwrap();
            node.style()
                .set_property("left", format!("{}px", x).as_str())
                .unwrap();
            node.focus().unwrap();
        }
    }
}
