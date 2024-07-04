use std::rc::Rc;

use fluent::{FluentBundle, FluentResource};
use log::error;
use sandcat_sdk::model::conversation::Conversation;
use sandcat_sdk::model::ContentType;
use sandcat_sdk::state::CreateConvState;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, MouseEvent};
use yew::{html, AttrValue, Component, Context, Html, NodeRef, Properties};
use yewdux::Dispatch;

use i18n::{en_us, zh_cn, LanguageType};
use icons::UpIcon;
use sandcat_sdk::api;
use sandcat_sdk::db;
use sandcat_sdk::model::friend::{
    Friend, FriendShipAgree, FriendShipWithUser, FriendStatus, ReadStatus,
};
use sandcat_sdk::model::FriendShipStateType;
use sandcat_sdk::state::FriendShipState;
use utils::tr;

use crate::constant::{
    ACCEPT, ADDED, APPLY_MSG, CANCEL, GO_VERIFY, MESSAGE, REMARK, REQUESTED, TITLE,
};

pub struct FriendShipList {
    list: Vec<FriendShipWithUser>,
    friendship_state: Rc<FriendShipState>,
    _fs_dis: Dispatch<FriendShipState>,
    show_detail: bool,
    detail: Option<FriendShipWithUser>,
    apply_msg_node: NodeRef,
    response_msg_node: NodeRef,
    i18n: FluentBundle<FluentResource>,
}

pub enum FriendShipListMsg {
    FriendShipStateChanged(Rc<FriendShipState>),
    QueryFriendships(Vec<FriendShipWithUser>),
    AgreeFriendShip,
    AgreeFriendShipRes(RequestStatus),
    ShowDetail(Box<FriendShipWithUser>),
    Cancel,
}

pub enum RequestStatus {
    Failed(AttrValue),
    Success(AttrValue, Box<Friend>),
    // Pending,
}

#[derive(Properties, PartialEq, Clone)]
pub struct FriendShipListProps {
    pub lang: LanguageType,
    pub user_id: AttrValue,
}

impl Component for FriendShipList {
    type Message = FriendShipListMsg;
    type Properties = FriendShipListProps;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async {
            FriendShipListMsg::QueryFriendships(db::db_ins().friendships.get_list().await.unwrap())
        });
        let fs_dis = Dispatch::global().subscribe_silent(
            ctx.link()
                .callback(FriendShipListMsg::FriendShipStateChanged),
        );
        let res = match ctx.props().lang {
            LanguageType::ZhCN => zh_cn::FRIENDSHIP,
            LanguageType::EnUS => en_us::FRIENDSHIP,
        };
        let i18n = utils::create_bundle(res);
        Self {
            list: vec![],
            i18n,
            friendship_state: fs_dis.get(),
            _fs_dis: fs_dis,
            show_detail: false,
            detail: None,
            apply_msg_node: NodeRef::default(),
            response_msg_node: NodeRef::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            FriendShipListMsg::QueryFriendships(list) => {
                self.list = list;
                true
            }
            FriendShipListMsg::AgreeFriendShip => {
                // 根据id查找元素，修改该条数据的状态--> 正在请求
                // let pos = self
                //     .list
                //     .iter()
                //     .position(|item| item.friendship_id == id.clone());
                if self.detail.is_some() {
                    // let item = self.list.get_mut(pos.unwrap()).unwrap();
                    let item = self.detail.as_mut().unwrap();
                    item.status = FriendStatus::Accepted as i32;

                    let remark = self.apply_msg_node.cast::<HtmlInputElement>().unwrap();
                    let response_msg = self.response_msg_node.cast::<HtmlInputElement>().unwrap();
                    let resp_remark = if remark.value().is_empty() {
                        None
                    } else {
                        Some(remark.value())
                    };
                    let resp_msg = if response_msg.value().is_empty() {
                        None
                    } else {
                        Some(response_msg.value())
                    };
                    let friendship_req = FriendShipAgree {
                        fs_id: item.fs_id.clone(),
                        resp_msg,
                        resp_remark,
                    };
                    let friendship_id = item.fs_id.clone();
                    // 发送好友同意请求
                    ctx.link().send_future(async move {
                        match api::friends().agree_friend(friendship_req).await {
                            Ok(res) => {
                                log::debug!("好友请求成功:{:?}", &res);
                                // todo
                                FriendShipListMsg::AgreeFriendShipRes(RequestStatus::Success(
                                    friendship_id,
                                    Box::new(res),
                                ))
                            }
                            Err(err) => {
                                log::debug!("好友请求失败:{:?}", err);
                                FriendShipListMsg::AgreeFriendShipRes(RequestStatus::Failed(
                                    friendship_id,
                                ))
                            }
                        }
                    });
                }
                false
            }
            FriendShipListMsg::AgreeFriendShipRes(res) => {
                match res {
                    RequestStatus::Success(friendship_id, friend) => {
                        if let Some(item) = self
                            .list
                            .iter_mut()
                            .find(|item| item.fs_id == friendship_id)
                        {
                            item.status = FriendStatus::Accepted as i32;
                            item.read = ReadStatus::True;
                        }
                        // todo should update the database at here
                        Dispatch::<FriendShipState>::global().reduce_mut(|s| {
                            s.friend = Some(*friend.clone());
                            s.ship = None;
                            s.state_type = FriendShipStateType::Res
                        });
                        // let send_id = ctx.props().user_id.clone();
                        // update the is_operated field
                        spawn_local(async move {
                            if let Err(err) =
                                db::db_ins().friendships.agree(friendship_id.as_str()).await
                            {
                                error!("save agree friendship error:{:?}", err);
                                return;
                            }
                            if let Err(err) = db::db_ins().friends.put_friend(&friend).await {
                                error!("save friend error:{:?}", err);
                                return;
                            }

                            let mut conv = Conversation::from(*friend);
                            conv.last_msg = AttrValue::from("new friend");
                            conv.last_msg_type = ContentType::Text;
                            conv.last_msg_time = chrono::Utc::now().timestamp_millis();
                            if let Err(e) = db::db_ins().convs.put_conv(&conv).await {
                                error!("save new conversation error: {:?}", e);
                                return;
                            }
                            CreateConvState::update(conv);
                        });
                        // 发送通知给contacts，刷新列表
                    }
                    RequestStatus::Failed(id) => {
                        if let Some(item) = self.list.iter_mut().find(|item| item.fs_id == id) {
                            item.status = FriendStatus::Failed as i32;
                        }
                    }
                }
                self.show_detail = false;
                self.detail = None;
                true
            }
            FriendShipListMsg::FriendShipStateChanged(state) => {
                log::debug!(
                    "friendship state changed:{:?}, {:?}",
                    &state.friend,
                    &state.state_type
                );

                match state.state_type {
                    FriendShipStateType::Req => {
                        self.friendship_state = state;
                        self.list
                            .insert(0, self.friendship_state.ship.as_ref().unwrap().clone());
                    }
                    FriendShipStateType::Res => {
                        // response friend
                    }
                    FriendShipStateType::RecResp => {
                        self.list
                            .iter_mut()
                            .filter(|v| v.fs_id == state.friend.as_ref().unwrap().fs_id)
                            .for_each(|v| {
                                v.status = FriendStatus::Accepted as i32;
                                v.read = ReadStatus::True
                            });
                    }
                }
                true
            }
            FriendShipListMsg::ShowDetail(item) => {
                self.detail = Some(*item);
                self.show_detail = true;
                true
            }
            FriendShipListMsg::Cancel => {
                self.detail = None;
                self.show_detail = false;
                true
            }
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let content = self
            .list
            .iter()
            .map(|item| {
                let cloned_item = item.clone();
                let onclick = ctx.link().callback(move |_: MouseEvent| {
                    FriendShipListMsg::ShowDetail(Box::new(cloned_item.clone()))
                });

                let mut action = if item.status == FriendStatus::Accepted as i32{
                    html!(<><UpIcon/><span>{tr!(self.i18n, ADDED)}</span></>)
                } else  {
                    html!(<><UpIcon/><span>{tr!(self.i18n, REQUESTED)}</span></>)
                };
                let mut remark = html!(<div class="remark">{tr!(self.i18n, REMARK)}{item.remark.clone()}</div>);
                if !item.is_self {
                    if item.status == FriendStatus::Accepted as i32 {
                        action = html! {
                            <button>{tr!(self.i18n, ADDED)}</button>
                        };
                    } else if item.status == FriendStatus::Pending as i32{
                        action = html! {
                            <button {onclick}>{tr!(self.i18n, GO_VERIFY)}</button>
                        };
                    }
                    remark = html! {
                        <div class="remark">{tr!(self.i18n, APPLY_MSG)}{item.apply_msg.clone()}</div>
                    };
                }
                html! {
                    <div class="friendship-item" /* {onclick} */>
                        <div class="item-left">
                            <img class="avatar" src={utils::get_avatar_url(&item.avatar)} />
                        // </div>
                        <div class="item-info">
                        //     <div class="name-time">
                                <span>{&item.name}</span>
                                // <span class="time">{time_str}</span>
                                {remark}
                            </div>
                        </div>
                        <div class="friendship-action">

                            {action}
                        </div>
                    </div>
                }
            })
            .collect::<Html>();

        let mut detail = html!();
        if self.show_detail {
            let mut remark = AttrValue::default();
            if let Some(friendship) = self.detail.as_ref() {
                if let Some(apply_ms) = &friendship.apply_msg {
                    remark = apply_ms.clone();
                }
            };
            detail = html! {
                <div class="friendship-detail box-shadow" >
                    // 标题
                    <div class="title">
                        {tr!(self.i18n, TITLE)}
                    </div>
                    // 备注
                    <div class="remark">
                        {tr!(self.i18n, REMARK)}
                        <input type="text" ref={self.apply_msg_node.clone()} value={remark} />
                    </div>
                    <div class="response_msg">
                        {tr!(self.i18n, MESSAGE)}
                        <input type="text" ref={self.response_msg_node.clone()} />
                    </div>
                    // 通过验证
                    <div class="agree">
                        <button onclick={ctx.link().callback(|_| FriendShipListMsg::AgreeFriendShip)}>
                            {tr!(self.i18n, ACCEPT)}
                        </button>
                        <button onclick={ctx.link().callback(|_| FriendShipListMsg::Cancel)}>
                            {tr!(self.i18n, CANCEL)}
                        </button>
                    </div>
                </div>
            };
        }
        html! {
            <div class="friendship-list">
                {detail}
                {content}
            </div>
        }
    }
}
