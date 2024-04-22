use std::rc::Rc;

use fluent::{FluentBundle, FluentResource};
use nanoid::nanoid;
use yew::prelude::*;
use yewdux::Dispatch;

use crate::i18n::{en_us, zh_cn, LanguageType};
use crate::icons::{PhoneIcon, SendMsgIcon, VideoIcon};
use crate::model::message::{InviteMsg, InviteType};
use crate::model::{ComponentType, CurrentItem};
use crate::state::ConvState;
use crate::state::SendCallState;
use crate::{model::RightContentType, state::AppState};
use crate::{tr, utils};

// 联系人卡面上的动作组件：发消息、点电话、打视频
pub struct Action {
    i18n: FluentBundle<FluentResource>,
    state: Rc<AppState>,
    app_dis: Dispatch<AppState>,
}

pub enum ActionMsg {
    AppStateChanged(Rc<AppState>),
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
        let app_dis = Dispatch::global().subscribe(ctx.link().callback(ActionMsg::AppStateChanged));
        let res = match ctx.props().lang {
            LanguageType::ZhCN => zh_cn::ACTION,
            LanguageType::EnUS => en_us::ACTION,
        };
        let i18n = utils::create_bundle(res);
        Action {
            i18n,
            state: app_dis.get(),
            app_dis,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ActionMsg::SendMessage => {
                let id = ctx.props().id.clone();

                self.app_dis
                    .reduce_mut(|s| s.component_type = ComponentType::Messages);

                Dispatch::<ConvState>::global().reduce_mut(|s| {
                    s.conv = CurrentItem {
                        item_id: id,
                        content_type: ctx.props().conv_type.clone(),
                    }
                });
                log::debug!("conv type: {:?}", ctx.props().conv_type.clone());
            }
            ActionMsg::AppStateChanged(state) => {
                self.state = state;
            }
            ActionMsg::SendCallInvite(t) => {
                Dispatch::<SendCallState>::global().reduce_mut(|s| {
                    s.msg = InviteMsg {
                        local_id: nanoid!().into(),
                        server_id: AttrValue::default(),
                        send_id: self.state.login_user.id.clone(),
                        friend_id: ctx.props().id.clone(),
                        create_time: chrono::Local::now().timestamp_millis(),
                        invite_type: t,
                    }
                });
            }
        }
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let onclick = ctx.link().callback(|_| ActionMsg::SendMessage);

        html! {
            <div class="action">
                <div {onclick}>
                    <SendMsgIcon/>
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
