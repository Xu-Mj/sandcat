use fluent::{FluentBundle, FluentResource};
use web_sys::HtmlDivElement;
use yew::prelude::*;
use yew::{Component, Properties};

use abi::model::RightContentType;
use i18n::{en_us, zh_cn, LanguageType};
use utils::tr;
pub struct SetDrawer {
    node: NodeRef,
    i18n: FluentBundle<FluentResource>,
}

#[derive(Debug, Clone, Properties, PartialEq)]
pub struct SetDrawerProps {
    pub conv_type: RightContentType,
    pub is_owner: bool,
    pub close: Callback<()>,
    pub delete: Callback<()>,
    pub lang: LanguageType,
}

pub enum SetDrawerMsg {}

impl Component for SetDrawer {
    type Message = SetDrawerMsg;

    type Properties = SetDrawerProps;

    fn create(ctx: &Context<Self>) -> Self {
        let res = match ctx.props().lang {
            LanguageType::ZhCN => zh_cn::SET_DRAWER,
            LanguageType::EnUS => en_us::SET_DRAWER,
        };
        let i18n = utils::create_bundle(res);
        Self {
            i18n,
            node: NodeRef::default(),
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        let node: HtmlDivElement = self.node.cast().unwrap();
        node.focus().unwrap();
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let btn_msg = match ctx.props().conv_type {
            RightContentType::Friend => tr!(self.i18n, "del_friend"),
            RightContentType::Group => {
                if ctx.props().is_owner {
                    tr!(self.i18n, "dismiss")
                } else {
                    tr!(self.i18n, "quit")
                }
            }
            _ => String::new(),
        };
        html! {
            <div ref={self.node.clone()}
                class="set-drawer box-shadow" tabindex="0"
                onblur={ctx.props().close.reform(|_|())}
                >
                <div class="set-drawer-item hover" onclick={ctx.props().delete.reform(|_|())}>
                    {btn_msg}
                </div>
            </div>
        }
    }
}
