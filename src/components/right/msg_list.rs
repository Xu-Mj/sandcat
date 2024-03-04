use crate::model::friend::ItemInfo;
use crate::model::message::Msg;
use crate::model::RightContentType;
use crate::{
    components::right::{msg_item::MsgItem, sender::Sender},
    db::{friend::FriendRepo, message::MessageRepo},
    model::message::Message,
    pages::RecSendMessageState,
};
use std::rc::Rc;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlElement;
use yew::prelude::*;

pub struct MessageList {
    list: Vec<Message>,
    node_ref: NodeRef,
    page: u32,
    page_size: u32,
    is_all: bool,
    scroll_state: ScrollState,
    // 监听消息接收状态，用来更新当前对话框消息列表
    _msg_state: Rc<RecSendMessageState>,
    _listener: ContextHandle<Rc<RecSendMessageState>>,
    friend: Option<Box<dyn ItemInfo>>,
    new_msg_count: u32,
}

pub enum MessageListMsg {
    QueryMsgList(Vec<Message>),
    NextPage,
    NextPageNone,
    SendFile(Message),
    ReceiveMsg(Rc<RecSendMessageState>),
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
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollState {
    None,
    Bottom,
}

impl MessageList {
    fn query(&self, ctx: &Context<Self>) {
        let id = ctx.props().friend_id.clone();
        log::debug!("message list props id: {} in query method", id.clone());
        if id != AttrValue::default() {
            // 查询数据库
            let id = id.clone();
            let page = self.page;
            let page_size = self.page_size;
            let conv_type = ctx.props().conv_type.clone();
            log::debug!("props conv type :{:?}", conv_type);
            ctx.link().send_future(async move {
                let msg_repo = MessageRepo::new().await;
                let list = msg_repo
                    .get_messages(id.clone(), page, page_size)
                    .await
                    .unwrap();
                MessageListMsg::QueryMsgList(list)
            });
        }
    }
    fn query_friend(&self, ctx: &Context<Self>) {
        let id = ctx.props().friend_id.clone();
        log::debug!("message list props id: {} in query method", id.clone());
        if id != AttrValue::default() {
            // 查询数据库
            let id = id.clone();
            let conv_type = ctx.props().conv_type.clone();

            ctx.link().send_future(async move {
                // 查询朋友信息
                let mut friend: Option<Box<dyn ItemInfo>> = None;
                match conv_type {
                    RightContentType::Friend => {
                        friend = Some(Box::new(FriendRepo::new().await.get_friend(id).await));
                    }
                    RightContentType::Group => {}
                    _ => {}
                }
                MessageListMsg::QueryFriend(friend)
            });
        }
    }
    fn reset(&mut self) {
        self.list = vec![];
        self.page = 1;
        self.page_size = 20;
        self.is_all = false;
        self.scroll_state = ScrollState::None;
    }

    fn insert_msg(&mut self, msg: Message, friend_id: AttrValue) -> bool {
        if msg.friend_id == friend_id {
            let is_self = msg.is_self;
            self.list.insert(0, msg);
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
}

impl Component for MessageList {
    type Message = MessageListMsg;
    type Properties = MessageListProps;

    fn create(ctx: &Context<Self>) -> Self {
        let (_msg_state, _listener) = ctx
            .link()
            .context(ctx.link().callback(MessageListMsg::ReceiveMsg))
            .expect("need msg context");
        let self_ = Self {
            list: vec![],
            node_ref: NodeRef::default(),
            page_size: 20,
            page: 1,
            is_all: false,
            scroll_state: ScrollState::Bottom,
            _msg_state,
            _listener,
            friend: None,
            new_msg_count: 0,
        };
        self_.query_friend(ctx);
        self_.query(ctx);
        self_
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        // if first_render {
        //     self.query(ctx);
        // }
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
            MessageListMsg::QueryMsgList(mut list) => {
                gloo::console::log!("message list update", JsValue::from(list.len()));
                // 判断是否是最后一页，优化查询次数
                if list.len() < self.page_size as usize {
                    self.is_all = true;
                }
                self.list.append(&mut list);
                true
            }
            MessageListMsg::NextPage => {
                gloo::console::log!("next page");
                self.page += 1;
                self.scroll_state = ScrollState::None;
                self.query(ctx);
                false
            }
            MessageListMsg::NextPageNone => false,
            MessageListMsg::SendFile(msg) => {
                self.list.insert(0, msg);
                self.scroll_state = ScrollState::Bottom;
                true
            }
            MessageListMsg::ReceiveMsg(msg_state) => {
                let msg = msg_state.msg.clone();
                match msg {
                    Msg::Single(msg) | Msg::Group(msg) => self.insert_msg(msg, friend_id),

                    Msg::SingleCallInviteCancel(msg) => self.insert_msg(msg.into(), friend_id),
                    Msg::SingleCallInviteAnswer(msg) => {
                        if !msg.agree {
                            return self.insert_msg(msg.into(), friend_id);
                        }
                        false
                    }

                    Msg::SingleCallHangUp(msg) => {
                        self.insert_msg(Message::from_hangup(msg), friend_id)
                    }
                    Msg::SingleCallNotAnswer(msg) => {
                        self.insert_msg(Message::from_not_answer(msg), friend_id)
                    }
                    Msg::SingleCallOffer(_)
                    | Msg::SingleCallInvite(_)
                    | Msg::SingleCallAgree(_)
                    | Msg::SendRelationshipReq(_)
                    | Msg::RecRelationship(_)
                    | Msg::ReadNotice(_)
                    | Msg::SingleDeliveredNotice(_)
                    | Msg::OfflineSync(_)
                    | Msg::NewIceCandidate(_)
                    | Msg::RelationshipRes(_)
                    | Msg::FriendshipDeliveredNotice(_) => false,
                }
            }
            MessageListMsg::GoBottom => {
                self.new_msg_count = 0;
                self.scroll_state = ScrollState::Bottom;
                true
            }
            MessageListMsg::QueryFriend(item) => {
                self.friend = item;
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
        html! {
            <>
                <div class="resize">
                    {new_msg_count}
                    <div ref={self.node_ref.clone()} class="msg-list"  {onscroll}>
                        {
                            if let Some(friend) = self.friend.as_ref() {
                                self.list.iter().map(|msg| {
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
                                            key={msg.id}
                                        />
                                    }
                                }).collect::<Html>()
                        }else {
                            html!()
                        }
                        }
                    </div>
                </div>
                <Sender
                    friend_id={props.friend_id.clone()}
                    cur_user_id={props.cur_user_id.clone()}
                    conv_type={ctx.props().conv_type.clone()}
                    {on_file_send}
                />
            </>
        }
    }
}
