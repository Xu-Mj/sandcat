mod conversations;

use std::rc::Rc;

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use indexmap::IndexMap;

use crate::{
    api,
    components::left::list_item::ListItem,
    db::{
        conv::ConvRepo, friend::FriendRepo, group::GroupRepo, group_members::GroupMembersRepo,
        groups::GroupInterface, message::MessageRepo, user::UserRepo,
    },
    model::{
        conversation::Conversation,
        group::{GroupMember, GroupRequest},
        CommonProps, ComponentType, ContentType, RightContentType,
    },
    pages::{
        ConvState, CreateConvState, RecSendMessageState, RemoveConvState, UnreadState, WaitState,
    },
};

use self::conversations::ChatsMsg;

pub struct Chats {
    list: IndexMap<AttrValue, Conversation>,
    result: IndexMap<AttrValue, Conversation>,
    is_searching: bool,
    query_complete: bool,
    show_friend_list: bool,
    show_context_menu: bool,
    /// hold right click item position and id
    context_menu_pos: (i32, i32, AttrValue, bool),

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
    _create_conv: Rc<CreateConvState>,
    _create_conv_listener: ContextHandle<Rc<CreateConvState>>,
}

impl Chats {
    fn new(ctx: &Context<Self>) -> Self {
        // query conversation list
        ctx.link().send_future(async {
            let conv_repo = ConvRepo::new().await;
            let convs = conv_repo.get_convs2().await.unwrap_or_default();
            ChatsMsg::QueryConvs(convs)
        });
        // register state
        let (msg_state, _msg_listener) = ctx
            .link()
            .context(ctx.link().callback(ChatsMsg::ReceiveMessage))
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
        Self {
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
            msg_state,
            _msg_listener,
            wait_state,
            _wait_listener,
            _create_conv: create_conv,
            _create_conv_listener,
        }
    }

    fn delete_item(&mut self) {
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
                if let Err(e) = ConvRepo::new().await.mute(&conv).await {
                    log::error!("mute conversation err: {:?}", e);
                }
            });
            self.show_context_menu = false;
            return true;
        }
        false
    }

    fn render_result(&self, ctx: &Context<Self>) -> Html {
        let oncontextmenu = ctx
            .link()
            .callback(|((x, y), id, is_mute)| ChatsMsg::ShowContextMenu((x, y), id, is_mute));
        self.result
            .iter()
            .map(|(_id, item)| Self::get_list_item(item, oncontextmenu.clone()))
            .collect::<Html>()
    }

    fn render_list(&self, ctx: &Context<Self>) -> Html {
        let oncontextmenu = ctx
            .link()
            .callback(|((x, y), id, is_mute)| ChatsMsg::ShowContextMenu((x, y), id, is_mute));
        self.list
            .iter()
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
                self.unread_state.add_msg_count.emit(1);
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
                self.unread_state.add_msg_count.emit(1);
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
                ChatsMsg::InsertConv(conv)
            });
            false
        }
    }

    fn get_group_mems(&mut self, ctx: &Context<Self>, nodes: Vec<String>) {
        log::debug!("get group mems: {:?} ; ", nodes);
        let user_id = ctx.props().user_id.clone();
        let self_avatar = ctx.props().avatar.clone();
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
                let friend = FriendRepo::new()
                    .await
                    .get_friend(node.clone().into())
                    .await;
                if !friend.id.is_empty() {
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
                UserRepo::new().await.get(user_id.clone()).await.unwrap(),
            ));
            // send create request
            match api::group::create_group(group_req, user_id).await {
                Ok(g) => {
                    log::debug!("group created: {:?}", g);
                    if let Err(err) = GroupRepo::new().await.put(&g).await {
                        log::error!("create group error: {:?}", err);
                        return ChatsMsg::None;
                    }
                    for v in values.iter_mut() {
                        v.group_id = g.id.clone();
                        if let Err(e) = GroupMembersRepo::new().await.put(v).await {
                            log::error!("save group member error: {:?}", e);
                            continue;
                        }
                    }
                    let conv = Conversation::from(g);
                    ConvRepo::new().await.put_conv(&conv, true).await.unwrap();
                    ChatsMsg::InsertConv(conv)
                }
                Err(err) => {
                    log::error!("create group request error: {:?}", err);
                    ChatsMsg::None
                }
            }
        });
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
