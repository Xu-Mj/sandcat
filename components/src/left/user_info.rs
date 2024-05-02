use fluent::{FluentBundle, FluentResource};
use web_sys::HtmlInputElement;
use yew::prelude::*;

use abi::model::friend::{FriendShipRequest, ReadStatus};
use abi::model::user::UserWithMatchType;
use abi::model::RightContentType;
use i18n::{en_us, zh_cn, LanguageType};
use utils::tr;

use crate::action::Action;

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
    Pendding,
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
            node: Default::default(),
            i18n,
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
                };

                log::debug!("发送好友申请:{:?}", &new_friend);
                ctx.link().send_message(UserInfoComMsg::ApplyFriendResult(
                    FriendShipRequestState::Pendding,
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
                            db::friendships().await.put_friendship(&friendship).await;
                            UserInfoComMsg::ApplyFriendResult(FriendShipRequestState::Success)
                        }
                    }
                });
                false
            }
            UserInfoComMsg::ApplyFriendResult(state) => match state {
                FriendShipRequestState::Pendding => {
                    self.apply_state = FriendShipRequestState::Pendding;
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
            FriendShipRequestState::NotApply => tr!(self.i18n, "apply"),
            FriendShipRequestState::Pendding => tr!(self.i18n, "applying"),
            FriendShipRequestState::Success => tr!(self.i18n, "applied"),
            FriendShipRequestState::Fail => tr!(self.i18n, "apply_failed"),
        };
        let apply = if ctx.props().info.is_friend {
            html!(<Action
                    friend_id={&ctx.props().info.id}
                    user_id={&ctx.props().user_id}
                    conv_type={RightContentType::Friend}
                    lang={ctx.props().lang}/>)
        } else {
            let onclick = ctx.link().callback(|_| UserInfoComMsg::Apply);
            html! {
                <div class="apply-detail">
                    <div class="apply-msg">
                        <label>{tr!(self.i18n, "apply_msg")}</label>
                        <input class="apply-input" ref={self.apply_node.clone()} type="text"/>
                    </div>
                    <div class="apply-remark">
                        <label>{tr!(self.i18n, "remark")}</label>
                        <input class="apply-input" ref={self.remark_node.clone()} type="text"/>
                    </div>
                    <div class="apply-friend" >
                        <button disabled={self.apply_state != FriendShipRequestState::NotApply} {onclick}  >{apply_btn}</button>
                    </div>
                </div>
            }
        };
        html! {
        <>
        <div class={"user-info"} ref={self.node.clone()}>
            <div class="friend-card-header">
                    <img src={&ctx.props().info.avatar} class="friend-card-avatar"/>
                    <div class="friend-card-info">
                        // <span><b>{&self.friend.remark}</b></span>
                        <span>{tr!(self.i18n, "nickname")}{&ctx.props().info.name}</span>
                        <span>{tr!(self.i18n, "account")}{&ctx.props().info.account}</span>
                        <span>{tr!(self.i18n, "region")}{&ctx.props().info.region.clone().unwrap_or_default()} </span>
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
