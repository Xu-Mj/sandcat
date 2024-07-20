pub mod add_friend;
pub mod contacts;
pub mod conv_com;
pub mod list_item;
pub mod right_click_panel;
pub mod top;
pub mod user_info;

use std::rc::Rc;

use gloo::utils::document;
use log::error;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::HtmlDivElement;
use yew::prelude::*;
use yewdux::Dispatch;

use sandcat_sdk::model::ComponentType;
use sandcat_sdk::state::{AppState, ComponentTypeState, MobileState, Notify};

use crate::left::contacts::Contacts;
use crate::left::conv_com::Chats;
use crate::left::top::Top;

#[derive(Properties, PartialEq, Debug)]
pub struct LeftProps {
    pub user_id: AttrValue,
}

pub enum LeftMsg {
    StateChanged(Rc<AppState>),
    ComStateChanged(Rc<ComponentTypeState>),
    ResizerMouseDown(MouseEvent),
    ResizerMouseUp,
}
pub struct Left {
    node: NodeRef,
    _dispatch: Dispatch<AppState>,
    app_state: Rc<AppState>,
    com_type: Rc<ComponentTypeState>,
    _com_dis: Dispatch<ComponentTypeState>,
    mouse_move: Option<Closure<dyn FnMut(MouseEvent)>>,
    mouse_up: Option<Closure<dyn FnMut(MouseEvent)>>,
}

impl Component for Left {
    type Message = LeftMsg;
    type Properties = LeftProps;

    fn create(ctx: &Context<Self>) -> Self {
        let dispatch = Dispatch::global().subscribe(ctx.link().callback(LeftMsg::StateChanged));
        let com_dis = Dispatch::global().subscribe(ctx.link().callback(LeftMsg::ComStateChanged));
        Self {
            node: NodeRef::default(),
            app_state: dispatch.get(),
            _dispatch: dispatch,
            com_type: com_dis.get(),
            _com_dis: com_dis,
            mouse_move: None,
            mouse_up: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            LeftMsg::StateChanged(state) => {
                self.app_state = state;
                true
            }
            LeftMsg::ComStateChanged(state) => {
                self.com_type = state;
                true
            }
            LeftMsg::ResizerMouseDown(event) => {
                event.prevent_default();
                event.stop_propagation();

                //  set onmousemove event for document
                if event.target().is_some() {
                    // get left container
                    let node = self.node.cast::<HtmlDivElement>().unwrap();

                    // mouse move event
                    let listener = Closure::wrap(Box::new(move |event: MouseEvent| {
                        let x = event.client_x();
                        // set the width of the element; ignore error
                        let _ = node.style().set_property("width", &format!("{}px", x));
                    })
                        as Box<dyn FnMut(MouseEvent)>);
                    if let Err(err) = document().add_event_listener_with_callback(
                        "mousemove",
                        listener.as_ref().unchecked_ref(),
                    ) {
                        error!("Failed to add mousemove event listener: {:?}", err);
                    };

                    // register mouse up for document
                    let ctx = ctx.link().clone();
                    let mouse_up = Closure::wrap(Box::new(move |_: MouseEvent| {
                        ctx.send_message(LeftMsg::ResizerMouseUp);
                    })
                        as Box<dyn FnMut(MouseEvent)>);

                    if let Err(err) = document().add_event_listener_with_callback(
                        "mouseup",
                        mouse_up.as_ref().unchecked_ref(),
                    ) {
                        error!("Failed to add mouseup event listener: {:?}", err);
                    };

                    self.mouse_move = Some(listener);
                    self.mouse_up = Some(mouse_up);
                }
                false
            }
            LeftMsg::ResizerMouseUp => {
                let document = document();

                // release mouse move event
                if let Some(listener) = self.mouse_move.take() {
                    // remove mousemove event
                    if let Err(err) = document.remove_event_listener_with_callback(
                        "mousemove",
                        listener.as_ref().unchecked_ref(),
                    ) {
                        error!("Failed to remove mousemove event listener: {:?}", err);
                    };
                }

                // release mouse up event
                if let Some(mouse_up) = self.mouse_up.as_ref() {
                    if let Err(err) = document.remove_event_listener_with_callback(
                        "mouseup",
                        mouse_up.as_ref().unchecked_ref(),
                    ) {
                        error!("Failed to remove mouseup event listener: {:?}", err);
                    };
                }
                false
            }
        }
    }

    // 左侧面板总布局,包含顶部选项栏、左侧列表、
    // 点击顶部选项栏切换左侧列表
    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut classes = "slider";
        match self.com_type.component_type {
            ComponentType::Contacts => {
                classes = "slider move-left";
            }
            ComponentType::Messages => {
                classes = "slider ";
                // <Messages />
            }
            ComponentType::Setting => {}
            ComponentType::Default => {}
        };
        let (class, resizer) = match *MobileState::get() {
            MobileState::Desktop => (
                "left-container",
                html!(
                    <div class="left-resizer" onmousedown={ctx.link().callback( LeftMsg::ResizerMouseDown)}>
                    </div>
                ),
            ),
            MobileState::Mobile => ("left-container-mobile", html!()),
        };
        html! {
            <div {class} ref={self.node.clone()}>
                // 左侧顶部组件：包含头像以及功能切换
                <Top />
                <div class="left-down">
                    <div class={classes}>
                    <Chats user_id={&ctx.props().user_id}
                            avatar={&self.app_state.login_user.avatar} />
                    <Contacts
                        user_id={&ctx.props().user_id}
                        avatar={&self.app_state.login_user.avatar}
                        nickname={&self.app_state.login_user.name}/>
                    </div>
                </div>
                {resizer}
            </div>
        }
    }
}
