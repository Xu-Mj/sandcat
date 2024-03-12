use web_sys::HtmlDivElement;
use yew::prelude::*;
use yew::{Component, Properties};

use crate::model::RightContentType;
pub struct SetDrawer {
    node: NodeRef,
}

#[derive(Debug, Clone, Properties, PartialEq)]
pub struct SetDrawerProps {
    pub conv_type: RightContentType,
    pub is_owner: bool,
    pub close: Callback<()>,
    pub delete: Callback<()>,
}

pub enum SetDrawerMsg {}

impl Component for SetDrawer {
    type Message = SetDrawerMsg;

    type Properties = SetDrawerProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            node: NodeRef::default(),
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        let node: HtmlDivElement = self.node.cast().unwrap();
        node.focus().unwrap();
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let btn_msg = match ctx.props().conv_type {
            RightContentType::Friend => "删除好友",
            RightContentType::Group => "退出群聊",
            _ => "未知操作",
        };
        let mut dismiss_group = html!();
        if ctx.props().is_owner {
            dismiss_group = html! {
                <div class="set-drawer-item hover" /* onclick={ctx.props().delete.reform(|_|())} */>
                    {"解散群聊"}
                </div>
            }
        }

        // let style =
        html! {
            <div ref={self.node.clone()}
                class="set-drawer box-shadow" tabindex="0"
                onblur={ctx.props().close.reform(|_|())}
                >
                <div class="set-drawer-item hover" /* onclick={ctx.props().delete.reform(|_|())} */>
                    {btn_msg}
                </div>
                {dismiss_group}
            </div>
        }
    }
}
