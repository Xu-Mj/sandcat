use std::rc::Rc;

use fluent::{FluentBundle, FluentResource};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, MouseEvent};
use yew::{html, AttrValue, Component, Context, Html, NodeRef, Properties};
use yewdux::Dispatch;

use crate::i18n::{en_us, zh_cn, LanguageType};
use crate::icons::UpIcon;
use crate::model::friend::{Friend, FriendShipAgree, FriendShipWithUser, FriendStatus, ReadStatus};
use crate::model::message::{Message, Msg, SendStatus, DEFAULT_HELLO_MESSAGE};
use crate::model::{ContentType, FriendShipStateType};
use crate::state::{FriendShipState, SendMessageState};
use crate::{api, db, tr, utils};

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
            FriendShipListMsg::QueryFriendships(db::friendships().await.get_list().await)
        });
        let fs_dis = Dispatch::global().subscribe(
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
                        let pos = self
                            .list
                            .iter()
                            .position(|item| item.fs_id == friendship_id);
                        if pos.is_some() {
                            let item = self.list.get_mut(pos.unwrap()).unwrap();
                            item.status = FriendStatus::Accepted as i32;
                            item.read = ReadStatus::True;
                        }
                        // todo should update the database at here
                        Dispatch::<FriendShipState>::global().reduce_mut(|s| {
                            s.friend = Some(*friend.clone());
                            s.ship = None;
                            s.state_type = FriendShipStateType::Res
                        });
                        let send_id = ctx.props().user_id.clone();
                        // update the is_operated field
                        spawn_local(async move {
                            db::friendships().await.agree(friendship_id.as_str()).await;
                            db::friends().await.put_friend(&friend).await;
                            let mut msg = Message {
                                local_id: nanoid::nanoid!().into(),
                                send_id,
                                friend_id: friend.friend_id.clone(),
                                content_type: ContentType::Text,
                                content: friend
                                    .hello
                                    .clone()
                                    .unwrap_or_else(|| AttrValue::from(DEFAULT_HELLO_MESSAGE)),
                                create_time: chrono::Local::now().timestamp_millis(),
                                is_read: true,
                                is_self: true,
                                send_status: SendStatus::Sending,
                                ..Default::default()
                            };
                            let _ = db::messages()
                                .await
                                .add_message(&mut msg)
                                .await
                                .map_err(|err| log::error!("添加好友打招呼消息入库失败:{:?}", err));
                            log::debug!("发送打招呼:{:?}", &msg);
                            Dispatch::<SendMessageState>::global()
                                .reduce_mut(|s| s.msg = Msg::Single(msg));
                        });
                        // ship.status = AttrValue::from("1");
                        // ship.read = ReadStatus::True;
                        // 发送通知给contacts，刷新列表
                    }
                    RequestStatus::Failed(id) => {
                        let pos = self.list.iter().position(|item| item.fs_id == id.clone());
                        if pos.is_some() {
                            let item = self.list.get_mut(pos.unwrap()).unwrap();
                            // 000 标识请求失败
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
                    html!(<><UpIcon/><span>{tr!(self.i18n, "added")}</span></>)
                } else  {
                    html!(<><UpIcon/><span>{tr!(self.i18n, "requested")}</span></>)
                };
                let mut remark = html!(<div class="remark">{tr!(self.i18n, "remark")}{item.remark.clone()}</div>);
                if !item.is_self {
                    if item.status == FriendStatus::Accepted as i32 {
                        action = html! {
                            <button>{tr!(self.i18n, "added")}</button>
                        };
                    } else if item.status == FriendStatus::Pending as i32{
                        action = html! {
                            <button {onclick}>{tr!(self.i18n, "go_verify")}</button>
                        };
                    }
                    remark = html! {
                        <div class="remark">{tr!(self.i18n, "apply_msg")}{item.apply_msg.clone()}</div>
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
                        {tr!(self.i18n, "title")}
                    </div>
                    // 备注
                    <div class="remark">
                        {tr!(self.i18n, "remark")}
                        <input type="text" ref={self.apply_msg_node.clone()} value={remark} />
                    </div>
                    <div class="response_msg">
                        {tr!(self.i18n, "message")}
                        <input type="text" ref={self.response_msg_node.clone()} />
                    </div>
                    // 通过验证
                    <div class="agree">
                        <button onclick={ctx.link().callback(|_| FriendShipListMsg::AgreeFriendShip)}>
                            {tr!(self.i18n, "accept")}
                        </button>
                        <button onclick={ctx.link().callback(|_| FriendShipListMsg::Cancel)}>
                            {tr!(self.i18n, "cancel")}
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
