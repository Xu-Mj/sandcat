use std::rc::Rc;

use indexmap::IndexMap;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;
use yew::prelude::*;
use yewdux::Dispatch;

use abi::model::friend::FriendStatus;
use abi::model::message::GroupMsg;
use abi::model::message::Message;
use abi::model::message::Msg;
use abi::model::message::SingleCall;
use abi::model::ItemInfo;
use abi::model::RightContentType;
use abi::state::OfflineMsgState;
use abi::state::RecMessageState;
use abi::state::SendResultState;
use i18n::LanguageType;

use crate::right::{msg_item::MsgItem, sender::Sender};

pub struct MessageList {
    list2: IndexMap<AttrValue, Message>,
    node_ref: NodeRef,
    page: u32,
    page_size: u32,
    is_all: bool,
    scroll_state: ScrollState,
    friend: Option<Box<dyn ItemInfo>>,
    new_msg_count: u32,
    is_black: bool,

    // listen sync offline message, query message list
    _sync_msg_dis: Dispatch<OfflineMsgState>,
    // listen rec message, update message list
    _rec_msg_dis: Dispatch<RecMessageState>,
    // listen send result, update message item status
    _send_result_dis: Dispatch<SendResultState>,
}

pub enum MessageListMsg {
    QueryMsgList(IndexMap<AttrValue, Message>),
    NextPage,
    NextPageNone,
    SendFile(Message),
    ReceiveMsg(Rc<RecMessageState>),
    SendResultCallback(Rc<SendResultState>),
    SyncOfflineMsg,
    GoBottom,
    QueryFriend(Option<Box<dyn ItemInfo>>),
}

/// 接收对方用户信息即可，
/// 根据对方的id查询数据库消息记录
/// 如果存在未读消息，则清空未读消息
/// 清空步骤，向服务器发送已读消息请求
/// 服务器收到请求后，更新数据库消息记录
///
/// 直接将必要|的信息都传递进来就行，省略一次数据库查询
#[derive(Properties, Clone, PartialEq, Debug)]
pub struct MessageListProps {
    pub friend_id: AttrValue,
    pub conv_type: RightContentType,
    pub cur_user_avatar: AttrValue,
    pub cur_user_id: AttrValue,
    pub lang: LanguageType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollState {
    None,
    Bottom,
}

impl MessageList {
    fn query(&self, ctx: &Context<Self>) {
        let id = ctx.props().friend_id.clone();
        log::debug!("message list props id: {} in query method", id);
        if !id.is_empty() {
            // 查询数据库
            let id = id.clone();
            let page = self.page;
            let page_size = self.page_size;
            let conv_type = ctx.props().conv_type.clone();
            log::debug!("props conv type :{:?}", conv_type);

            ctx.link().send_future(async move {
                match conv_type {
                    RightContentType::Friend => {
                        let list = db::messages()
                            .await
                            .get_messages(id.as_str(), page, page_size)
                            .await
                            .unwrap();
                        MessageListMsg::QueryMsgList(list)
                    }
                    RightContentType::Group => {
                        let list = db::group_msgs()
                            .await
                            .get_messages(id.as_str(), page, page_size)
                            .await
                            .unwrap();
                        MessageListMsg::QueryMsgList(list)
                    }
                    _ => {
                        todo!()
                    }
                }
            });
        }
    }

    fn query_friend(&self, ctx: &Context<Self>) {
        let id = ctx.props().friend_id.clone();
        log::debug!("message list props id: {} in query method", id.clone());
        if !id.is_empty() {
            // 查询数据库
            let conv_type = ctx.props().conv_type.clone();

            ctx.link().send_future(async move {
                // 查询朋友信息
                let mut friend: Option<Box<dyn ItemInfo>> = None;
                match conv_type {
                    RightContentType::Friend => {
                        friend = Some(Box::new(db::friends().await.get(id.as_str()).await));
                    }
                    RightContentType::Group => {
                        friend = Some(Box::new(
                            db::groups().await.get(id.as_str()).await.unwrap().unwrap(),
                        ));
                    }
                    _ => {}
                }
                MessageListMsg::QueryFriend(friend)
            });
        }
    }

    fn reset(&mut self) {
        self.list2 = IndexMap::new();
        self.page = 1;
        self.page_size = 20;
        self.new_msg_count = 0;
        self.is_all = false;
        self.scroll_state = ScrollState::None;
    }

    fn insert_msg(&mut self, msg: Message, friend_id: AttrValue) -> bool {
        if msg.friend_id == friend_id {
            let is_self = msg.is_self;
            self.list2.shift_insert(0, msg.local_id.clone(), msg);
            if is_self || self.scroll_state == ScrollState::Bottom {
                self.new_msg_count = 0;
                self.scroll_state = ScrollState::Bottom;
            } else {
                // 如果消息列表没有拉到最下面，那么加一
                self.new_msg_count += 1;
                self.scroll_state = ScrollState::None;
            }
            true
        } else {
            false
        }
    }
    fn handle_rec_msg(&mut self, ctx: &Context<Self>, msg: Msg, friend_id: AttrValue) -> bool {
        log::debug!("handle_rec_msg: {:?}", msg);
        match msg {
            Msg::Single(msg) => self.insert_msg(msg, friend_id),
            Msg::Group(msg) => {
                match msg {
                    GroupMsg::Message(msg) => self.insert_msg(msg, friend_id),
                    // need to handle, as system notify
                    GroupMsg::MemberExit(_) => false,
                    GroupMsg::Dismiss((group_id, _)) => {
                        if group_id == friend_id {
                            self.is_black = true;
                            return true;
                        }
                        false
                    }
                    _ => false,
                }
            }

            Msg::SingleCall(m) => match m {
                SingleCall::InviteCancel(msg) => self.insert_msg(msg.into(), friend_id),
                SingleCall::InviteAnswer(msg) => {
                    if !msg.agree {
                        return self.insert_msg(msg.into(), friend_id);
                    }
                    false
                }
                SingleCall::HangUp(msg) => self.insert_msg(Message::from_hangup(msg), friend_id),
                SingleCall::NotAnswer(msg) => {
                    self.insert_msg(Message::from_not_answer(msg), friend_id)
                }
                _ => false,
            },

            Msg::SendRelationshipReq(_)
            | Msg::RecRelationship(_)
            | Msg::ReadNotice(_)
            | Msg::SingleDeliveredNotice(_)
            | Msg::OfflineSync(_)
            | Msg::RelationshipRes(_)
            | Msg::FriendshipDeliveredNotice(_) => false,
            // todo query list item , update state
            Msg::ServerRecResp(_) => false,
            Msg::RecRelationshipDel((friend_id, _)) => {
                log::debug!(
                    "rec friendship del in msg list {}, ctx friend id {:?}",
                    friend_id,
                    ctx.props().friend_id
                );
                // judge if friend_id is current user
                if friend_id == ctx.props().friend_id {
                    self.is_black = true;
                    return true;
                }
                false
            }
        }
    }
}

impl Component for MessageList {
    type Message = MessageListMsg;
    type Properties = MessageListProps;

    fn create(ctx: &Context<Self>) -> Self {
        let _sync_msg_dis = Dispatch::global()
            .subscribe_silent(ctx.link().callback(|_| MessageListMsg::SyncOfflineMsg));
        let _rec_msg_dis =
            Dispatch::global().subscribe_silent(ctx.link().callback(MessageListMsg::ReceiveMsg));
        let _send_result_dis = Dispatch::global()
            .subscribe_silent(ctx.link().callback(MessageListMsg::SendResultCallback));
        let self_ = Self {
            // list: vec![],
            node_ref: NodeRef::default(),
            page_size: 20,
            page: 1,
            is_all: false,
            scroll_state: ScrollState::Bottom,
            friend: None,
            new_msg_count: 0,
            is_black: false,
            _sync_msg_dis,
            _rec_msg_dis,
            _send_result_dis,
            list2: IndexMap::new(),
        };
        self_.query_friend(ctx);
        self_.query(ctx);
        self_
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if self.scroll_state == ScrollState::Bottom {
            let node = self.node_ref.cast::<HtmlElement>().unwrap();
            node.set_scroll_top(0);
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.reset();
        self.query_friend(ctx);
        self.query(ctx);
        // 这里不能让组件重新绘制，
        // 因为query方法会触发组件的渲染，
        // 重复渲染会导致页面出现难以预料的问题
        false
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let friend_id = ctx.props().friend_id.clone();
        match msg {
            MessageListMsg::QueryMsgList(list) => {
                log::debug!("message list update: {}", list.len());
                // 判断是否是最后一页，优化查询次数
                if list.len() < self.page_size as usize {
                    self.is_all = true;
                }
                self.list2.extend(list);
                true
            }
            MessageListMsg::NextPage => {
                log::debug!("next page");
                self.page += 1;
                self.scroll_state = ScrollState::None;
                self.query(ctx);
                false
            }
            MessageListMsg::NextPageNone => false,
            MessageListMsg::SendFile(msg) => {
                self.list2.insert(msg.local_id.clone(), msg);
                self.scroll_state = ScrollState::Bottom;
                true
            }
            MessageListMsg::ReceiveMsg(msg_state) => {
                log::debug!("rec message in message list....");
                let msg = msg_state.msg.clone();
                self.handle_rec_msg(ctx, msg, friend_id)
            }
            MessageListMsg::GoBottom => {
                self.new_msg_count = 0;
                self.scroll_state = ScrollState::Bottom;
                true
            }
            MessageListMsg::QueryFriend(item) => {
                if let Some(item) = item.as_ref() {
                    self.is_black = item.status() == FriendStatus::Delete;
                }
                self.friend = item;
                true
            }
            MessageListMsg::SyncOfflineMsg => {
                log::debug!("sync offline msg in message list....");
                self.reset();
                self.query(ctx);
                false
            }
            MessageListMsg::SendResultCallback(state) => {
                let msg = state.msg.clone();
                /*  let mut result = self.list.iter_mut().filter(|v| *v.local_id == msg.local_id);
                result.next().map(|v| {
                    v.send_status = msg.send_status;
                }); */
                if let Some(v) = self.list2.get_mut(&msg.local_id) {
                    v.send_status = msg.send_status;
                }
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let props = ctx.props();
        let is_all = self.is_all;
        let onscroll = ctx.link().callback(move |event: Event| {
            let node: HtmlElement = event.target().unwrap().dyn_into().unwrap();
            let height = node.client_height() - node.scroll_height() + 1;
            // 判断是否滑动到顶部，
            if node.scroll_top() == height && !is_all {
                // 翻页
                return MessageListMsg::NextPage;
            }
            MessageListMsg::NextPageNone
        });
        let on_file_send = ctx.link().callback(MessageListMsg::SendFile);

        // 未读消息数量
        let new_msg_count = if self.scroll_state != ScrollState::Bottom && self.new_msg_count > 0 {
            let onclick = ctx.link().callback(|_| MessageListMsg::GoBottom);
            html! {
                <div class="msg-list-new-msg-count" {onclick} >
                    {self.new_msg_count}
                </div>
            }
        } else {
            html!()
        };

        let mut list = html!();
        if let Some(friend) = self.friend.as_ref() {
            let conv_type = props.conv_type.clone();
            list = self
                .list2
                .iter()
                .map(|(_, msg)| {
                    let mut avatar = friend.avatar().clone();
                    if msg.is_self {
                        avatar = props.cur_user_avatar.clone();
                    }
                    html! {
                        <MsgItem
                            user_id={props.cur_user_id.clone()}
                            friend_id={props.friend_id.clone()}
                            msg={msg.clone()}
                            avatar={avatar}
                            conv_type={conv_type.clone()}
                            key={msg.id}
                        />
                    }
                })
                .collect::<Html>()
        }
        html! {
            <>
                <div class="resize">
                    {new_msg_count}
                    <div ref={self.node_ref.clone()} class="msg-list"  {onscroll}>
                        {list}
                    </div>
                </div>
                <Sender
                    friend_id={props.friend_id.clone()}
                    cur_user_id={props.cur_user_id.clone()}
                    conv_type={ctx.props().conv_type.clone()}
                    disable = {self.is_black}
                    {on_file_send}
                    lang={ctx.props().lang}
                />
            </>
        }
    }
}
