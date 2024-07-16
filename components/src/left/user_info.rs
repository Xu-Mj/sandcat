use fluent::{FluentBundle, FluentResource};
use log::error;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use i18n::{en_us, zh_cn, LanguageType};
use sandcat_sdk::api;
use sandcat_sdk::db;
use sandcat_sdk::model::friend::{FriendShipRequest, ReadStatus};
use sandcat_sdk::model::user::UserWithMatchType;
use sandcat_sdk::model::RightContentType;
use sandcat_sdk::state::MobileState;
use utils::tr;

use crate::action::Action;
use crate::constant::ACCOUNT;
use crate::constant::APPLIED;
use crate::constant::APPLY;
use crate::constant::APPLYING;
use crate::constant::APPLY_FAILED;
use crate::constant::APPLY_MSG;
use crate::constant::NICKNAME;
use crate::constant::REGION;
use crate::constant::REMARK;
use crate::get_platform;

pub struct UserInfoCom {
    node: NodeRef,
    i18n: FluentBundle<FluentResource>,
    apply_node: NodeRef,
    remark_node: NodeRef,
    apply_state: FriendShipRequestState,
}

#[derive(Properties, PartialEq)]
pub struct UserInfoComProps {
    pub user_id: AttrValue,
    pub avatar: AttrValue,
    pub nickname: AttrValue,
    pub lang: LanguageType,
    pub info: UserWithMatchType,
}

pub enum UserInfoComMsg {
    Apply,
    ApplyFriendResult(FriendShipRequestState),
}

#[derive(Debug, Clone, PartialEq)]
pub enum FriendShipRequestState {
    NotApply,
    Pending,
    Success,
    Fail,
}

impl Component for UserInfoCom {
    type Message = UserInfoComMsg;

    type Properties = UserInfoComProps;

    fn create(ctx: &Context<Self>) -> Self {
        let res = match ctx.props().lang {
            LanguageType::ZhCN => zh_cn::ADD_FRIEND,
            LanguageType::EnUS => en_us::ADD_FRIEND,
        };
        let i18n = utils::create_bundle(res);
        Self {
            i18n,
            node: Default::default(),
            apply_node: NodeRef::default(),
            remark_node: NodeRef::default(),
            apply_state: FriendShipRequestState::NotApply,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            UserInfoComMsg::Apply => {
                let friend_id = ctx.props().info.id.clone();
                let source = ctx.props().info.match_type.clone();
                let user_id = ctx.props().user_id.clone();
                let apply_node: HtmlInputElement = self.apply_node.cast().unwrap();
                let apply_msg = if apply_node.value().is_empty() {
                    None
                } else {
                    Some(AttrValue::from(apply_node.value()))
                };
                let remark: HtmlInputElement = self.remark_node.cast().unwrap();
                let req_remark =
                    (!remark.value().is_empty()).then_some(AttrValue::from(remark.value()));
                // 发送好友申请
                let new_friend = FriendShipRequest {
                    user_id,
                    friend_id,
                    apply_msg,
                    source,
                    req_remark,
                    platform: get_platform(MobileState::is_mobile()),
                };

                log::debug!("发送好友申请:{:?}", &new_friend);
                ctx.link().send_message(UserInfoComMsg::ApplyFriendResult(
                    FriendShipRequestState::Pending,
                ));

                // send friendship state to friendship list
                // let update_friendship_list = self.;
                ctx.link().send_future(async move {
                    match api::friends().apply_friend(new_friend).await {
                        Err(err) => {
                            log::error!("发送好友申请错误: {:?}", err);
                            UserInfoComMsg::ApplyFriendResult(FriendShipRequestState::Fail)
                        }
                        Ok(mut friendship) => {
                            friendship.is_self = true;
                            friendship.read = ReadStatus::True;
                            // 数据入库
                            if let Err(err) =
                                db::db_ins().friendships.put_friendship(&friendship).await
                            {
                                error!("put friendship error:{:?}", err);

                                return UserInfoComMsg::ApplyFriendResult(
                                    FriendShipRequestState::Fail,
                                );
                            }
                            UserInfoComMsg::ApplyFriendResult(FriendShipRequestState::Success)
                        }
                    }
                });
                false
            }
            UserInfoComMsg::ApplyFriendResult(state) => match state {
                FriendShipRequestState::Pending => {
                    self.apply_state = FriendShipRequestState::Pending;
                    true
                }
                FriendShipRequestState::Fail => {
                    self.apply_state = FriendShipRequestState::Fail;
                    true
                }
                FriendShipRequestState::Success => {
                    self.apply_state = FriendShipRequestState::Success;
                    true
                }
                FriendShipRequestState::NotApply => false,
            },
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // 根据参数渲染组件
        let apply_btn = match self.apply_state {
            FriendShipRequestState::NotApply => tr!(self.i18n, APPLY),
            FriendShipRequestState::Pending => tr!(self.i18n, APPLYING),
            FriendShipRequestState::Success => tr!(self.i18n, APPLIED),
            FriendShipRequestState::Fail => tr!(self.i18n, APPLY_FAILED),
        };
        let apply = if ctx.props().info.is_friend {
            html!(<Action
                    friend_id={&ctx.props().info.id}
                    user_id={&ctx.props().user_id}
                    avatar={&ctx.props().avatar}
                    nickname={&ctx.props().nickname}
                    conv_type={RightContentType::Friend}
                    lang={ctx.props().lang}/>)
        } else {
            let onclick = ctx.link().callback(|_| UserInfoComMsg::Apply);
            html! {
                <div class="apply-detail">
                    <div class="apply-msg">
                        <label>{tr!(self.i18n, APPLY_MSG)}</label>
                        <input class="apply-input" ref={self.apply_node.clone()} type="text"/>
                    </div>
                    <div class="apply-remark">
                        <label>{tr!(self.i18n, REMARK)}</label>
                        <input class="apply-input" ref={self.remark_node.clone()} type="text"/>
                    </div>
                    <div class="apply-friend" >
                        <button aria-label={apply_btn.clone()} disabled={self.apply_state != FriendShipRequestState::NotApply} {onclick} >{apply_btn}</button>
                    </div>
                </div>
            }
        };
        html! {
        <>
        <div class={"user-info"} ref={self.node.clone()}>
            <div class="friend-card-header">
                    <img alt="avatar" src={utils::get_avatar_url(&ctx.props().info.avatar)} class="friend-card-avatar"/>
                    <div class="friend-card-info">
                        // <span><b>{&self.friend.remark}</b></span>
                        <span>{tr!(self.i18n, NICKNAME)}{&ctx.props().info.name}</span>
                        <span>{tr!(self.i18n, ACCOUNT)}{&ctx.props().info.account}</span>
                        <span>{tr!(self.i18n, REGION)}{&ctx.props().info.region.clone().unwrap_or_default()} </span>
                    </div>
                </div>
                <div class="friend-card-body">
                    {apply}
                </div>
        </div>
        </>
        }
    }
}
