use fluent::{FluentBundle, FluentResource};
use web_sys::HtmlDivElement;
use yew::prelude::*;
use yew::{Component, Properties};

use i18n::{en_us, zh_cn, LanguageType};
use icons::{DeleteIcon, ForwardIcon, QuoteIcon};
use sandcat_sdk::model::ContentType;
use sandcat_sdk::state::{I18nState, Notify};
use utils::tr;

use crate::constant::{DELETE, FORWARD, RELATED};

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
    pub related: Callback<()>,
}

pub enum RightClickPanelMsg {}

impl Component for MsgRightClick {
    type Message = RightClickPanelMsg;

    type Properties = RightClickPanelProps;

    fn create(_ctx: &Context<Self>) -> Self {
        let res = match I18nState::get().lang {
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
        let mut related = html!();
        if ctx.props().content_type != ContentType::Audio
            || ctx.props().content_type != ContentType::AudioCall
            || ctx.props().content_type != ContentType::VideoCall
        {
            forward = html!(
                 <div class="right-click-panel-item right-click-panel-item-display  hover"
                    onclick={ctx.props().forward.reform(|_|())}>
                    <ForwardIcon fill={"var(--color-text)"}/>{tr!(self.i18n, FORWARD)}
                 </div>
            );

            related = html!(
                 <div class="right-click-panel-item right-click-panel-item-display  hover"
                    onclick={ctx.props().related.reform(|_|())}>
                    <QuoteIcon fill={"var(--color-text)"}/>{tr!(self.i18n, RELATED)}
                 </div>
            );
        };
        html! {
            <div ref={self.node.clone()}
                {style}
                class="right-click-panel box-shadow" tabindex="0"
                onblur={ctx.props().close.reform(|_|())}
                >
                {forward}
                {related}
                <div class="right-click-panel-item delete-color right-click-panel-item-display hover"
                    onclick={ctx.props().delete.reform(|_|())}>
                    <DeleteIcon fill={"var(--color-text-delete)"}/>{tr!(self.i18n, DELETE)}
                </div>
            </div>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if let Some(node) = self.node.cast::<HtmlDivElement>() {
            let _ = node.focus();
        }
    }
}
