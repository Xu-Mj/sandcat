#![allow(dead_code)]
#![allow(unused_imports)]
use std::{rc::Rc, sync::atomic::AtomicU8};

use yew::prelude::*;

use crate::{
    icons::{ContactsIcon, MessagesIcon, SettingIcon},
    pages::{AppState, ComponentType, ConvState, RecSendMessageState},
};

/// 增加双击切换置顶未读消息
pub struct Top {
    state: Rc<AppState>,
    _handler: ContextHandle<Rc<AppState>>,
    // 修改为单独的未读消息增加减少的状态
    conv_state: Rc<ConvState>,
    _conv_handler: ContextHandle<Rc<ConvState>>,
    count: usize,
}

#[derive(Properties, PartialEq)]
pub struct TopProps {
    pub avatar: AttrValue,
}

pub enum TopMsg {
    AppContextChanged(Rc<AppState>),
    ConvStateChanged(Rc<ConvState>),
    EmptyCallback,
}

impl Component for Top {
    type Message = TopMsg;

    type Properties = TopProps;

    fn create(ctx: &Context<Self>) -> Self {
        let (state, _handler) = ctx
            .link()
            .context::<Rc<AppState>>(ctx.link().callback(TopMsg::AppContextChanged))
            .expect("need state");
        let (conv_state, _conv_handler) = ctx
            .link()
            .context::<Rc<ConvState>>(ctx.link().callback(TopMsg::ConvStateChanged))
            .expect("need state");
        Self {
            state,
            _handler,
            conv_state,
            _conv_handler,
            count: 0,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TopMsg::AppContextChanged(state) => {
                self.state = state;
                true
            }
            TopMsg::EmptyCallback => false,
            TopMsg::ConvStateChanged(state) => {
                self.conv_state = state;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut msg_class = "top-icon-selected";
        let msg_onclick = if self.state.component_type != ComponentType::Messages {
            msg_class = "hover";
            self.state
                .switch_com_event
                .reform(move |_| ComponentType::Messages)
        } else {
            ctx.link().callback(move |_| TopMsg::EmptyCallback)
        };
        let mut msg_class = classes!(msg_class);
        msg_class.push("msg-icon");
        let mut contact_class = "top-icon-selected";
        let contact_onclick = if self.state.component_type != ComponentType::Contacts {
            contact_class = "hover";
            self.state
                .switch_com_event
                .reform(move |_| ComponentType::Contacts)
        } else {
            ctx.link().callback(move |_| TopMsg::EmptyCallback)
        };
        let mut setting_class = "top-icon-selected";
        // let setting_onclick = if self.state.component_type != ComponentType::Setting {
        //     setting_class = "hover";
        //     self.state
        //         .switch_com_event
        //         .reform(move |_| ComponentType::Setting)
        // } else {
        //     ctx.link().callback(move |_| TopMsg::EmptyCallback)
        // };
        let mut count = html!();
        if self.conv_state.conv.unread_count > 0 {
            count = html! {
                <span class="unread-count">
                    {self.conv_state.conv.unread_count}
                </span>
            };
        }
        html! {
            <div class="top">
                <div class="top-left">
                    <img class="avatar" src={ctx.props().avatar.clone()} />
                </div>
                <div class="top-right">
                    <span class={msg_class} onclick={msg_onclick}>
                        <MessagesIcon />
                        {count}
                    </span>
                    <span class={contact_class} onclick={contact_onclick}>
                        <ContactsIcon/>
                    </span>
                    // <span class={setting_class} onclick={setting_onclick}>
                    //     <SettingIcon />
                    // </span>
                </div>

            </div>
        }
    }
}
