#![allow(unused_variables)]
#![allow(dead_code)]

use gloo::{timers::callback::Timeout, utils::document};
use web_sys::{Element, HtmlDialogElement, HtmlElement};
use yew::prelude::*;

pub struct Dialog {
    node_ref: NodeRef,
    container: Element,
    pub title: AttrValue,
    pub content: AttrValue,
    pub type_: AttrValue,
}

impl Dialog {
    fn mount(&self) {
        // 查询body节点
        let body = document().body().expect("body is not defined");
        // 将dialog渲染到容器中
        body.append_child(&self.container.clone()).unwrap();
    }

    pub fn container() -> Element {
        let container = document().create_element("div").unwrap();
        // 设置容器绝对定位
        container
            .set_attribute("style", "position: absolute;")
            .unwrap();
        container
    }
    pub fn success(msg: AttrValue) {
        let container = Dialog::container();
        let props = DialogProps {
            container: container.clone(),
            title: "成功".into(),
            content: msg,
            type_: "success".into(),
            delay: 1000,
        };
        yew::Renderer::<Dialog>::with_root_and_props(container, props).render();
        // yew::Renderer::<Dialog>::with_root(dialog.container.clone()).render();
        /* Timeout::new(1000, move || {
            dialog.unmount();
        })
        .forget(); */
    }

    fn unmount(&self) {
        self.container.remove()
    }
}
pub enum DialogMsg {
    Close,
}
#[derive(Properties, Clone, PartialEq)]
pub struct DialogProps {
    container: Element,
    pub title: AttrValue,
    pub content: AttrValue,
    pub type_: AttrValue,
    pub delay: u32,
}

impl Component for Dialog {
    type Message = DialogMsg;
    type Properties = DialogProps;

    fn create(ctx: &Context<Self>) -> Self {
        let container = ctx.props().container.clone();
        let dialog = Self {
            node_ref: NodeRef::default(),
            container: container.clone(),
            title: AttrValue::default(),
            content: AttrValue::default(),
            type_: AttrValue::default(),
        };
        dialog.mount();

        Timeout::new(ctx.props().delay, move || {
            container.remove();
        })
        .forget();
        dialog
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            // gloo::console::log!("rendered", self.container.clone());
            let node = self.node_ref.cast::<HtmlElement>().unwrap();
            node.focus().unwrap();
            let node = self.node_ref.cast::<HtmlDialogElement>().unwrap();
            let _ = node.show_modal();
        }
    }
    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            DialogMsg::Close => {
                // let node = self.node_ref.cast::<HtmlDialogElement>().unwrap();
                // node.close();
                self.unmount();
            }
        }
        true
        // false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onblur = ctx.link().callback(|_| DialogMsg::Close);
        html! {
        <div tabindex="1" ref={self.node_ref.clone()} {onblur}>
        </div>
        /* <dialog ref={self.node_ref.clone()} {onblur}>
        <div class="dialog">
            <div class="dialog-content">
                <div class="dialog-header">
                    <div class="dialog-title">{"标题"}</div>
                    <div class="dialog-close">{"×"}</div>
                </div>
                <div class="dialog-body">
                    {"内容"}
                </div>
            </div>
        </div>
        </dialog> */
                // <div class="dialog-footer">
            }
    }
}
