mod conversations;
mod handle_group;
mod handle_msg;
mod handle_offline_msg;

use std::{cell::RefCell, rc::Rc};

use fluent::{FluentBundle, FluentResource};
use gloo::utils::window;
use indexmap::IndexMap;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::{
    api,
    components::left::list_item::ListItem,
    db::{self, TOKEN, WS_ADDR},
    i18n::{
        en_us::{self, CONVERSATION},
        zh_cn, LanguageType,
    },
    model::{
        conversation::Conversation,
        message::{Msg, SingleCall},
        seq::Seq,
        CommonProps, ComponentType, ContentType, RightContentType,
    },
    pages::{
        AddFriendState, ConvState, CreateConvState, FriendShipState, I18nState, MuteState,
        OfflineMsgState, RecMessageState, RemoveConvState, SendMessageState, UnreadState,
        WaitState,
    },
    tr, utils,
    ws::WebSocketManager,
};

use self::conversations::ChatsMsg;

pub struct Chats {
    /// used to notify the PhoneCall component to make a call
    /// when it changed, the PhoneCall component will be re-rendered
    call_msg: SingleCall,
    /// websocket manager, all messages from the server will be handled by this manager
    ws: Rc<RefCell<WebSocketManager>>,
    /// received messages sequence,
    /// used to determine whether the message is the latest message
    seq: Seq,
    /// the list of conversations
    list: IndexMap<AttrValue, Conversation>,
    /// search result list
    result: IndexMap<AttrValue, Conversation>,
    /// whether the search is in progress
    is_searching: bool,
    /// whether the query is complete
    query_complete: bool,
    /// create group friend list panel
    show_friend_list: bool,
    /// right click menu
    show_context_menu: bool,
    i18n: FluentBundle<FluentResource>,
    /// hold right click item position and id
    context_menu_pos: (i32, i32, AttrValue, bool),

    /// receive the message state from sender
    /// sender send message through Home Component
    _send_msg_state: Rc<SendMessageState>,
    _send_msg_listener: ContextHandle<Rc<SendMessageState>>,
    /// listen the conversation change
    /// used to change the right panel content
    conv_state: Rc<ConvState>,
    _conv_listener: ContextHandle<Rc<ConvState>>,
    /// listen the conversation remove,
    /// used to receive that contact list to delete the friends to remove the conversation
    _remove_conv_state: Rc<RemoveConvState>,
    _remove_conv_listener: ContextHandle<Rc<RemoveConvState>>,
    /// change the global unread count
    unread_state: Rc<UnreadState>,
    _unread_listener: ContextHandle<Rc<UnreadState>>,
    /// when this component is ready, send the event to notify the parent component
    wait_state: Rc<WaitState>,
    _wait_listener: ContextHandle<Rc<WaitState>>,
    /// listen to the create conv event, like:
    _create_conv: Rc<CreateConvState>,
    _create_conv_listener: ContextHandle<Rc<CreateConvState>>,
    /// mute conversation,
    /// used to receive the mute event from right panel
    _mute_state: Rc<MuteState>,
    _mute_state_listener: ContextHandle<Rc<MuteState>>,
    /// send the create friend/group event to contact list
    add_friend_state: Rc<AddFriendState>,
    _add_friend_state_listener: ContextHandle<Rc<AddFriendState>>,
    /// send the event to other components after sync offline message completed
    sync_msg_state: Rc<OfflineMsgState>,
    _sync_msg_state_listener: ContextHandle<Rc<OfflineMsgState>>,
    /// send the event to other components after receive a message
    rec_msg_state: Rc<RecMessageState>,
    _rec_msg_state_listener: ContextHandle<Rc<RecMessageState>>,
    /// friendship state, notify the contact component after receive a friend application
    fs_state: Rc<FriendShipState>,
    _fs_state_listener: ContextHandle<Rc<FriendShipState>>,
    lang_state: Rc<I18nState>,
    _lang_state_listener: ContextHandle<Rc<I18nState>>,
}

impl Chats {
    fn new(ctx: &Context<Self>) -> Self {
        let id = ctx.props().user_id.clone();
        // query conversation list
        let user_id = id.clone();
        ctx.link().send_future(async move {
            let convs = db::convs().await.get_convs2().await.unwrap_or_default();
            // pull offline messages
            // get the seq
            // todo handle the error
            let server_seq = api::seq().get_seq(&user_id).await.unwrap();
            let seq_repo = db::seq().await;
            let mut local_seq = seq_repo.get().await.unwrap_or_default();
            let mut messages = Vec::new();
            log::debug!("seq: {:?}", local_seq);
            if local_seq.local_seq < server_seq.seq {
                // request offline messages
                messages = api::messages()
                    .pull_offline_msg(user_id.as_str(), local_seq.local_seq, server_seq.seq)
                    .await
                    .unwrap();
            }
            local_seq.local_seq = server_seq.seq;
            seq_repo.put(&local_seq).await.unwrap();
            ChatsMsg::QueryConvs((convs, messages, local_seq))
        });
        // register state
        let (_send_msg_state, _send_msg_listener) = ctx
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
        let (_remove_conv_state, _remove_conv_listener) = ctx
            .link()
            .context(ctx.link().callback(ChatsMsg::RemoveConvStateChanged))
            .expect("need state in item");
        let (wait_state, _wait_listener) = ctx
            .link()
            .context(ctx.link().callback(|_| ChatsMsg::WaitStateChanged))
            .expect("need state in item");
        let (_create_conv, _create_conv_listener) = ctx
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
        let (fs_state, _fs_state_listener) = ctx
            .link()
            .context(ctx.link().callback(|_| ChatsMsg::None))
            .expect("need state in item");
        let (lang_state, _lang_state_listener) = ctx
            .link()
            .context(ctx.link().callback(ChatsMsg::SwitchLanguage))
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
        let res = match lang_state.lang {
            LanguageType::ZhCN => zh_cn::CONVERSATION,
            LanguageType::EnUS => en_us::CONVERSATION,
        };
        let i18n = utils::create_bundle(res);
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
            _remove_conv_state,
            _remove_conv_listener,
            unread_state,
            _unread_listener,
            _send_msg_state,
            _send_msg_listener,
            wait_state,
            _wait_listener,
            _create_conv,
            _create_conv_listener,
            _mute_state,
            _mute_state_listener,
            add_friend_state: _add_friend_state,
            _add_friend_state_listener,
            sync_msg_state,
            _sync_msg_state_listener,
            rec_msg_state,
            _rec_msg_state_listener,
            fs_state,
            _fs_state_listener,
            i18n,
            lang_state,
            _lang_state_listener,
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
        self.render(&self.result, ctx)
    }

    fn render_list(&self, ctx: &Context<Self>) -> Html {
        self.render(&self.list, ctx)
    }

    fn render(&self, list: &IndexMap<AttrValue, Conversation>, ctx: &Context<Self>) -> Html {
        let oncontextmenu = ctx
            .link()
            .callback(|((x, y), id, is_mute)| ChatsMsg::ShowContextMenu((x, y), id, is_mute));

        list.iter()
            .map(|(_id, item)| self.get_list_item(item, oncontextmenu.clone()))
            .collect::<Html>()
    }

    fn get_list_item(
        &self,
        item: &Conversation,
        oncontextmenu: Callback<((i32, i32), AttrValue, bool)>,
    ) -> Html {
        let remark = get_msg_type(&self.i18n, item.last_msg_type, &item.last_msg);
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

    fn get_msg_type(&self, msg: &Msg) -> RightContentType {
        match msg {
            Msg::Group(_) => RightContentType::Group,
            Msg::Single(_) | Msg::SingleCall(_) => RightContentType::Friend,
            _ => RightContentType::Default,
        }
    }

    fn deal_with_conv_state_change(&mut self, ctx: &Context<Self>, state: Rc<ConvState>) -> bool {
        self.conv_state = state;
        let cur_conv_id = self.conv_state.conv.item_id.clone();
        // 设置了一个查询状态，如果在查询没有完成时更新了状态，那么不进行更新列表，这里有待于优化，
        // 因为状态会在
        if cur_conv_id.is_empty() || !self.query_complete {
            return false;
        }

        // if use searching, do not rerender the UI, just update the data
        let need_rerender = !self.is_searching;
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
            need_rerender
        } else {
            // not exists, create a new conversation
            let friend_id = cur_conv_id.clone();
            let conv_type = self.conv_state.conv.content_type.clone();
            log::debug!("conv type in messages: {:?}", conv_type.clone());

            ctx.link().send_future(async move {
                // i18n
                let bundle = utils::create_bundle(CONVERSATION);
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
                            get_msg_type(&bundle, result.content_type, &result.content)
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
                            get_msg_type(&bundle, result.content_type, &result.content)
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
                if need_rerender {
                    ChatsMsg::InsertConv(conv)
                } else {
                    ChatsMsg::InsertConvWithoutUpdate(conv)
                }
            });
            false
        }
    }
}

fn get_msg_type(
    bundle: &FluentBundle<FluentResource>,
    msg_type: ContentType,
    content: &AttrValue,
) -> AttrValue {
    match msg_type {
        ContentType::Text => content.clone(),
        ContentType::Image => AttrValue::from(tr!(bundle, "image")),
        ContentType::Video => AttrValue::from(tr!(bundle, "video")),
        ContentType::File => AttrValue::from(tr!(bundle, "file")),
        ContentType::Emoji => AttrValue::from(tr!(bundle, "emoji")),
        ContentType::Default => AttrValue::from(""),
        ContentType::VideoCall => AttrValue::from(tr!(bundle, "video call")),
        ContentType::AudioCall => AttrValue::from(tr!(bundle, "audio call")),
        ContentType::Audio => AttrValue::from(tr!(bundle, "audio")),
        ContentType::Error => AttrValue::from(tr!(bundle, "error")),
    }
}
