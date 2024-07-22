use fluent::{FluentBundle, FluentResource};
use gloo::utils::window;
use web_sys::HtmlDivElement;
use yew::prelude::*;

use i18n::{en_us, zh_cn, LanguageType};
use sandcat_sdk::model::{friend::Friend, RightContentType};
use utils::tr;

use crate::{
    action::Action,
    constant::{ACCOUNT, NICKNAME, REGION},
};

#[derive(Default)]
pub struct FriendCard {
    node_ref: NodeRef,
    i18n: FluentBundle<FluentResource>,
}

pub enum FriendCardMsg {
    Destroy,
}

#[derive(Properties, Clone, PartialEq)]
pub struct FriendCardProps {
    // container: Element,
    pub friend: Friend,
    pub user_id: AttrValue,
    pub avatar: AttrValue,
    pub nickname: AttrValue,
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
        let res = match ctx.props().lang {
            LanguageType::ZhCN => zh_cn::FRIEND_CARD,
            LanguageType::EnUS => en_us::FRIEND_CARD,
        };
        let i18n = utils::create_bundle(res);

        Self {
            i18n,
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
        let friend = &ctx.props().friend;
        let mut remark = html!();
        if friend.remark.is_some() {
            remark = html!(<span><b>{&friend.remark}</b></span>)
        }
        html! {
            <div
                class="friend-card box-shadow"
                tabindex="-1"
                ref={self.node_ref.clone()}
                onblur={ctx.link().callback(|_| FriendCardMsg::Destroy)}
                >
                <div class="friend-card-header">
                    <img alt="avatar" src={utils::get_avatar_url(&friend.avatar)} class="friend-card-avatar"/>
                    <div class="friend-card-info">
                        {remark}
                        <span>{tr!(self.i18n, NICKNAME)}{&friend.name}</span>
                        <span>{tr!(self.i18n, ACCOUNT)}{&friend.account}</span>
                        <span>{tr!(self.i18n, REGION)}{friend.region.clone().unwrap_or_default()} </span>
                    </div>
                </div>
                <div class="friend-card-body">
                    <Action
                        friend_id={&friend.friend_id}
                        user_id={&ctx.props().user_id}
                        avatar={&ctx.props().avatar}
                        nickname={&ctx.props().nickname}
                        conv_type={RightContentType::Friend}
                        lang={ctx.props().lang}/>
                </div>
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if !first_render {
            return;
        }
        if let Some(node) = self.node_ref.cast::<HtmlDivElement>() {
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
            let _ = node
                .style()
                .set_property("top", format!("{}px", y).as_str());
            let _ = node
                .style()
                .set_property("left", format!("{}px", x).as_str());
            let _ = node.focus();
        }
    }
}
