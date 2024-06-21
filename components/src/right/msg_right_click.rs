use fluent::{FluentBundle, FluentResource};
use web_sys::HtmlDivElement;
use yew::prelude::*;
use yew::{Component, Properties};

use i18n::{en_us, zh_cn, LanguageType};
use sandcat_sdk::model::ContentType;
use sandcat_sdk::state::I18nState;
use utils::tr;
use yewdux::Dispatch;

use crate::constant::{DELETE, FORWARD};

pub struct MsgRightClick {
    node: NodeRef,
    i18n: FluentBundle<FluentResource>,
}

#[derive(Debug, Clone, Properties, PartialEq)]
pub struct RightClickPanelProps {
    pub content_type: ContentType,
    pub x: i32,
    pub y: i32,
    pub close: Callback<()>,
    pub delete: Callback<()>,
    pub forward: Callback<()>,
}

pub enum RightClickPanelMsg {}

impl Component for MsgRightClick {
    type Message = RightClickPanelMsg;

    type Properties = RightClickPanelProps;

    fn create(_ctx: &Context<Self>) -> Self {
        let res = match Dispatch::<I18nState>::global().get().lang {
            LanguageType::ZhCN => zh_cn::RIGHT_CLICK_PANEL,
            LanguageType::EnUS => en_us::RIGHT_CLICK_PANEL,
        };
        let i18n = utils::create_bundle(res);
        Self {
            node: NodeRef::default(),
            i18n,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let style = format!("left: {}px; top: {}px;", ctx.props().x, ctx.props().y);

        let mut forward = html!();
        if ctx.props().content_type != ContentType::Audio
            || ctx.props().content_type != ContentType::AudioCall
            || ctx.props().content_type != ContentType::VideoCall
        {
            forward = html!(
                 <div class="right-click-panel-item hover" onclick={ctx.props().forward.reform(|_|())}>
                     {tr!(self.i18n, FORWARD)}
                 </div>
            );
        };
        html! {
            <div ref={self.node.clone()}
                {style}
                class="right-click-panel box-shadow" tabindex="0"
                onblur={ctx.props().close.reform(|_|())}
                >
                <div class="right-click-panel-item hover" onclick={ctx.props().delete.reform(|_|())}>
                    {tr!(self.i18n, DELETE)}
                </div>
                {forward}
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if let Some(node) = self.node.cast::<HtmlDivElement>() {
            let _ = node.focus();
        }
    }
}
