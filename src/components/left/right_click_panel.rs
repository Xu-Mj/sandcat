use web_sys::HtmlDivElement;
use yew::prelude::*;
use yew::{Component, Properties};
pub struct RightClickPanel {
    node: NodeRef,
}

#[derive(Debug, Clone, Properties, PartialEq)]
pub struct RightClickPanelProps {
    pub x: i32,
    pub y: i32,
    pub close: Callback<()>,
    pub delete: Callback<()>,
    pub mute: Callback<()>,
}

pub enum RightClickPanelMsg {}

impl Component for RightClickPanel {
    type Message = RightClickPanelMsg;

    type Properties = RightClickPanelProps;

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
        let style = format!("left: {}px; top: {}px;", ctx.props().x, ctx.props().y);
        html! {
            <div ref={self.node.clone()}
                {style}
                class="right-click-panel box-shadow" tabindex="0"
                onblur={ctx.props().close.reform(|_|())}
                >
                <div class="right-click-panel-item hover" onclick={ctx.props().delete.reform(|_|())}>
                    {"删除会话"}
                </div>
                <div class="right-click-panel-item hover" onclick={ctx.props().mute.reform(|_|())}>
                    {"消息免打扰"}
                </div>
            </div>
        }
    }
}
