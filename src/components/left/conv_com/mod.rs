mod conversations;

use std::{cell::RefCell, collections::HashMap, rc::Rc};

use gloo::utils::window;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use indexmap::IndexMap;

use crate::{
    api,
    components::left::list_item::ListItem,
    db::{self, TOKEN, WS_ADDR},
    model::{
        conversation::{get_invite_type, Conversation},
        friend::FriendShipWithUser,
        group::{GroupMember, GroupRequest},
        message::{
            convert_server_msg, GroupInvitation, GroupMsg, Message, Msg, SingleCall,
            DEFAULT_HELLO_MESSAGE,
        },
        seq::Seq,
        CommonProps, ComponentType, ContentType, RightContentType,
    },
    pages::{
        AddFriendState, ConvState, CreateConvState, MuteState, OfflineMsgState, RecMessageState,
        RemoveConvState, SendMessageState, UnreadState, WaitState,
    },
    pb::message::Msg as PbMsg,
    ws::WebSocketManager,
};

use self::conversations::ChatsMsg;

pub struct Chats {
    call_msg: SingleCall,
    ws: Rc<RefCell<WebSocketManager>>,
    seq: Seq,
    list: IndexMap<AttrValue, Conversation>,
    result: IndexMap<AttrValue, Conversation>,
    is_searching: bool,
    query_complete: bool,
    show_friend_list: bool,
    show_context_menu: bool,
    /// hold right click item position and id
    context_menu_pos: (i32, i32, AttrValue, bool),

    /// receive the message state from sender
    _msg_state: Rc<SendMessageState>,
    _msg_listener: ContextHandle<Rc<SendMessageState>>,
    /// listen the conversation change
    conv_state: Rc<ConvState>,
    _conv_listener: ContextHandle<Rc<ConvState>>,
    _remove_conv_state: Rc<RemoveConvState>,
    _remove_conv_listener: ContextHandle<Rc<RemoveConvState>>,
    unread_state: Rc<UnreadState>,
    _unread_listener: ContextHandle<Rc<UnreadState>>,
    wait_state: Rc<WaitState>,
    _wait_listener: ContextHandle<Rc<WaitState>>,
    _create_conv: Rc<CreateConvState>,
    _create_conv_listener: ContextHandle<Rc<CreateConvState>>,
    _mute_state: Rc<MuteState>,
    _mute_state_listener: ContextHandle<Rc<MuteState>>,
    _add_friend_state: Rc<AddFriendState>,
    _add_friend_state_listener: ContextHandle<Rc<AddFriendState>>,
    sync_msg_state: Rc<OfflineMsgState>,
    _sync_msg_state_listener: ContextHandle<Rc<OfflineMsgState>>,
    rec_msg_state: Rc<RecMessageState>,
    _rec_msg_state_listener: ContextHandle<Rc<RecMessageState>>,
}

impl Chats {
    fn new(ctx: &Context<Self>) -> Self {
        let id = ctx.props().user_id.clone();
        // query conversation list
        let user_id = id.clone();
        ctx.link().send_future(async move {
            let convs = db::convs().await.get_convs2().await.unwrap_or_default();
            // pull offline messages
            let seq_repo = db::seq().await;
            let mut local_seq = seq_repo.get().await.unwrap_or_default();
            let mut messages = Vec::new();
            log::debug!("seq: {:?}", local_seq);
            if local_seq.local_seq < local_seq.server_seq {
                // request offline messages
                messages = api::messages()
                    .pull_offline_msg(user_id.as_str(), local_seq.local_seq, local_seq.server_seq)
                    .await
                    .unwrap();
            }
            local_seq.local_seq = local_seq.server_seq;
            seq_repo.put(&local_seq).await.unwrap();
            ChatsMsg::QueryConvs((convs, messages, local_seq))
        });
        // register state
        let (msg_state, _msg_listener) = ctx
            .link()
            .context(ctx.link().callback(ChatsMsg::SendMsg))
            .expect("need conv state in item");
        let (conv_state, _conv_listener) = ctx
            .link()
            .context(ctx.link().callback(ChatsMsg::ConvStateChanged))
            .expect("need state in item");
        let (unread_state, _unread_listener) = ctx
            .link()
            .context(ctx.link().callback(|_| ChatsMsg::None))
            .expect("need state in item");
        let (remove_conv_state, _remove_conv_listener) = ctx
            .link()
            .context(ctx.link().callback(ChatsMsg::RemoveConvStateChanged))
            .expect("need state in item");
        let (wait_state, _wait_listener) = ctx
            .link()
            .context(ctx.link().callback(|_| ChatsMsg::WaitStateChanged))
            .expect("need state in item");
        let (create_conv, _create_conv_listener) = ctx
            .link()
            .context(ctx.link().callback(ChatsMsg::CreateConvStateChanged))
            .expect("need state in item");
        let (_mute_state, _mute_state_listener) = ctx
            .link()
            .context(ctx.link().callback(ChatsMsg::MuteStateChanged))
            .expect("need state in item");
        let (_add_friend_state, _add_friend_state_listener) = ctx
            .link()
            .context(ctx.link().callback(|_| ChatsMsg::None))
            .expect("need state in item");
        let (sync_msg_state, _sync_msg_state_listener) = ctx
            .link()
            .context(ctx.link().callback(|_| ChatsMsg::None))
            .expect("need state in item");
        let (rec_msg_state, _rec_msg_state_listener) = ctx
            .link()
            .context(ctx.link().callback(|_| ChatsMsg::None))
            .expect("need state in item");
        let rec_msg_listener = ctx.link().callback(ChatsMsg::ReceiveMsg);
        let token = window()
            .local_storage()
            .unwrap()
            .unwrap()
            .get(TOKEN)
            .unwrap()
            .unwrap();
        let addr = window()
            .local_storage()
            .unwrap()
            .unwrap()
            .get(WS_ADDR)
            .unwrap()
            .unwrap();
        let url = format!("{}/{}/conn/{}/{}", addr, id.clone(), token, id);
        let ws = Rc::new(RefCell::new(WebSocketManager::new(url, rec_msg_listener)));
        Self {
            call_msg: SingleCall::default(),
            ws,
            seq: Seq::default(),
            list: IndexMap::new(),
            result: IndexMap::new(),
            query_complete: false,
            is_searching: false,
            show_friend_list: false,
            show_context_menu: false,
            context_menu_pos: (0, 0, AttrValue::default(), false),
            conv_state,
            _conv_listener,
            _remove_conv_state: remove_conv_state,
            _remove_conv_listener,
            unread_state,
            _unread_listener,
            _msg_state: msg_state,
            _msg_listener,
            wait_state,
            _wait_listener,
            _create_conv: create_conv,
            _create_conv_listener,
            _mute_state,
            _mute_state_listener,
            _add_friend_state,
            _add_friend_state_listener,
            sync_msg_state,
            _sync_msg_state_listener,
            rec_msg_state,
            _rec_msg_state_listener,
        }
    }

    pub fn send_msg(&self, msg: Msg) {
        // 发送已收到消息给服务器
        match self.ws.borrow().send_message(msg) {
            Ok(_) => {
                log::info!("发送成功")
            }
            Err(e) => {
                log::error!("发送失败: {:?}", e)
            }
        };
    }
    fn delete_item(&mut self) {
        // delete database data
        let id = self.context_menu_pos.2.clone();
        spawn_local(async move {
            if let Err(e) = db::convs().await.delete(id.as_str()).await {
                log::error!("delete conversation error: {:?}", e);
            }
        });
        if let Some(conv) = self.list.shift_remove(self.context_menu_pos.2.as_str()) {
            if conv.unread_count > 0 {
                self.unread_state.sub_msg_count.emit(conv.unread_count);
            }
        }
        self.show_context_menu = false;
        self.context_menu_pos = (0, 0, AttrValue::default(), false);
        // set right content type
        let mut conv = self.conv_state.conv.clone();
        conv.item_id = AttrValue::default();
        conv.content_type = RightContentType::Default;
        self.conv_state.state_change_event.emit(conv);
    }

    fn mute(&mut self) -> bool {
        if let Some(conv) = self.list.get_mut(&self.context_menu_pos.2) {
            // if concel mute need to notify unread count event
            if conv.mute {
                // notify
                self.unread_state.add_msg_count.emit(conv.unread_count);
            } else {
                self.unread_state.sub_msg_count.emit(conv.unread_count);
            }
            conv.mute = !conv.mute;
            let conv = conv.clone();
            spawn_local(async move {
                if let Err(e) = db::convs().await.mute(&conv).await {
                    log::error!("mute conversation err: {:?}", e);
                }
            });
            self.show_context_menu = false;
            return true;
        }
        false
    }

    fn render_result(&self, ctx: &Context<Self>) -> Html {
        Self::render(&self.result, ctx)
    }

    fn render_list(&self, ctx: &Context<Self>) -> Html {
        Self::render(&self.list, ctx)
    }

    fn render(list: &IndexMap<AttrValue, Conversation>, ctx: &Context<Self>) -> Html {
        let oncontextmenu = ctx
            .link()
            .callback(|((x, y), id, is_mute)| ChatsMsg::ShowContextMenu((x, y), id, is_mute));
        list.iter()
            .map(|(_id, item)| Self::get_list_item(item, oncontextmenu.clone()))
            .collect::<Html>()
    }

    fn get_list_item(
        item: &Conversation,
        oncontextmenu: Callback<((i32, i32), AttrValue, bool)>,
    ) -> Html {
        let remark = get_msg_type(item.last_msg_type, &item.last_msg);
        html!(
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
        )
    }

    fn handle_sent_msg(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        let conv_type = match msg {
            Msg::Group(_) => RightContentType::Group,
            Msg::Single(_) | Msg::SingleCall(_) => RightContentType::Friend,
            _ => RightContentType::Default,
        };
        match msg {
            Msg::Single(mut msg)
            | Msg::Group(GroupMsg::Message(mut msg))
            | Msg::OfflineSync(mut msg) => {
                let conv = Conversation {
                    last_msg: msg.content.clone(),
                    last_msg_time: msg.create_time,
                    last_msg_type: msg.content_type,
                    conv_type,
                    friend_id: msg.friend_id.clone(),
                    unread_count: 1,
                    ..Default::default()
                };
                let is_self = msg.is_self;
                spawn_local(async move {
                    if let Err(err) = db::messages().await.add_message(&mut msg).await {
                        log::error!("{:?}", err);
                    }
                });
                self.operate_msg(ctx, conv, is_self)
            }
            Msg::Group(group_msg) => {
                match group_msg {
                    GroupMsg::Invitation(msg) => {
                        self.handle_group_invitation(ctx, msg);
                    }
                    GroupMsg::Dismiss(group_id) => {
                        self.handle_group_dismiss(ctx, group_id);
                    }
                    // don't handle it now
                    _ => {}
                }
                false
            }
            Msg::SingleCall(SingleCall::Invite(msg)) => {
                let mut conv = Conversation::from(msg);
                conv.unread_count = 1;
                conv.conv_type = conv_type;
                self.operate_msg(ctx, conv, false)
            }
            Msg::SingleCall(SingleCall::InviteCancel(msg)) => {
                let is_self = msg.is_self;
                let mut conv = Conversation::from(msg);
                conv.unread_count = 1;
                conv.conv_type = conv_type;
                self.operate_msg(ctx, conv, is_self)
            }
            Msg::SingleCall(SingleCall::NotAnswer(msg)) => {
                let is_self = msg.is_self;
                let mut conv = Conversation::from(msg);
                conv.unread_count = 1;
                conv.conv_type = conv_type;
                self.operate_msg(ctx, conv, is_self)
            }
            Msg::SingleCall(SingleCall::HangUp(msg)) => {
                let is_self = msg.is_self;
                let mut conv = Conversation::from(msg);
                conv.unread_count = 1;
                conv.conv_type = conv_type;
                self.operate_msg(ctx, conv, is_self)
            }
            Msg::SingleCall(SingleCall::InviteAnswer(msg)) => {
                let is_self = msg.is_self;
                let mut conv = Conversation::from(msg);
                conv.unread_count = 1;
                conv.conv_type = conv_type;
                self.operate_msg(ctx, conv, is_self)
            }
            _ => false,
        }
    }

    fn handle_group_invitation(&mut self, ctx: &Context<Self>, msg: GroupInvitation) {
        // create group conversation directly
        let clone_ctx = ctx.link().clone();
        ctx.link().send_future(async move {
            // store conversation
            let conv = Conversation::from(msg.info.clone());
            db::convs().await.put_conv(&conv, true).await.unwrap();

            // store group information
            if let Err(err) = db::groups().await.put(&msg.info).await {
                log::error!("store group error : {:?}", err);
            };

            // store group members
            if let Err(e) = db::group_members().await.put_list(msg.members).await {
                log::error!("save group member error: {:?}", e);
            }

            // send back received message
            clone_ctx.send_message(ChatsMsg::SendBackGroupInvitation(msg.info.id.clone()));

            // send add friend state
            clone_ctx.send_message(ChatsMsg::SendCreateGroupToContacts(msg.info));
            ChatsMsg::InsertConv(conv)
        });
    }

    fn handle_group_dismiss(&mut self, ctx: &Context<Self>, group_id: String) {
        let key = AttrValue::from(group_id.clone());
        if let Some(conv) = self.list.get_mut(&key) {
            conv.last_msg_time = chrono::Local::now().timestamp_millis();
            conv.last_msg_type = ContentType::Text;
            let mut conv = conv.clone();
            ctx.link().send_future(async move {
                // query group information and owner info
                if let Ok(Some(group)) = db::groups().await.get(&group_id).await {
                    if let Ok(Some(mem)) = db::group_members()
                        .await
                        .get_by_group_id_and_friend_id(&group_id, group.owner.as_str())
                        .await
                    {
                        let message = format!("{} dismissed this group", mem.group_name);
                        conv.last_msg = message.clone().into();

                        if let Err(e) = db::convs().await.put_conv(&conv, true).await {
                            log::error!("dismiss group error: {:?}", e);
                        } else {
                            return ChatsMsg::DismissGroup(key, message);
                        }
                    }
                }
                ChatsMsg::None
            })
        }
    }

    fn get_msg_type(&self, msg: &Msg) -> RightContentType {
        match msg {
            Msg::Group(_) => RightContentType::Group,
            Msg::Single(_) | Msg::SingleCall(_) => RightContentType::Friend,
            _ => RightContentType::Default,
        }
    }

    fn handle_offline_messages(&mut self, ctx: &Context<Self>, messages: Vec<PbMsg>) {
        let mut map: HashMap<AttrValue, Conversation> = HashMap::with_capacity(messages.len());
        for item in messages.into_iter() {
            // let friend_id = item.send_id.clone();
            let msg = convert_server_msg(item).unwrap();
            let conv_type = self.get_msg_type(&msg);
            match msg {
                Msg::Single(mut msg) => {
                    let friend_id = msg.send_id.clone();
                    msg.send_id = msg.friend_id.clone();
                    msg.friend_id = friend_id.clone();
                    msg.is_read = false;
                    if let Some(v) = map.get_mut(&friend_id) {
                        v.last_msg = msg.content.clone();
                        v.last_msg_time = msg.create_time;
                        v.last_msg_type = msg.content_type;
                        v.unread_count += 1;
                    } else {
                        map.insert(
                            friend_id,
                            Conversation {
                                last_msg: msg.content.clone(),
                                last_msg_time: msg.create_time,
                                last_msg_type: msg.content_type,
                                conv_type,
                                friend_id: msg.friend_id.clone(),
                                unread_count: 1,
                                ..Default::default()
                            },
                        );
                    }

                    spawn_local(async move {
                        if let Err(err) = db::messages().await.add_message(&mut msg).await {
                            log::error!("{:?}", err);
                        }
                    });
                }
                Msg::Group(group_msg) => match group_msg {
                    GroupMsg::Invitation(msg) => {
                        self.handle_group_invitation(ctx, msg);
                    }
                    GroupMsg::Dismiss(group_id) => {
                        self.handle_group_dismiss(ctx, group_id);
                    }
                    GroupMsg::Message(msg) => {
                        spawn_local(async move {
                            db::group_msgs().await.put(&msg).await.unwrap();
                        });
                    }
                    GroupMsg::MemberExit((mem_id, group_id)) => {
                        spawn_local(async move {
                            db::group_members()
                                .await
                                .delete(&mem_id, &group_id)
                                .await
                                .unwrap();
                        });
                    }
                    GroupMsg::DismissOrExitReceived(_) => todo!(),
                    GroupMsg::InvitationReceived(_) => todo!(),
                },
                Msg::SingleCall(call_msg) => match call_msg {
                    SingleCall::Invite(_msg) => {
                        // let friend_id = msg.send_id.clone();
                        // msg.send_id = msg.friend_id.clone();
                        // msg.friend_id = friend_id.clone();
                        // let mut conv = Conversation::from(msg);
                        // conv.unread_count = 1;
                        // conv.conv_type = conv_type;
                        // if let Some(v) = map.get_mut(&conv.friend_id) {
                        //     v.last_msg = conv.last_msg;
                        //     v.last_msg_time = conv.last_msg_time;
                        //     v.last_msg_type = conv.last_msg_type;
                        //     v.unread_count += 1;
                        // } else {
                        //     map.insert(conv.friend_id.clone(), conv);
                        // }
                    }
                    SingleCall::InviteCancel(mut msg) => {
                        let friend_id = msg.send_id.clone();
                        msg.send_id = msg.friend_id.clone();
                        msg.friend_id = friend_id.clone();

                        let (last_msg, last_msg_type) = get_invite_type(&msg.invite_type);
                        let conv = Conversation {
                            friend_id: msg.friend_id.clone(),
                            last_msg,
                            last_msg_time: msg.send_time,
                            last_msg_type,
                            unread_count: 1,
                            conv_type,
                            ..Default::default()
                        };

                        spawn_local(async move {
                            db::messages()
                                .await
                                .add_message(&mut Message::from(msg))
                                .await
                                .unwrap();
                        });

                        if let Some(v) = map.get_mut(&conv.friend_id) {
                            v.last_msg = conv.last_msg;
                            v.last_msg_time = conv.last_msg_time;
                            v.last_msg_type = conv.last_msg_type;
                            v.unread_count += 1;
                        } else {
                            map.insert(conv.friend_id.clone(), conv);
                        }
                    }
                    SingleCall::InviteAnswer(mut msg) => {
                        if msg.agree {
                            let friend_id = msg.send_id.clone();
                            msg.send_id = msg.friend_id.clone();
                            msg.friend_id = friend_id;
                            let (last_msg, last_msg_type) = get_invite_type(&msg.invite_type);
                            let conv = Conversation {
                                friend_id: msg.friend_id.clone(),
                                last_msg,
                                last_msg_time: msg.send_time,
                                last_msg_type,
                                unread_count: 1,
                                conv_type,
                                ..Default::default()
                            };
                            spawn_local(async move {
                                db::messages()
                                    .await
                                    .add_message(&mut Message::from(msg))
                                    .await
                                    .unwrap();
                            });
                            if let Some(v) = map.get_mut(&conv.friend_id) {
                                v.last_msg = conv.last_msg;
                                v.last_msg_time = conv.last_msg_time;
                                v.last_msg_type = conv.last_msg_type;
                                v.unread_count += 1;
                            } else {
                                map.insert(conv.friend_id.clone(), conv);
                            }
                        }
                    }
                    SingleCall::NotAnswer(mut msg) => {
                        let friend_id = msg.send_id.clone();
                        msg.send_id = msg.friend_id.clone();
                        msg.friend_id = friend_id;
                        let (last_msg, last_msg_type) = get_invite_type(&msg.invite_type);
                        let conv = Conversation {
                            friend_id: msg.friend_id.clone(),
                            last_msg,
                            last_msg_time: msg.send_time,
                            last_msg_type,
                            unread_count: 1,
                            conv_type,
                            ..Default::default()
                        };
                        spawn_local(async move {
                            db::messages()
                                .await
                                .add_message(&mut Message::from(msg))
                                .await
                                .unwrap();
                        });
                        if let Some(v) = map.get_mut(&conv.friend_id) {
                            v.last_msg = conv.last_msg;
                            v.last_msg_time = conv.last_msg_time;
                            v.last_msg_type = conv.last_msg_type;
                            v.unread_count += 1;
                        } else {
                            map.insert(conv.friend_id.clone(), conv);
                        }
                    }
                    SingleCall::HangUp(mut msg) => {
                        let friend_id = msg.send_id.clone();
                        msg.send_id = msg.friend_id.clone();
                        msg.friend_id = friend_id;
                        let (last_msg, last_msg_type) = get_invite_type(&msg.invite_type);
                        let conv = Conversation {
                            friend_id: msg.friend_id.clone(),
                            last_msg,
                            last_msg_time: msg.send_time,
                            last_msg_type,
                            unread_count: 1,
                            conv_type,
                            ..Default::default()
                        };
                        spawn_local(async move {
                            db::messages()
                                .await
                                .add_message(&mut Message::from(msg))
                                .await
                                .unwrap();
                        });
                        if let Some(v) = map.get_mut(&conv.friend_id) {
                            v.last_msg = conv.last_msg;
                            v.last_msg_time = conv.last_msg_time;
                            v.last_msg_type = conv.last_msg_type;
                            v.unread_count += 1;
                        } else {
                            map.insert(conv.friend_id.clone(), conv);
                        }
                    }

                    _ => {}
                },

                _ => {}
            }
        }
        let is_send = (self.conv_state.conv.content_type == RightContentType::Friend
            || self.conv_state.conv.content_type == RightContentType::Group)
            && map.contains_key(&self.conv_state.conv.item_id);
        for (_, v) in map.into_iter() {
            self.operate_msg(ctx, v, false);
        }
        // todo send sync offline message complete message to msg_list component
        if is_send {
            self.sync_msg_state.complete.emit(());
        }
    }

    fn operate_msg(&mut self, ctx: &Context<Self>, mut conv: Conversation, is_self: bool) -> bool {
        let friend_id = conv.friend_id.clone();
        let dest = self.list.shift_remove(&friend_id);
        let mut clean = false;
        let unread_count = conv.unread_count;
        if dest.is_some() {
            let mut old = dest.unwrap();
            // deal with unread message count
            if !old.mute && !is_self && self.conv_state.conv.item_id != friend_id {
                self.unread_state.add_msg_count.emit(unread_count);
            }
            // 这里是因为要直接更新面板上的数据，所以需要处理未读数量
            if friend_id != self.conv_state.conv.item_id {
                old.unread_count += unread_count;
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
                db::convs().await.put_conv(&conv, clean).await.unwrap();
            });
            true
        } else {
            if !is_self && self.conv_state.conv.item_id != friend_id {
                self.unread_state.add_msg_count.emit(unread_count);
            }
            // 如果会话列表中不存在那么需要新建
            ctx.link().send_future(async move {
                let friend = db::friends().await.get_friend(friend_id.as_str()).await;
                conv.avatar = friend.avatar;
                if let Some(name) = friend.remark {
                    conv.name = name;
                } else {
                    conv.name = friend.name;
                }
                db::convs().await.put_conv(&conv, false).await.unwrap();
                conv.unread_count = unread_count;
                log::debug!("创建会话: {:?}", &conv);
                ChatsMsg::InsertConv(conv)
            });
            false
        }
    }

    fn deal_with_conv_state_change(&mut self, ctx: &Context<Self>, state: Rc<ConvState>) -> bool {
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
                db::convs().await.put_conv(&conv, true).await.unwrap();
            });
            true
        } else {
            // 不存在，那么创建
            let friend_id = cur_conv_id.clone();
            let conv_type = self.conv_state.conv.content_type.clone();
            log::debug!("conv type in messages: {:?}", conv_type.clone());
            ctx.link().send_future(async move {
                // query information by conv_type
                let conv = match conv_type {
                    RightContentType::Friend => {
                        let friend = db::friends().await.get_friend(friend_id.as_str()).await;
                        // todo查询上一条消息
                        let result = db::messages()
                            .await
                            .get_last_msg(friend_id.as_str())
                            .await
                            .unwrap_or_default();
                        let content = if result.id != 0 {
                            get_msg_type(result.content_type, &result.content)
                        } else {
                            AttrValue::default()
                        };
                        Conversation {
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
                        }
                    }
                    RightContentType::Group => {
                        let group = db::groups()
                            .await
                            .get(friend_id.as_str())
                            .await
                            .unwrap()
                            .unwrap();
                        // todo查询上一条消息
                        let result = db::group_msgs()
                            .await
                            .get_last_msg(friend_id.as_str())
                            .await
                            .unwrap_or_default();
                        let content = if result.id != 0 {
                            get_msg_type(result.content_type, &result.content)
                        } else {
                            AttrValue::default()
                        };
                        Conversation {
                            id: 0,
                            name: group.name,
                            avatar: group.avatar,
                            last_msg: content,
                            last_msg_time: result.create_time,
                            last_msg_type: result.content_type,
                            unread_count: 0,
                            friend_id,
                            conv_type,
                            mute: false,
                        }
                    }
                    _ => {
                        log::warn!("not support this type {:?} for now", conv_type);
                        return ChatsMsg::None;
                    }
                };

                db::convs().await.put_conv(&conv, true).await.unwrap();
                log::debug!("状态更新，不存在的会话，添加数据: {:?}", &conv);
                ChatsMsg::InsertConv(conv)
            });
            false
        }
    }

    fn create_group(&mut self, ctx: &Context<Self>, nodes: Vec<String>) {
        log::debug!("get group mems: {:?} ; ", nodes);
        let user_id = ctx.props().user_id.clone();
        let self_avatar = ctx.props().avatar.clone();

        // clone ctx to send message
        let cloned_ctx = ctx.link().clone();

        ctx.link().send_future(async move {
            if nodes.is_empty() {
                return ChatsMsg::ShowSelectFriendList;
            }
            let mut values = Vec::with_capacity(nodes.len());
            // let mut ids = Vec::with_capacity(nodes.len());
            let mut avatar = Vec::with_capacity(nodes.len());
            // push self avatar
            avatar.push(self_avatar.to_string());
            let mut group_name = String::new();
            for (i, node) in nodes.iter().enumerate() {
                let friend = db::friends().await.get_friend(node).await;
                if !friend.fs_id.is_empty() {
                    let mut name = friend.name.clone();
                    if friend.remark.is_some() {
                        name = friend.remark.as_ref().unwrap().clone();
                    }
                    group_name.push_str(name.as_str());
                    if i < 8 {
                        avatar.push(friend.avatar.clone().to_string());
                    }
                    values.push(GroupMember::from(friend));
                }
            }

            group_name.push_str("、Group");
            let group_req = GroupRequest {
                owner: user_id.to_string(),
                avatar: avatar.join(","),
                group_name,
                members_id: nodes,
                id: String::new(),
            };
            // push self
            values.push(GroupMember::from(
                db::users().await.get(user_id.as_str()).await.unwrap(),
            ));
            // send create request
            match api::groups()
                .create_group(group_req, user_id.as_str())
                .await
            {
                Ok(g) => {
                    log::debug!("group created: {:?}", g);

                    // sotre the group info to database
                    if let Err(err) = db::groups().await.put(&g).await {
                        log::error!("create group error: {:?}", err);
                        return ChatsMsg::None;
                    }

                    // store group members to db
                    for v in values.iter_mut() {
                        v.group_id = g.id.clone();
                        if let Err(e) = db::group_members().await.put(v).await {
                            log::error!("save group member error: {:?}", e);
                            continue;
                        }
                    }

                    // send message to contacts component
                    cloned_ctx.send_message(ChatsMsg::SendCreateGroupToContacts(g.clone()));

                    // store conversation info to db
                    let conv = Conversation::from(g);
                    db::convs().await.put_conv(&conv, true).await.unwrap();

                    // insert conversation to ui list
                    ChatsMsg::InsertConv(conv)
                }
                Err(err) => {
                    log::error!("create group request error: {:?}", err);
                    ChatsMsg::None
                }
            }
        });
    }

    pub fn handle_receive_message(&mut self, ctx: &Context<Self>, mut message: Msg) -> bool {
        log::debug!("receive message from websocket");
        let conv_type = match message {
            Msg::Group(_) => RightContentType::Group,
            Msg::Single(_) | Msg::SingleCall(_) => RightContentType::Friend,
            _ => RightContentType::Default,
        };
        match message {
            Msg::Single(ref mut msg) => {
                let friend_id = msg.send_id.clone();
                msg.send_id = msg.friend_id.clone();
                msg.friend_id = friend_id;
                msg.is_read = false;
                msg.is_self = false;

                let conv = Conversation {
                    last_msg: msg.content.clone(),
                    last_msg_time: msg.create_time,
                    last_msg_type: msg.content_type,
                    conv_type,
                    friend_id: msg.friend_id.clone(),
                    unread_count: 1,
                    ..Default::default()
                };
                let is_self = msg.is_self;

                let mut msg = msg.clone();
                // let msg_id = msg.server_id.to_string();

                let is_send = (self.conv_state.conv.content_type == RightContentType::Friend
                    || self.conv_state.conv.content_type == RightContentType::Group)
                    && self.conv_state.conv.item_id == msg.friend_id;
                ctx.link().send_future(async move {
                    // save to db
                    db::messages().await.add_message(&mut msg).await.unwrap();
                    ChatsMsg::None
                    // if let Err(err) = db::messages().await.add_message(&mut msg).await {
                    //     HomeMsg::Notification(Notification::error_from_content(
                    //         format!("Internal Error:{:?}", err).into(),
                    //     ))
                    // } else {
                    //     HomeMsg::SendBackMsg(Msg::SingleDeliveredNotice(msg_id))
                    // }
                });

                // notify other components we have received new message
                if is_send {
                    ctx.link().send_message(ChatsMsg::RecMsgNotify(message));
                }
                return self.operate_msg(ctx, conv, is_self);
            }
            Msg::Group(ref group_msg) => {
                match group_msg {
                    GroupMsg::Invitation(msg) => {
                        // receive create group message
                        self.handle_group_invitation(ctx, msg.clone());
                    }
                    GroupMsg::Message(msg) => {
                        let msg = msg.clone();
                        let _msg_id = msg.server_id.to_string();
                        let conv = Conversation {
                            last_msg: msg.content.clone(),
                            last_msg_time: msg.create_time,
                            last_msg_type: msg.content_type,
                            conv_type,
                            friend_id: msg.friend_id.clone(),
                            unread_count: 1,
                            ..Default::default()
                        };
                        let is_self = msg.is_self;
                        // if self.conv_state.conv.item_id != msg.friend_id {
                        //     let conv_state = Rc::make_mut(&mut self.conv_state);
                        //     let _ = current_item::save_conv(&conv_state.conv)
                        //         .map_err(|err| log::error!("save conv fail{:?}", err));
                        // }
                        let is_send = (self.conv_state.conv.content_type
                            == RightContentType::Friend
                            || self.conv_state.conv.content_type == RightContentType::Group)
                            && self.conv_state.conv.item_id == msg.friend_id;
                        ctx.link().send_future(async move {
                            // 数据入库
                            db::group_msgs().await.put(&msg).await.unwrap();
                            ChatsMsg::None
                            // if let Err(err) = db::group_msgs().await.put(&msg).await {
                            //     HomeMsg::Notification(Notification::error_from_content(
                            //         format!("内部错误:{:?}", err).into(),
                            //     ))
                            // } else {
                            //     HomeMsg::SendBackMsg(Msg::SingleDeliveredNotice(msg_id))
                            // }
                        });

                        if is_send {
                            ctx.link().send_message(ChatsMsg::RecMsgNotify(message));
                        }
                        return self.operate_msg(ctx, conv, is_self);
                    }
                    GroupMsg::MemberExit((mem_id, group_id)) => {
                        // todo modify conversation list
                        // delete member information from da
                        // let user_id = ctx.props().user_id.clone();
                        let mem_id = mem_id.clone();
                        let group_id = group_id.clone();
                        // let ctx = ctx.link().clone();
                        spawn_local(async move {
                            db::group_members()
                                .await
                                .delete(&mem_id, &group_id)
                                .await
                                .unwrap();
                            // if let Err(err) =
                            //     db::group_members().await.delete(&mem_id, &group_id).await
                            // {
                            //     log::error!("remove group member fail:{:?}", err);
                            // } else {
                            //     // send message received
                            //     ctx.send_message(HomeMsg::SendBackMsg(Msg::Group(
                            //         GroupMsg::DismissOrExitReceived((
                            //             user_id.to_string(),
                            //             group_id,
                            //         )),
                            //     )));
                            // }
                        });
                    }
                    GroupMsg::Dismiss(group_id) => {
                        // delete group from db
                        // let user_id = ctx.props().user_id.clone();
                        // we can consume the group_msg here because it is behind in the reference
                        let cloned_group_id = group_id.clone();
                        log::debug!("received dismiss message, group id : {}", group_id);
                        let is_send = (self.conv_state.conv.content_type
                            == RightContentType::Friend
                            || self.conv_state.conv.content_type == RightContentType::Group)
                            && self.conv_state.conv.item_id == group_id;
                        spawn_local(async move {
                            if let Err(err) = db::groups().await.dismiss(&cloned_group_id).await {
                                log::error!("remove group fail:{:?}", err);
                            } else {
                                //     // send message to other component
                                //     ctx.send_message(HomeMsg::RecSendMsgStateChange(message));
                                //     // send message received
                                //     ctx.send_message(HomeMsg::SendBackMsg(Msg::Group(
                                //         GroupMsg::DismissOrExitReceived((
                                //             user_id.to_string(),
                                //             group_id,
                                //         )),
                                //     )));
                            }
                        });

                        self.handle_group_dismiss(ctx, group_id.to_string());
                        if is_send {
                            ctx.link().send_message(ChatsMsg::RecMsgNotify(message));
                        }
                    }
                    GroupMsg::DismissOrExitReceived(_) | GroupMsg::InvitationReceived(_) => {}
                }
            }
            Msg::SendRelationshipReq(_msg) => {}
            Msg::RecRelationship(friendship) => {
                // 收到好友请求
                self.handle_friendship_req(ctx, friendship);
            }
            Msg::ReadNotice(_) | Msg::SingleDeliveredNotice(_) => {}
            Msg::OfflineSync(_) => {}
            Msg::SingleCall(m) => {
                // call message is handled by PhoneCall component
                // 保存电话信息，通知phone call组件
                self.call_msg = m;
                return true;
            }
            Msg::FriendshipDeliveredNotice(_) => {}
            Msg::RelationshipRes(friend) => {
                // 收到好友同意消息
                // self.info(AttrValue::from("好友同意"));
                let send_id = ctx.props().user_id.clone();
                // 需要通知联系人列表更新
                // 数据入库
                // let cloned_ctx = ctx.link().clone();
                ctx.link().send_future(async move {
                    db::friendships()
                        .await
                        .agree_by_friend_id(friend.friend_id.as_str())
                        .await;
                    db::friends().await.put_friend(&friend).await;
                    // send received message
                    // cloned_ctx.send_message(HomeMsg::SendBackMsg(Msg::FriendshipDeliveredNotice(
                    // friend.fs_id.to_string(),
                    // )));
                    // send hello message
                    let mut msg = Message {
                        seq: 0,
                        local_id: nanoid::nanoid!().into(),
                        server_id: AttrValue::default(),
                        send_id,
                        friend_id: friend.friend_id.clone(),
                        content_type: ContentType::Text,
                        content: friend
                            .hello
                            .unwrap_or_else(|| AttrValue::from(DEFAULT_HELLO_MESSAGE)),
                        create_time: chrono::Local::now().timestamp_millis(),
                        is_read: true,
                        is_self: true,
                        file_content: AttrValue::default(),
                        id: 0,
                        send_time: 0,
                        is_success: false,
                    };
                    let _ = db::messages()
                        .await
                        .add_message(&mut msg)
                        .await
                        .map_err(|err| log::error!("save message fail:{:?}", err));
                    ChatsMsg::SendMessage(Msg::Single(msg))
                });
            }
            Msg::ServerRecResp(msg) => {
                log::debug!("receive server response: {:?}", msg);
            }
        }
        false
    }
    fn handle_friendship_req(
        &mut self,
        _ctx: &Context<Self>,
        friendship: FriendShipWithUser,
    ) -> bool {
        log::debug!("ReceiveFriendShipReq:{:?}", &friendship);
        // let id = friendship.fs_id.clone().to_string();
        // let state = Rc::make_mut(&mut self.friend_ship_state);
        // state.ship = Some(friendship.clone());
        // state.state_type = FriendShipStateType::Req;
        // // 入库
        // ctx.link().send_future(async move {
        //     db::friendships().await.put_friendship(&friendship).await;
        //     // 发送收到通知
        //     HomeMsg::SendBackMsg(Msg::FriendshipDeliveredNotice(id))
        // });
        // // 显示通知
        // // self.info(AttrValue::from("收到好友请求"));
        // ctx.link().send_message(HomeMsg::Notification(Notification {
        //     type_: NotificationType::Info,
        //     title: AttrValue::default(),
        //     content: AttrValue::from("收到好友请求"),
        // }));
        true
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
        ContentType::Error => AttrValue::from("[ERROR]"),
    }
}
