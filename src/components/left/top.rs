use std::rc::Rc;

use yew::prelude::*;

use crate::{
    icons::{ContactsIcon, MessagesIcon},
    pages::{AppState, ComponentType, ConvState, UnreadState},
};

/// 增加双击切换置顶未读消息
pub struct Top {
    state: Rc<AppState>,
    _handler: ContextHandle<Rc<AppState>>,
    // 修改为单独的未读消息增加减少的状态
    conv_state: Rc<ConvState>,
    _conv_handler: ContextHandle<Rc<ConvState>>,
    unread_state: Rc<UnreadState>,
    _unread_handler: ContextHandle<Rc<UnreadState>>,
}

#[derive(Properties, PartialEq)]
pub struct TopProps {
    pub avatar: AttrValue,
    pub name: AttrValue,
}

pub enum TopMsg {
    AppContextChanged(Rc<AppState>),
    ConvStateChanged(Rc<ConvState>),
    UnreadStateChanged(Rc<UnreadState>),
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
        let (unread_state, _unread_handler) = ctx
            .link()
            .context::<Rc<UnreadState>>(ctx.link().callback(TopMsg::UnreadStateChanged))
            .expect("need state");
        Self {
            state,
            _handler,
            conv_state,
            _conv_handler,
            unread_state,
            _unread_handler,
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
            TopMsg::UnreadStateChanged(state) => {
                self.unread_state = state;
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
        // let mut setting_class = "top-icon-selected";
        // let setting_onclick = if self.state.component_type != ComponentType::Setting {
        //     setting_class = "hover";
        //     self.state
        //         .switch_com_event
        //         .reform(move |_| ComponentType::Setting)
        // } else {
        //     ctx.link().callback(move |_| TopMsg::EmptyCallback)
        // };
        let mut count = html!();
        if self.unread_state.unread.unread_msg > 0 {
            count = html! {
                <span class="unread-count">
                    {self.unread_state.unread.unread_msg}
                </span>
            };
        }
        html! {
            <div class="top">
                <div class="top-left">
                    <img class="avatar" title={ctx.props().name.clone()} src={ctx.props().avatar.clone()} />
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
