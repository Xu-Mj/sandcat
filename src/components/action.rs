use std::rc::Rc;

use fluent::{FluentBundle, FluentResource};
use nanoid::nanoid;
use yew::prelude::*;

use crate::i18n::{en_us, zh_cn, LanguageType};
use crate::icons::{MessagesIcon, PhoneIcon, VideoIcon};
use crate::model::message::{InviteMsg, InviteType};
use crate::model::{ComponentType, CurrentItem};
use crate::pages::{ConvState, RecSendCallState};
use crate::{model::RightContentType, pages::AppState};
use crate::{tr, utils};

// 联系人卡面上的动作组件：发消息、点电话、打视频
pub struct Action {
    i18n: FluentBundle<FluentResource>,
    state: Rc<AppState>,
    _listener: ContextHandle<Rc<AppState>>,
    conv_state: Rc<ConvState>,
    _conv_listener: ContextHandle<Rc<ConvState>>,
    msg_state: Rc<RecSendCallState>,
    _call_listener: ContextHandle<Rc<RecSendCallState>>,
}

pub enum ActionMsg {
    AppStateChanged(Rc<AppState>),
    ConvStateChanged(Rc<ConvState>),
    CallStateChanged(Rc<RecSendCallState>),
    SendMessage,
    SendCallInvite(InviteType),
}

#[derive(Properties, Clone, PartialEq)]
pub struct ActionProps {
    pub id: AttrValue,
    pub conv_type: RightContentType,
    pub lang: LanguageType,
}

impl Component for Action {
    type Message = ActionMsg;
    type Properties = ActionProps;

    fn create(ctx: &Context<Self>) -> Self {
        let (state, _listener) = ctx
            .link()
            .context(ctx.link().callback(ActionMsg::AppStateChanged))
            .expect("action state needed");
        let (conv_state, _conv_listener) = ctx
            .link()
            .context(ctx.link().callback(ActionMsg::ConvStateChanged))
            .expect("action state needed");
        let (msg_state, _call_listener) = ctx
            .link()
            .context(ctx.link().callback(ActionMsg::CallStateChanged))
            .expect("action state needed");
        let res = match ctx.props().lang {
            LanguageType::ZhCN => zh_cn::ACTION,
            LanguageType::EnUS => en_us::ACTION,
        };
        let i18n = utils::create_bundle(res);
        Action {
            i18n,
            state,
            _listener,
            conv_state,
            _conv_listener,
            msg_state,
            _call_listener,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ActionMsg::SendMessage => {
                let id = ctx.props().id.clone();
                self.state.switch_com_event.emit(ComponentType::Messages);
                self.conv_state.state_change_event.emit(CurrentItem {
                    item_id: id,
                    content_type: ctx.props().conv_type.clone(),
                });
                log::debug!("conv type: {:?}", ctx.props().conv_type.clone());
                true
            }
            ActionMsg::AppStateChanged(state) => {
                self.state = state;
                false
            }
            ActionMsg::ConvStateChanged(state) => {
                self.conv_state = state;
                false
            }
            ActionMsg::CallStateChanged(_) => false,
            ActionMsg::SendCallInvite(t) => {
                self.msg_state.call_event.emit(InviteMsg {
                    local_id: nanoid!().into(),
                    server_id: AttrValue::default(),
                    send_id: self.state.login_user.id.clone(),
                    friend_id: ctx.props().id.clone(),
                    create_time: chrono::Local::now().timestamp_millis(),
                    invite_type: t,
                });
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|_| ActionMsg::SendMessage);

        html! {
            <div class="action">
                <div {onclick}>
                    <MessagesIcon/>
                    <span>{tr!(self.i18n, "send_message")}</span>
                </div>
                // don't support call for group
                if ctx.props().conv_type == RightContentType::Friend {
                    <div onclick={ctx.link().callback(|_| ActionMsg::SendCallInvite(InviteType::Audio))}>
                       <PhoneIcon/>
                        <span>{tr!(self.i18n, "voice_call")}</span>
                    </div>
                    <div onclick={ctx.link().callback(|_| ActionMsg::SendCallInvite(InviteType::Video))}>
                        <VideoIcon/>
                        <span>{tr!(self.i18n, "video_call")}</span>
                    </div>
                }
            </div>
        }
    }
}
