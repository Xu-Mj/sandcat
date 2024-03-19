use std::rc::Rc;

use web_sys::{HtmlInputElement, MouseEvent};
use yew::{html, AttrValue, Component, Context, ContextHandle, Html, NodeRef};

use crate::model::friend::{Friend, FriendShipAgree, FriendShipWithUser, FriendStatus, ReadStatus};
use crate::model::FriendShipStateType;
use crate::pages::FriendShipState;
use crate::{api, db};

pub struct FriendShipList {
    list: Vec<FriendShipWithUser>,
    friendship_state: Rc<FriendShipState>,
    _listener: ContextHandle<Rc<FriendShipState>>,
    show_detail: bool,
    detail: Option<FriendShipWithUser>,
    apply_msg_node: NodeRef,
    response_msg_node: NodeRef,
}

pub enum FriendShipListMsg {
    FriendShipStateChanged(Rc<FriendShipState>),
    QueryFriendships(Vec<FriendShipWithUser>),
    AgreeFriendShip,
    AgreeFriendShipRes(RequestStatus),
    ShowDetail(FriendShipWithUser),
    Cancel,
}

pub enum RequestStatus {
    Failed(AttrValue),
    Success(AttrValue, Box<Friend>),
    // Pending,
}

impl Component for FriendShipList {
    type Message = FriendShipListMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async {
            FriendShipListMsg::QueryFriendships(db::friendships().await.get_list().await)
        });
        let (friendship_state, _listener) = ctx
            .link()
            .context(
                ctx.link()
                    .callback(FriendShipListMsg::FriendShipStateChanged),
            )
            .expect("need friend ship state");
        Self {
            list: vec![],
            friendship_state,
            _listener,
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
                    item.status = FriendStatus::Accepted;

                    let remark = self.apply_msg_node.cast::<HtmlInputElement>().unwrap();
                    let response_msg = self.response_msg_node.cast::<HtmlInputElement>().unwrap();
                    let remark = if remark.value().is_empty() {
                        None
                    } else {
                        Some(remark.value())
                    };
                    let response_msg = if response_msg.value().is_empty() {
                        None
                    } else {
                        Some(response_msg.value())
                    };
                    let friendship_req = FriendShipAgree {
                        friendship_id: item.friendship_id.clone(),
                        response_msg,
                        remark,
                    };
                    let friendship_id = item.friendship_id.clone();
                    // 发送好友同意请求
                    ctx.link().send_future(async move {
                        match api::friends().agree_friend(friendship_req).await {
                            Ok(res) => {
                                log::debug!("好友请求成功:{:?}", &res);
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
                true
            }
            FriendShipListMsg::AgreeFriendShipRes(res) => {
                match res {
                    RequestStatus::Success(friendship_id, friend) => {
                        let pos = self
                            .list
                            .iter()
                            .position(|item| item.friendship_id == friendship_id);
                        if pos.is_some() {
                            let item = self.list.get_mut(pos.unwrap()).unwrap();
                            item.status = FriendStatus::Accepted;
                            item.read = ReadStatus::True;
                        }
                        self.friendship_state
                            .res_change_event
                            .emit((friendship_id, *friend));
                        // ship.status = AttrValue::from("1");
                        // ship.read = ReadStatus::True;
                        // 发送通知给contacts，刷新列表
                    }
                    RequestStatus::Failed(id) => {
                        let pos = self
                            .list
                            .iter()
                            .position(|item| item.friendship_id == id.clone());
                        if pos.is_some() {
                            let item = self.list.get_mut(pos.unwrap()).unwrap();
                            // 000 标识请求失败
                            item.status = FriendStatus::Failed;
                        }
                    }
                }
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
                        true
                    }
                    FriendShipStateType::Res => false,
                }
            }
            FriendShipListMsg::ShowDetail(item) => {
                self.detail = Some(item);
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
                    FriendShipListMsg::ShowDetail(cloned_item.clone())
                });

                let mut action = html!(<span>{"已请求"}</span>);
                if !item.is_self {
                    action = html! {
                        <button {onclick}>{"前往验证"}</button>
                    };
                }
                html! {
                    <div class="friendship-item" /* {onclick} */>
                        <div class="item-left">
                            <img class="avatar" src={item.avatar.clone()} />
                        // </div>
                        <div class="item-info">
                        //     <div class="name-time">
                                <span>{item.name.clone()}</span>
                                // <span class="time">{time_str}</span>
                                <div class="remark">{"备注:\t"}{item.apply_msg.clone()}</div>
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
                        {"验证好友请求"}
                    </div>
                    // 备注
                    <div class="remark">
                        {"备注:\t"}
                        <input type="text" ref={self.apply_msg_node.clone()} value={remark} />
                    </div>
                    <div class="response_msg">
                        {"验证信息:\t"}
                        <input type="text" ref={self.response_msg_node.clone()} />
                    </div>
                    // 通过验证
                    <div class="agree">
                        <button onclick={ctx.link().callback(|_| FriendShipListMsg::AgreeFriendShip)}>
                            {"通过验证"}
                        </button>
                        <button onclick={ctx.link().callback(|_| FriendShipListMsg::Cancel)}>
                            {"取消"}
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
