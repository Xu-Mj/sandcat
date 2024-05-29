use std::rc::Rc;

use gloo::{timers::callback::Timeout, utils::document};
use sandcat_sdk::state::MobileState;
use web_sys::Element;
use yew::prelude::*;
use yewdux::{Dispatch, Store};

use icons::HangUpLoadingIcon;
pub struct Dialog {
    timer: Option<Timeout>,
    is_mobile: bool,
    _loading_dis: Dispatch<LoadingState>,
}

#[derive(Debug, Clone, Default, PartialEq, Store)]
pub struct LoadingState {
    is_loading: bool,
}

pub enum DialogMsg {
    Close,
    LoadingStateChanged(Rc<LoadingState>),
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum DialogType {
    #[default]
    Info,
    Success,
    Warn,
    Error,
    Panic,
    Loading,
}

#[derive(Properties, Clone, PartialEq)]
pub struct DialogProps {
    container: Element,
    pub title: AttrValue,
    pub content: AttrValue,
    pub type_: DialogType,
    /// display time
    #[prop_or(3000)]
    pub delay: u32,
}

impl Default for DialogProps {
    fn default() -> Self {
        Self {
            container: Dialog::container(),
            title: Default::default(),
            content: Default::default(),
            type_: Default::default(),
            delay: 3000,
        }
    }
}

impl DialogProps {
    pub fn loading(content: &str) -> Self {
        Self {
            type_: DialogType::Loading,
            content: content.to_string().into(),
            ..Default::default()
        }
    }

    pub fn info(content: &str) -> Self {
        Self {
            content: content.to_string().into(),
            ..Default::default()
        }
    }

    pub fn success(content: &str) -> Self {
        Self {
            type_: DialogType::Success,
            content: content.to_string().into(),
            ..Default::default()
        }
    }

    pub fn warn(content: &str) -> Self {
        Self {
            type_: DialogType::Warn,
            content: content.to_string().into(),
            ..Default::default()
        }
    }

    pub fn error(content: &str) -> Self {
        Self {
            type_: DialogType::Error,
            content: content.to_string().into(),
            ..Default::default()
        }
    }
}

impl Component for Dialog {
    type Message = DialogMsg;
    type Properties = DialogProps;

    fn create(ctx: &Context<Self>) -> Self {
        let mut timer = None;
        if ctx.props().type_ == DialogType::Loading {
            Dispatch::<LoadingState>::global().reduce_mut(|s| s.is_loading = true);
        } else if ctx.props().type_ != DialogType::Panic {
            let container = ctx.props().container.clone();
            timer = Some(Timeout::new(ctx.props().delay, move || {
                container.remove();
            }));
        }
        Self::mount(ctx);

        Self {
            timer,
            is_mobile: Dispatch::<MobileState>::global().get().is_mobile(),
            _loading_dis: Dispatch::<LoadingState>::global()
                .subscribe_silent(ctx.link().callback(Self::Message::LoadingStateChanged)),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            DialogMsg::Close => {
                // let node = self.node_ref.cast::<HtmlDialogElement>().unwrap();
                // node.close();
                self.unmount(ctx);
            }
            DialogMsg::LoadingStateChanged(state) => {
                if !state.is_loading {
                    self.unmount(ctx);
                }
            }
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut class = classes!();
        if self.is_mobile {
            class.push("size-mobile");
        } else {
            class.push("size");
        }
        let content = match ctx.props().type_ {
            DialogType::Info => {
                class.push("notification");
                class.push("info");
                html! (<div class="content">{&ctx.props().content}</div>)
            }

            DialogType::Success => {
                class.push("notification");
                class.push("success");
                html! (<div class="content">{&ctx.props().content}</div>)
            }
            DialogType::Warn | DialogType::Error => {
                class.push("error");
                html! {
                    <>
                        <div class="header">{&ctx.props().title}</div>
                        <div class="content">{&ctx.props().content}</div>
                        <div class="footer"><div class="button">{"got it"}</div></div>
                    </>
                }
            }
            DialogType::Panic => {
                class.push("panic");
                html! {
                    <>
                        <div class="header">{&ctx.props().title}</div>
                        <div class="content">{&ctx.props().content}</div>
                        <div class="footer"></div>
                    </>
                }
            }
            DialogType::Loading => {
                html! {
                    <div class="loading">{&ctx.props().content}<HangUpLoadingIcon/></div>
                }
            }
        };
        html! {
            <div tabindex="1" {class} >
                {content}
            </div>
        }
    }
}

impl Dialog {
    fn mount(ctx: &Context<Self>) {
        // 查询body节点
        document()
            .body()
            .map(|body| body.append_child(&ctx.props().container));
    }

    pub fn container() -> Element {
        let container = document().create_element("div").unwrap();
        // 设置容器绝对定位
        container.set_class_name("notification-container");
        container
    }

    pub fn loading(msg: &str) {
        let props = DialogProps::loading(msg);
        yew::Renderer::<Dialog>::with_root_and_props(props.container.clone(), props).render();
    }

    pub fn close_loading() {
        Dispatch::<LoadingState>::global().reduce_mut(|s| s.is_loading = false);
    }

    pub fn success(msg: &str) {
        let container = Dialog::container();
        let props = DialogProps::success(msg);
        yew::Renderer::<Dialog>::with_root_and_props(container, props).render();
    }

    pub fn info(msg: &str) {
        let props = DialogProps::info(msg);
        yew::Renderer::<Dialog>::with_root_and_props(props.container.clone(), props).render();
    }

    pub fn warn(msg: &str) {
        let props = DialogProps::warn(msg);
        yew::Renderer::<Dialog>::with_root_and_props(props.container.clone(), props).render();
    }

    pub fn error(msg: &str) {
        let props = DialogProps::error(msg);
        yew::Renderer::<Dialog>::with_root_and_props(props.container.clone(), props).render();
    }

    fn unmount(&mut self, ctx: &Context<Self>) {
        self.timer = None;
        ctx.props().container.remove()
    }
}
