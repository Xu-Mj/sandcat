use indexmap::IndexMap;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::components::left::add_conv::AddConv;
use crate::components::left::right_click_panel::RightClickPanel;
use crate::db::group::GroupRepo;
use crate::db::group_members::GroupMembersRepo;
use crate::db::message::MessageRepo;
use crate::model::conversation::Conversation;
use crate::model::group::Group;
use crate::model::message::{Msg, SingleCall};
use crate::model::RightContentType;
use crate::pages::{ConvState, CurrentItem, RemoveConvState, UnreadState, WaitState};
use crate::{
    components::{left::list_item::ListItem, top_bar::TopBar},
    db::{conv::ConvRepo, friend::FriendRepo},
    model::ContentType,
    pages::{CommonProps, ComponentType, RecSendMessageState},
};

#[derive(Properties, PartialEq, Debug)]
pub struct MessagesProps {
    pub user_id: AttrValue,
    pub avatar: AttrValue,
}

pub struct Messages {
    list: IndexMap<AttrValue, Conversation>,
    result: IndexMap<AttrValue, Conversation>,
    is_searching: bool,
    query_complete: bool,
    show_friend_list: bool,
    show_context_menu: bool,
    /// hold right click item position and id
    context_menu_pos: (i32, i32, AttrValue),

    msg_state: Rc<RecSendMessageState>,
    _msg_listener: ContextHandle<Rc<RecSendMessageState>>,
    conv_state: Rc<ConvState>,
    _conv_listener: ContextHandle<Rc<ConvState>>,
    _remove_conv_state: Rc<RemoveConvState>,
    _remove_conv_listener: ContextHandle<Rc<RemoveConvState>>,
    unread_state: Rc<UnreadState>,
    _unread_listener: ContextHandle<Rc<UnreadState>>,
    wait_state: Rc<WaitState>,
    _wait_listener: ContextHandle<Rc<WaitState>>,
}

pub enum MessagesMsg {
    FilterContact(AttrValue),
    CleanupSearchResult,
    QueryConvs(IndexMap<AttrValue, Conversation>),
    ReceiveMessage(Rc<RecSendMessageState>),
    InsertConv(Conversation),
    ConvStateChanged(Rc<ConvState>),
    WaitStateChanged,
    AddConv,
    CreateGroup(Group),
    SendBackGroupInvitation(AttrValue),
    ShowContextMenu((i32, i32), AttrValue),
    CloseContextMenu,
    DeleteItem,
    Mute,
    None,
    RemoveConvStateChanged(Rc<RemoveConvState>),
}

impl Component for Messages {
    type Message = MessagesMsg;

    type Properties = MessagesProps;

    fn create(ctx: &Context<Self>) -> Self {
        // query conversation list
        ctx.link().send_future(async {
            let conv_repo = ConvRepo::new().await;
            let convs = conv_repo.get_convs2().await.unwrap_or_default();
            MessagesMsg::QueryConvs(convs)
        });
        // register state
        let (msg_state, _msg_listener) = ctx
            .link()
            .context(ctx.link().callback(MessagesMsg::ReceiveMessage))
            .expect("need conv state in item");
        let (conv_state, _conv_listener) = ctx
            .link()
            .context(ctx.link().callback(MessagesMsg::ConvStateChanged))
            .expect("need state in item");
        let (unread_state, _unread_listener) = ctx
            .link()
            .context(ctx.link().callback(|_| MessagesMsg::None))
            .expect("need state in item");
        let (remove_conv_state, _remove_conv_listener) = ctx
            .link()
            .context(ctx.link().callback(MessagesMsg::RemoveConvStateChanged))
            .expect("need state in item");
        let (wait_state, _wait_listener) = ctx
            .link()
            .context(ctx.link().callback(|_| MessagesMsg::WaitStateChanged))
            .expect("need state in item");
        Self {
            list: IndexMap::new(),
            result: IndexMap::new(),
            query_complete: false,
            is_searching: false,
            show_friend_list: false,
            show_context_menu: false,
            context_menu_pos: (0, 0, AttrValue::default()),
            conv_state,
            _conv_listener,
            _remove_conv_state: remove_conv_state,
            _remove_conv_listener,
            unread_state,
            _unread_listener,
            msg_state,
            _msg_listener,
            wait_state,
            _wait_listener,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            MessagesMsg::FilterContact(pattern) => {
                self.is_searching = true;
                // filter message list
                if pattern.is_empty() {
                    self.result.clear();
                } else {
                    self.list.iter().for_each(|(key, item)| {
                        if item.name.contains(pattern.as_str()) {
                            self.result.insert((*key).clone(), (*item).clone());
                        }
                    });
                }
                true
            }
            MessagesMsg::CleanupSearchResult => {
                self.is_searching = false;
                self.result.clear();
                true
            }
            MessagesMsg::QueryConvs(convs) => {
                self.list = convs;
                self.query_complete = true;
                // 数据查询完成，通知Home组件我已经做完必要的工作了
                self.wait_state.ready.emit(());
                true
            }
            MessagesMsg::ReceiveMessage(state) => {
                let msg = state.msg.clone();
                let conv_type = match msg {
                    Msg::Group(_) => RightContentType::Group,
                    Msg::Single(_) | Msg::SingleCall(_) => RightContentType::Friend,
                    _ => RightContentType::Default,
                };
                match msg {
                    Msg::Single(msg) | Msg::Group(msg) | Msg::OfflineSync(msg) => {
                        let conv = Conversation {
                            last_msg: msg.content.clone(),
                            last_msg_time: msg.create_time,
                            last_msg_type: msg.content_type,
                            conv_type,
                            friend_id: msg.friend_id.clone(),
                            ..Default::default()
                        };
                        self.operate_msg(ctx, msg.friend_id, conv, msg.is_self)
                    }
                    Msg::GroupInvitation(msg) => {
                        // create group conversation directly
                        let clone_ctx = ctx.link().clone();
                        ctx.link().send_future(async move {
                            let conv = Conversation::from(msg.info.clone());
                            ConvRepo::new().await.put_conv(&conv, true).await.unwrap();
                            if let Err(err) = GroupRepo::new().await.put(&msg.info).await {
                                log::error!("store group error : {:?}", err);
                            };
                            if let Err(e) =
                                GroupMembersRepo::new().await.put_list(msg.members).await
                            {
                                log::error!("save group member error: {:?}", e);
                            }
                            // send back received message
                            clone_ctx
                                .send_message(MessagesMsg::SendBackGroupInvitation(msg.info.id));
                            MessagesMsg::InsertConv(conv)
                        });

                        false
                    }
                    Msg::SingleCall(SingleCall::Invite(msg)) => {
                        let friend_id = msg.friend_id.clone();
                        let mut conv = Conversation::from(msg);
                        conv.conv_type = conv_type;
                        self.operate_msg(ctx, friend_id, conv, false)
                    }
                    Msg::SingleCall(SingleCall::InviteCancel(msg)) => {
                        let friend_id = msg.friend_id.clone();
                        let is_self = msg.is_self;
                        let mut conv = Conversation::from(msg);
                        conv.conv_type = conv_type;
                        self.operate_msg(ctx, friend_id, conv, is_self)
                    }
                    Msg::SingleCall(SingleCall::NotAnswer(msg)) => {
                        let friend_id = msg.friend_id.clone();
                        let is_self = msg.is_self;
                        let mut conv = Conversation::from(msg);
                        conv.conv_type = conv_type;
                        self.operate_msg(ctx, friend_id, conv, is_self)
                    }
                    Msg::SingleCall(SingleCall::HangUp(msg)) => {
                        let friend_id = msg.friend_id.clone();
                        let is_self = msg.is_self;
                        let mut conv = Conversation::from(msg);
                        conv.conv_type = conv_type;
                        self.operate_msg(ctx, friend_id, conv, is_self)
                    }
                    Msg::SingleCall(SingleCall::InviteAnswer(msg)) => {
                        let friend_id = msg.friend_id.clone();
                        let is_self = msg.is_self;
                        let mut conv = Conversation::from(msg);
                        conv.conv_type = conv_type;
                        self.operate_msg(ctx, friend_id, conv, is_self)
                    }
                    _ => false,
                }
            }
            MessagesMsg::InsertConv(flag) => {
                // self.list.insert(0, flag);
                self.list.shift_insert(0, flag.friend_id.clone(), flag);
                true
            }
            MessagesMsg::AddConv => {
                self.show_friend_list = !self.show_friend_list;
                true
            }
            MessagesMsg::ConvStateChanged(state) => {
                log::debug!("conv state change: {:?}", state.conv.item_id);
                self.conv_state = state;
                let cur_conv_id = self.conv_state.conv.item_id.clone();
                // 设置了一个查询状态，如果在查询没有完成时更新了状态，那么不进行更新列表，这里有待于优化，
                // 因为状态会在
                if cur_conv_id.is_empty() || !self.query_complete {
                    return false;
                }
                // log::debug!("in update app state changed: {:?} ; id: {}", self.list.clone(), self.app_state.current_conv_id);
                // 判断是否需要更新当前会话
                let dest = self.list.get_mut(&cur_conv_id);
                if dest.is_some() {
                    let conv = dest.unwrap();
                    conv.unread_count = 0;
                    // self.list.shift_insert(index, cur_conv_id, conv.clone());
                    let conv = conv.clone();
                    spawn_local(async move {
                        let conv_repo = ConvRepo::new().await;
                        conv_repo.put_conv(&conv, true).await.unwrap();
                    });
                    true
                } else {
                    // 不存在，那么创建
                    let friend_id = cur_conv_id.clone();
                    let conv_type = self.conv_state.conv.content_type.clone();
                    log::debug!("conv type in messages: {:?}", conv_type.clone());
                    ctx.link().send_future(async move {
                        let friend_repo = FriendRepo::new().await;
                        let friend = friend_repo.get_friend(friend_id.clone()).await;
                        // todo查询上一条消息
                        let msg_repo = MessageRepo::new().await;
                        let result = msg_repo
                            .get_last_msg(friend_id.clone())
                            .await
                            .unwrap_or_default();
                        let content = if result.id != 0 {
                            get_msg_type(result.content_type, &result.content)
                        } else {
                            AttrValue::default()
                        };
                        let conv = Conversation {
                            id: 0,
                            name: friend.name,
                            avatar: friend.avatar,
                            last_msg: content,
                            last_msg_time: result.create_time,
                            last_msg_type: result.content_type,
                            unread_count: 0,
                            friend_id,
                            conv_type,
                            mute: false,
                        };
                        let conv_repo = ConvRepo::new().await;
                        conv_repo.put_conv(&conv, true).await.unwrap();
                        log::debug!("状态更新，不存在的会话，添加数据: {:?}", &conv);
                        MessagesMsg::InsertConv(conv)
                    });
                    false
                }
            }
            MessagesMsg::WaitStateChanged => false,
            MessagesMsg::CreateGroup(g) => {
                self.show_friend_list = false;
                // create group conversation and send 'create group' message
                ctx.link().send_future(async move {
                    let conv = Conversation::from(g);
                    ConvRepo::new().await.put_conv(&conv, true).await.unwrap();
                    MessagesMsg::InsertConv(conv)
                });
                false
            }
            MessagesMsg::SendBackGroupInvitation(group_id) => {
                self.msg_state
                    .send_back_event
                    .emit(Msg::GroupInvitationReceived((
                        ctx.props().user_id.to_string(),
                        group_id.to_string(),
                    )));
                false
            }
            MessagesMsg::ShowContextMenu((x, y), id) => {
                // event.prevent_default();
                self.context_menu_pos = (x, y, id);
                self.show_context_menu = true;
                true
            }
            MessagesMsg::CloseContextMenu => {
                log::debug!("close context menu");
                self.show_context_menu = false;
                self.context_menu_pos = (0, 0, AttrValue::default());
                true
            }
            MessagesMsg::DeleteItem => {
                // delete database data
                let id = self.context_menu_pos.2.clone();
                spawn_local(async move {
                    if let Err(e) = ConvRepo::new().await.delete(id).await {
                        log::error!("delete conversation error: {:?}", e);
                    }
                });
                if let Some(conv) = self.list.shift_remove(self.context_menu_pos.2.as_str()) {
                    if conv.unread_count > 0 {
                        self.unread_state.sub_msg_count.emit(conv.unread_count);
                    }
                }
                self.show_context_menu = false;
                self.context_menu_pos = (0, 0, AttrValue::default());
                // set right content type
                let mut conv = self.conv_state.conv.clone();
                conv.item_id = AttrValue::default();
                conv.content_type = RightContentType::Default;
                self.conv_state.state_change_event.emit(conv);
                true
            }
            MessagesMsg::None => false,
            MessagesMsg::RemoveConvStateChanged(state) => {
                // delete conversation from database should be here
                if let Some(conv) = self.list.shift_remove(state.id.as_str()) {
                    if conv.unread_count > 0 {
                        self.unread_state.sub_msg_count.emit(conv.unread_count);
                    }
                    if conv.friend_id == self.conv_state.conv.item_id {
                        self.conv_state
                            .state_change_event
                            .emit(CurrentItem::default());
                    }
                };
                true
            }
            MessagesMsg::Mute => {
                if let Some(conv) = self.list.get_mut(&self.context_menu_pos.2) {
                    conv.mute = true;
                    let conv = conv.clone();
                    spawn_local(async move {
                        if let Err(e) = ConvRepo::new().await.mute(&conv).await {
                            log::error!("mute conversation err: {:?}", e);
                        }
                    });
                    return true;
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let oncontextmenu = ctx
            .link()
            .callback(|((x, y), id)| MessagesMsg::ShowContextMenu((x, y), id));
        let content = if self.is_searching {
            if self.result.is_empty() {
                html! {<div class="no-result">{"没有搜索结果"}</div>}
            } else {
                self.result
                    .iter()
                    .map(|(_id, item)| {
                        let remark = get_msg_type(item.last_msg_type, &item.last_msg);
                        html! {
                                <ListItem
                                    component_type={ComponentType::Messages}
                                    props={CommonProps{name:item.name.clone().into(),
                                        avatar:item.avatar.clone().into(),
                                        time:item.last_msg_time,
                                        remark,
                                        id: item.friend_id.clone() }}
                                    unread_count={item.unread_count}
                                    conv_type={item.conv_type.clone()}
                                    oncontextmenu={oncontextmenu.clone()}
                                    mute={item.mute}
                                    key={item.friend_id.clone().as_str()} />
                        }
                    })
                    .collect::<Html>()
            }
        } else {
            self.list
                .iter()
                // .map(|item| {
                .map(|(_friend_id, item)| {
                    let remark = get_msg_type(item.last_msg_type, &item.last_msg);
                    html! {
                            <ListItem
                                component_type={ComponentType::Messages}
                                props={CommonProps{name:item.name.clone().into(),
                                    avatar:item.avatar.clone().into(),
                                    time:item.last_msg_time,
                                    remark,
                                    id: item.friend_id.clone() }}
                                unread_count={item.unread_count}
                                oncontextmenu={oncontextmenu.clone()}
                                conv_type={item.conv_type.clone()}
                                mute={item.mute}
                                key={item.friend_id.clone().as_str()} />
                    }
                })
                .collect::<Html>()
        };
        let search_callback = ctx.link().callback(MessagesMsg::FilterContact);
        let clean_callback = ctx
            .link()
            .callback(move |_| MessagesMsg::CleanupSearchResult);
        let plus_click = ctx.link().callback(|_| MessagesMsg::AddConv);
        let submit_back = ctx.link().callback(MessagesMsg::CreateGroup);

        // spawn friend list
        let mut friend_list = html!();
        if self.show_friend_list {
            friend_list = html! {
                <AddConv
                    user_id={ctx.props().user_id.clone()}
                    close_back={plus_click.clone()} {submit_back}
                    avatar={ctx.props().avatar.clone()}
                    />
            };
        }
        let mut context_menu = html!();
        if self.show_context_menu {
            context_menu = html! {
                <RightClickPanel
                    x={self.context_menu_pos.0}
                    y={self.context_menu_pos.1}
                    close={ctx.link().callback( |_|MessagesMsg::CloseContextMenu)}
                    mute={ctx.link().callback(|_| MessagesMsg::Mute)}
                    delete={ctx.link().callback(|_|MessagesMsg::DeleteItem)}/>
            }
        }
        html! {
            <div class="list-wrapper">
                {context_menu}
                {friend_list}
                <TopBar components_type={ComponentType::Messages} {search_callback} {clean_callback} {plus_click}/>
                <div class="contacts-list">
                    {content}
                </div>
            </div>
        }
    }
}

fn get_msg_type(msg_type: ContentType, content: &AttrValue) -> AttrValue {
    match msg_type {
        ContentType::Text => content.clone(),
        ContentType::Image => AttrValue::from("[图片]"),
        ContentType::Video => AttrValue::from("[视频]"),
        ContentType::File => AttrValue::from("[文件]"),
        ContentType::Emoji => AttrValue::from("[表情]"),
        ContentType::Default => AttrValue::from(""),
        ContentType::VideoCall => AttrValue::from("[视频通话]"),
        ContentType::AudioCall => AttrValue::from("[语音通话]"),
        ContentType::Audio => AttrValue::from("[voice]"),
    }
}

impl Messages {
    fn operate_msg(
        &mut self,
        ctx: &Context<Self>,
        friend_id: AttrValue,
        mut conv: Conversation,
        is_self: bool,
    ) -> bool {
        let dest = self.list.shift_remove(&friend_id);
        let mut clean = false;
        if dest.is_some() {
            let mut old = dest.unwrap();
            // deal with unread message count
            if !old.mute && !is_self && self.conv_state.conv.item_id != friend_id {
                self.unread_state.add_msg_count.emit(());
            }
            // 这里是因为要直接更新面板上的数据，所以需要处理未读数量
            if friend_id != self.conv_state.conv.item_id {
                old.unread_count += 1;
            } else {
                old.unread_count = 0;
                clean = true;
            }
            conv.name = old.name;
            conv.avatar = old.avatar;
            conv.id = old.id;
            conv.unread_count = old.unread_count;
            conv.mute = old.mute;
            self.list.shift_insert(0, friend_id, conv.clone());
            spawn_local(async move {
                let conv_repo = ConvRepo::new().await;
                conv_repo.put_conv(&conv, clean).await.unwrap();
            });
            true
        } else {
            if !is_self && self.conv_state.conv.item_id != friend_id {
                self.unread_state.add_msg_count.emit(());
            }
            // 如果会话列表中不存在那么需要新建
            ctx.link().send_future(async move {
                let friend_repo = FriendRepo::new().await;
                let friend = friend_repo.get_friend(friend_id).await;
                conv.avatar = friend.avatar;
                if let Some(name) = friend.remark {
                    conv.name = name;
                } else {
                    conv.name = friend.name;
                }
                let conv_repo = ConvRepo::new().await;
                conv_repo.put_conv(&conv, false).await.unwrap();
                conv.unread_count = 1;
                log::debug!("创建会话: {:?}", &conv);
                MessagesMsg::InsertConv(conv)
            });
            false
        }
    }
}
