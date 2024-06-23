mod conversations;
mod handle_group;
mod handle_msg;
mod handle_offline_msg;

use std::{cell::RefCell, rc::Rc};

use base64::prelude::*;
use fluent::{FluentBundle, FluentResource};
use gloo::timers::callback::Timeout;
use indexmap::IndexMap;
use log::error;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yewdux::Dispatch;

use i18n::{
    en_us::{self, CONVERSATION},
    zh_cn, LanguageType,
};
use sandcat_sdk::{
    api, db,
    error::{Error, WebSocketError},
    model::{
        conversation::Conversation,
        message::{Msg, SingleCall},
        notification::Notification,
        seq::Seq,
        user::Claims,
        CommonProps, ComponentType, ContentType, CurrentItem, RightContentType, REFRESH_TOKEN,
        TOKEN, WS_ADDR,
    },
    state::{
        ConvState, CreateConvState, I18nState, MobileState, MuteState, RecMessageState,
        RemoveConvState, SendMessageState, UnreadState, UpdateConvState,
    },
};
use utils::tr;
use ws::WebSocketManager;

use self::conversations::ChatsMsg;
use crate::{
    constant::{AUDIO, AUDIO_CALL, EMOJI, ERROR, FILE, IMAGE, LOADING, VIDEO, VIDEO_CALL},
    dialog::Dialog,
    left::list_item::ListItem,
};

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
    _send_msg_dis: Dispatch<SendMessageState>,
    /// listen the conversation change
    /// used to change the right panel content
    conv_state: Rc<ConvState>,
    conv_dispatch: Dispatch<ConvState>,
    /// listen the conversation remove,
    /// used to receive that contact list to delete the friends to remove the conversation
    _remove_conv_dis: Dispatch<RemoveConvState>,
    /// listen to the create conv event, like:
    _create_conv_dis: Dispatch<CreateConvState>,
    // _create_conv_listener: ContextHandle<Rc<CreateConvState>>,
    /// mute conversation,
    /// used to receive the mute event from right panel
    _mute_dis: Dispatch<MuteState>,
    /// send the create friend/group event to contact list
    /// send the event to other components after receive a message
    rec_msg_dis: Dispatch<RecMessageState>,
    lang_state: Rc<I18nState>,
    _lang_dispatch: Dispatch<I18nState>,
    _update_dis: Dispatch<UpdateConvState>,
    touch_start: i32,
    is_mobile: bool,
    is_knocked: bool,
    token_getter: Option<Timeout>,
    refresh_token_getter: Option<Timeout>,
}

impl Chats {
    fn new(ctx: &Context<Self>) -> Self {
        let lang_dispatch =
            Dispatch::global().subscribe(ctx.link().callback(ChatsMsg::SwitchLanguage));
        let lang_state = lang_dispatch.get();
        let res = match lang_state.lang {
            LanguageType::ZhCN => zh_cn::CONVERSATION,
            LanguageType::EnUS => en_us::CONVERSATION,
        };
        let i18n = utils::create_bundle(res);
        Dialog::loading(&tr!(i18n, LOADING));

        let id = ctx.props().user_id.clone();
        // query conversation list
        let user_id = id.clone();
        ctx.link().send_future(async move {
            let convs = db::db_ins().convs.get_convs().await.unwrap_or_default();
            // pull offline messages
            // get the seq
            // todo handle the error
            let server_seq = api::seq().get_seq(&user_id).await.unwrap_or_default();
            let mut local_seq = db::db_ins().seq.get().await.unwrap_or_default();
            let mut messages = Vec::new();
            log::debug!("local seq: {:?}; server seq:{:?}", local_seq, server_seq);
            if local_seq.local_seq < server_seq.seq {
                log::debug!("pull offline messages");
                // request offline messages
                messages = match api::messages()
                    .pull_offline_msg(user_id.as_str(), local_seq.local_seq, server_seq.seq)
                    .await
                {
                    Ok(messages) => messages,
                    Err(e) => {
                        error!("pull offline messages error: {:?}", e);
                        Notification::error("pull offline messages error").notify();
                        return ChatsMsg::None;
                    }
                };
                local_seq.local_seq = server_seq.seq;
                if let Err(e) = db::db_ins().seq.put(&local_seq).await {
                    error!("save local seq error: {:?}", e);
                    Notification::error("save local seq error").notify();
                    return ChatsMsg::None;
                }
            }
            ChatsMsg::QueryConvList((convs, messages, local_seq))
        });
        // we need use conv state to rerender the chats component, so use subscribe in create
        let conv_dispatch =
            Dispatch::global().subscribe(ctx.link().callback(ChatsMsg::ConvStateChanged));

        let _send_msg_dis =
            Dispatch::global().subscribe_silent(ctx.link().callback(ChatsMsg::SendMsg));
        let _remove_conv_dis = Dispatch::global()
            .subscribe_silent(ctx.link().callback(ChatsMsg::RemoveConvStateChanged));
        let _create_conv_dis = Dispatch::global()
            .subscribe_silent(ctx.link().callback(ChatsMsg::CreateConvStateChanged));
        let _mute_dis =
            Dispatch::global().subscribe_silent(ctx.link().callback(ChatsMsg::MuteStateChanged));
        let rec_msg_dis =
            Dispatch::global().subscribe_silent(ctx.link().callback(|_| ChatsMsg::None));
        // same as conv state

        let rec_msg_listener = ctx.link().callback(ChatsMsg::ReceiveMsg);
        let addr = utils::get_local_storage(WS_ADDR).unwrap();
        let platform = MobileState::get();
        let is_mobile = platform.is_mobile();
        let url = format!(
            "{}/{}/conn/{}/{}",
            addr,
            id.clone(),
            id,
            (*platform).clone() as i32
        );
        let knockoff = ctx.link().callback(|_| ChatsMsg::KnockOff);
        let logout = ctx.link().callback(|_| ChatsMsg::Unauthorized);
        let ws = Rc::new(RefCell::new(WebSocketManager::new(
            url,
            rec_msg_listener,
            knockoff,
            logout,
        )));

        let _update_dis = Dispatch::global()
            .subscribe_silent(ctx.link().callback(ChatsMsg::UpdateConvStateChanged));

        // validate token
        Self::validate_token(ctx, false);
        Self::validate_token(ctx, true);

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
            _remove_conv_dis,
            _send_msg_dis,
            _create_conv_dis,
            _mute_dis,
            rec_msg_dis,
            i18n,
            lang_state,
            _lang_dispatch: lang_dispatch,
            conv_state: conv_dispatch.get(),
            conv_dispatch,
            _update_dis,
            touch_start: 0,
            is_mobile,
            is_knocked: false,
            token_getter: None,
            refresh_token_getter: None,
        }
    }

    fn validate_token(ctx: &Context<Self>, is_refresh: bool) {
        let key = if is_refresh { REFRESH_TOKEN } else { TOKEN };
        let token = utils::get_local_storage(key).unwrap();
        if let Some(claim) = Self::decode_jwt(&token) {
            if Self::should_refresh(claim.exp) {
                // refresh token
                ctx.link().send_future(async move {
                    match api::users()
                        .refresh_token(
                            &utils::get_local_storage(REFRESH_TOKEN).unwrap(),
                            is_refresh,
                        )
                        .await
                    {
                        Ok(token) => {
                            utils::set_local_storage(key, &token).unwrap();
                            ChatsMsg::UpdateToken(token, is_refresh)
                        }
                        Err(e) => {
                            error!("refresh token error: {:?}", e);
                            ChatsMsg::None
                        }
                    }
                });
            } else {
                ctx.link()
                    .send_message(ChatsMsg::UpdateToken(token, is_refresh))
            }
        }
    }

    fn should_refresh(exp: i64) -> bool {
        let now = chrono::Utc::now().timestamp();
        if exp - now < 60 {
            return true;
        }
        false
    }

    fn decode_jwt(token: &str) -> Option<Claims> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        BASE64_STANDARD_NO_PAD
            .decode(parts[1])
            .map_err(|e| log::error!("decode jwt error: {:?}", e))
            .ok()
            .and_then(|decoded| serde_json::from_slice::<Claims>(&decoded).ok())
    }

    pub fn send_msg(&self, msg: Msg) {
        // 发送已收到消息给服务器
        if let Err(e) = self.ws.borrow().send_message(msg) {
            if e == Error::WebSocket(WebSocketError::Closed) {
                // reconnect websocket
                if let Err(e) = WebSocketManager::connect(self.ws.clone()) {
                    log::error!("websocket connect error: {:?}", e);
                }
            } else {
                log::error!("send message error: {:?}", e);
            }
        }
    }

    fn delete_item(&mut self) {
        // delete database data
        let id = self.context_menu_pos.2.clone();
        spawn_local(async move {
            if let Err(e) = db::db_ins().convs.delete(id.as_str()).await {
                log::error!("delete conversation error: {:?}", e);
            }
        });

        if let Some(conv) = self.list.shift_remove(self.context_menu_pos.2.as_str()) {
            if conv.unread_count > 0 {
                Dispatch::<UnreadState>::global()
                    .reduce_mut(|s| s.msg_count = s.msg_count.saturating_sub(conv.unread_count));
            }
        }

        // set right content type
        if Dispatch::<ConvState>::global().get().conv.item_id == self.context_menu_pos.2 {
            Dispatch::<ConvState>::global().reduce_mut(|s| s.conv = CurrentItem::default());
        }

        self.show_context_menu = false;
        self.context_menu_pos = (0, 0, AttrValue::default(), false);
    }

    fn mute(&mut self) -> bool {
        if let Some(conv) = self.list.get_mut(&self.context_menu_pos.2) {
            // if concel mute need to notify unread count event
            if conv.mute {
                // notify
                Dispatch::<UnreadState>::global()
                    .reduce_mut(|s| s.msg_count = s.msg_count.saturating_add(conv.unread_count));
            } else {
                Dispatch::<UnreadState>::global()
                    .reduce_mut(|s| s.msg_count = s.msg_count.saturating_sub(conv.unread_count));
            }

            conv.mute = !conv.mute;
            let conv = conv.clone();

            spawn_local(async move {
                if let Err(e) = db::db_ins().convs.mute(&conv).await {
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
        let name = if let Some(remark) = &item.remark {
            remark.clone()
        } else {
            item.name.clone()
        };
        html!(
            <ListItem
                component_type={ComponentType::Messages}
                props={CommonProps{name,
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
                db::db_ins().convs.put_conv(&conv).await.unwrap();
            });
            need_rerender
        } else {
            // not exists, create a new conversation
            let friend_id = cur_conv_id;
            let conv_type = self.conv_state.conv.content_type.clone();
            log::debug!("conv type in messages: {:?}", conv_type.clone());

            ctx.link().send_future(async move {
                // i18n
                let bundle = utils::create_bundle(CONVERSATION);
                // query information by conv_type
                let conv = match conv_type {
                    RightContentType::Friend => {
                        let friend = db::db_ins().friends.get(friend_id.as_str()).await;
                        // todo查询上一条消息
                        let result = db::db_ins()
                            .messages
                            .get_last_msg(friend_id.as_str())
                            .await
                            .unwrap_or_default();
                        let content = if result.id != 0 {
                            get_msg_type(&bundle, result.content_type, &result.content)
                        } else {
                            AttrValue::default()
                        };
                        Conversation {
                            // id: 0,
                            name: friend.name,
                            remark: friend.remark,
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
                        let group = db::db_ins()
                            .groups
                            .get(friend_id.as_str())
                            .await
                            .unwrap()
                            .unwrap();
                        // todo查询上一条消息
                        let result = db::db_ins()
                            .group_msgs
                            .get_last_msg(friend_id.as_str())
                            .await
                            .unwrap_or_default();
                        let content = if result.id != 0 {
                            get_msg_type(&bundle, result.content_type, &result.content)
                        } else {
                            AttrValue::default()
                        };
                        Conversation {
                            // id: 0,
                            name: group.name,
                            remark: group.remark,
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

                db::db_ins().convs.put_conv(&conv).await.unwrap();
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
        ContentType::Image => AttrValue::from(tr!(bundle, IMAGE)),
        ContentType::Video => AttrValue::from(tr!(bundle, VIDEO)),
        ContentType::File => AttrValue::from(tr!(bundle, FILE)),
        ContentType::Emoji => AttrValue::from(tr!(bundle, EMOJI)),
        ContentType::Default => AttrValue::from(""),
        ContentType::VideoCall => AttrValue::from(tr!(bundle, VIDEO_CALL)),
        ContentType::AudioCall => AttrValue::from(tr!(bundle, AUDIO_CALL)),
        ContentType::Audio => AttrValue::from(tr!(bundle, AUDIO)),
        ContentType::Error => AttrValue::from(tr!(bundle, ERROR)),
    }
}

#[cfg(test)]
mod test {
    use base64::Engine;
    use sandcat_sdk::model::user::Claims;

    #[test]
    fn test_get_msg_type() {
        let s = "eyJzdWIiOiJ4bWoiLCJleHAiOjE3MTg1MDMyMjc3NTcsImlhdCI6MTcxODMzMDQyNzc1N30";
        let claims = base64::engine::general_purpose::URL_SAFE
            .decode(s)
            .map_err(|e| println!("decode jwt error: {:?}", e))
            .ok()
            .and_then(|decoded| serde_json::from_slice::<Claims>(&decoded).ok());

        println!("claims: {:?}", claims);
    }
}
