use std::rc::Rc;

use gloo::timers::callback::Timeout;
use indexmap::IndexMap;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::scope_ext::RouterScopeExt;
use yewdux::Dispatch;

use i18n::{en_us, zh_cn, LanguageType};
use sandcat_sdk::api;
use sandcat_sdk::db;
use sandcat_sdk::model::conversation::Conversation;
use sandcat_sdk::model::group::Group;
use sandcat_sdk::model::message::Msg;
use sandcat_sdk::model::notification::Notification;
use sandcat_sdk::model::page::Page;
use sandcat_sdk::model::seq::Seq;
use sandcat_sdk::model::{ComponentType, CurrentItem, REFRESH_TOKEN, TOKEN};
use sandcat_sdk::state::CreateConvState;
use sandcat_sdk::state::{
    AddFriendState, AddFriendStateItem, ComponentTypeState, CreateGroupConvState, I18nState,
    ItemType, MuteState, RemoveConvState, SendMessageState, UpdateFriendState,
};
use sandcat_sdk::state::{ConvState, UnreadState};
use utils::tr;
use ws::WebSocketManager;

use crate::call::PhoneCall;
use crate::constant::{KNOCK_OFF_MSG, NO_RESULT, OK};
use crate::dialog::Dialog;
use crate::left::right_click_panel::RightClickPanel;
use crate::select_friends::SelectFriendList;
use crate::top_bar::TopBar;

use super::Chats;

type QueryResult = (
    IndexMap<AttrValue, Conversation>,
    IndexMap<AttrValue, Conversation>,
    Seq,
);

#[derive(Debug)]
pub enum ChatsMsg {
    /// filter conversations for local data
    FilterConv(AttrValue),
    /// clean filter result
    CleanupSearchResult,
    /// query conversation list from db
    QueryConvList(QueryResult),
    /// receive a message from server
    ReceiveMsg(Msg),
    /// send message from sender component
    SendMsg(Rc<SendMessageState>),
    /// send message for self
    SendMessage(Msg),
    /// insert a conversation item to list
    InsertConv(Conversation),
    InsertConvWithoutUpdate(Conversation),
    /// conversation state changed -- change the current conv
    ConvStateChanged(Rc<ConvState>),
    /// show friend list while we want to create a group
    ShowSelectFriendList,
    /// create group by selected friend id
    CreateGroup(Vec<String>),
    SendBackGroupInvitation(AttrValue),
    /// right click menu, contains click position, conv id and if mute
    ShowContextMenu((i32, i32), AttrValue, bool, bool),
    CloseContextMenu,
    /// delete conversation
    DeleteItem,
    /// mute conversation
    Mute,
    /// pin conversation to top
    Pin(bool),
    /// do nothing
    None,
    /// create a conversation item by received state
    CreateGroupConvStateChanged(Rc<CreateGroupConvState>),
    CreateConvStateChanged(Rc<CreateConvState>),
    /// update a conversation item by received state
    UpdateConvStateChanged(Rc<UpdateFriendState>),
    /// remove a conversation item by received state
    RemoveConvStateChanged(Rc<RemoveConvState>),
    /// receive mute message from right component
    MuteStateChanged(Rc<MuteState>),
    /// send create group message to contacts component
    /// after we receive a group creation message
    SendCreateGroupToContacts(Group),
    /// dismiss group positive
    DismissGroup(AttrValue, String),
    RecMsgNotify(Msg),
    /// handle the lack messages
    HandleLackMessages(Vec<Conversation>),
    /// switch language by received state
    SwitchLanguage(Rc<I18nState>),
    /// handle touch event for mobile
    OnTouchStart(TouchEvent),
    OnTouchEnd(TouchEvent),
    /// this client is knocked off by another client with same platform
    KnockOff,
    /// sign out
    Logout,
    /// need to update token in local storage
    UpdateToken(String, bool),
    /// send refresh token request
    RefreshToken(bool),
    /// unauthorized, go to login page
    Unauthorized,
}

#[derive(Properties, PartialEq, Debug)]
pub struct ChatsProps {
    pub user_id: AttrValue,
    pub avatar: AttrValue,
}

impl Component for Chats {
    type Message = ChatsMsg;

    type Properties = ChatsProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self::new(ctx)
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        log::debug!("Chats update:{:?}", msg);
        match msg {
            ChatsMsg::FilterConv(pattern) => {
                self.is_searching = true;
                // filter message list
                if pattern.is_empty() {
                    ctx.link().send_message(ChatsMsg::CleanupSearchResult);
                } else {
                    let insert_if_matches =
                        |key: &AttrValue,
                         item: &Conversation,
                         result: &mut IndexMap<AttrValue, Conversation>| {
                            if item
                                .remark
                                .as_deref()
                                .map_or(false, |remark| remark.contains(pattern.as_str()))
                                || item.name.contains(pattern.as_str())
                            {
                                result.insert(key.clone(), item.clone());
                            }
                        };

                    self.list
                        .iter()
                        .for_each(|(key, item)| insert_if_matches(key, item, &mut self.result));

                    self.pinned_list
                        .iter()
                        .for_each(|(key, item)| insert_if_matches(key, item, &mut self.result));
                    return true;
                }
                false
            }
            ChatsMsg::CleanupSearchResult => {
                self.is_searching = false;
                self.result.clear();
                true
            }
            ChatsMsg::QueryConvList((pined_list, convs, seq)) => {
                self.pinned_list = pined_list;
                self.list = convs;
                self.query_complete = true;
                self.seq = seq;

                // unmount loading
                Dialog::close_loading();
                if let Err(e) = WebSocketManager::connect(self.ws.clone()) {
                    Notification::error(e).notify();
                }
                true
            }
            ChatsMsg::InsertConv(conv) => {
                self.list.shift_insert(0, conv.friend_id.clone(), conv);
                true
            }
            ChatsMsg::ShowSelectFriendList => {
                self.show_friend_list = !self.show_friend_list;
                true
            }
            ChatsMsg::ConvStateChanged(state) => self.deal_with_conv_state_change(ctx, state),
            ChatsMsg::CreateGroup(nodes) => {
                self.show_friend_list = false;
                if nodes.is_empty() {
                    return true;
                }
                // create group conversation and send 'create group' message
                Self::create_group(ctx, nodes);
                // send message to contacts component
                false
            }
            ChatsMsg::SendBackGroupInvitation(_group_id) => {
                // self.msg_state
                //     .send_back_event
                //     .emit(Msg::Group(GroupMsg::InvitationReceived((
                //         ctx.props().user_id.to_string(),
                //         group_id.to_string(),
                //     ))));
                false
            }
            ChatsMsg::ShowContextMenu((x, y), id, is_mute, is_pined) => {
                // event.prevent_default();
                self.context_menu_pos = (x, y, id, is_mute, is_pined);
                self.show_context_menu = true;
                true
            }
            ChatsMsg::CloseContextMenu => {
                self.show_context_menu = false;
                self.context_menu_pos = (0, 0, AttrValue::default(), false, false);
                true
            }
            ChatsMsg::DeleteItem => self.delete_item(),
            ChatsMsg::None => false,
            ChatsMsg::RemoveConvStateChanged(state) => {
                if state.id.is_empty() {
                    return false;
                }
                // delete conversation from database should be here
                if let Some(conv) = self.list.shift_remove(state.id.as_str()) {
                    if conv.unread_count > 0 {
                        Dispatch::<UnreadState>::global().reduce_mut(|s| {
                            s.msg_count = s.msg_count.saturating_sub(conv.unread_count)
                        });
                    }
                    if conv.friend_id == self.conv_state.conv.item_id {
                        self.conv_dispatch
                            .reduce_mut(|s| s.conv = CurrentItem::default());
                    }
                };
                true
            }
            ChatsMsg::Mute => self.mute(),
            ChatsMsg::Pin(is_pined) => self.pin(is_pined),
            ChatsMsg::CreateGroupConvStateChanged(state) => {
                if state.group.is_some() {
                    let list = state.group.clone();
                    Self::create_group(ctx, list.unwrap());
                    return true;
                }
                false
            }
            ChatsMsg::MuteStateChanged(state) => {
                if let Some(item) = self.list.get_mut(&state.conv_id) {
                    item.mute = !item.mute;
                };
                false
            }
            ChatsMsg::SendCreateGroupToContacts(group) => {
                Dispatch::<AddFriendState>::global()
                    .reduce_mut(|s| s.item = AddFriendStateItem::from(group));
                false
            }
            ChatsMsg::DismissGroup(group_id, msg) => {
                if let Some(conv) = self.list.get_mut(&group_id) {
                    conv.last_msg = msg.into();
                    conv.last_msg_time = chrono::Utc::now().timestamp_millis();
                }
                true
            }
            ChatsMsg::ReceiveMsg(msg) => self.handle_receive_message(ctx, msg),
            ChatsMsg::SendMsg(state) => {
                log::debug!("send message from sender in conversation");
                let msg = state.msg.clone();
                self.handle_sent_msg(ctx, &msg);
                self.send_msg(msg);
                true
            }
            ChatsMsg::RecMsgNotify(msg) => {
                self.rec_msg_dis.reduce_mut(|s| s.msg = msg.clone());
                false
            }
            ChatsMsg::SendMessage(msg) => {
                self.handle_sent_msg(ctx, &msg);
                self.send_msg(msg);
                true
            }
            ChatsMsg::InsertConvWithoutUpdate(conv) => {
                self.list.shift_insert(0, conv.friend_id.clone(), conv);
                false
            }
            ChatsMsg::HandleLackMessages(conv_list) => {
                // todo 优化
                conv_list.into_iter().for_each(|conv| {
                    self.handle_conv(ctx, conv);
                });
                true
            }
            ChatsMsg::SwitchLanguage(state) => {
                self.lang_state = state;
                let content = match self.lang_state.lang {
                    LanguageType::ZhCN => zh_cn::CONVERSATION,
                    LanguageType::EnUS => en_us::CONVERSATION,
                };
                self.i18n = utils::create_bundle(content);
                true
            }
            ChatsMsg::UpdateConvStateChanged(state) => {
                let update_conv = |conv: &mut Conversation| {
                    if let Some(name) = &state.name {
                        conv.name.clone_from(name);
                    }
                    if let Some(avatar) = &state.avatar {
                        conv.avatar.clone_from(avatar);
                    }
                    conv.remark.clone_from(&state.remark);

                    // save to db
                    let conv = conv.clone();
                    spawn_local(async move {
                        if let Err(err) = db::db_ins().convs.put_conv(&conv).await {
                            log::error!("update conv error: {:?}", err);
                        }
                    });
                };
                if let Some(item) = self.pinned_list.get_mut(&state.id) {
                    update_conv(item);
                    return true;
                } else if let Some(item) = self.list.get_mut(&state.id) {
                    update_conv(item);
                    return true;
                }
                false
            }
            ChatsMsg::OnTouchStart(event) => {
                if let Some(touch) = event.touches().get(0) {
                    log::debug!("TouchStart: {}", touch.client_x());
                    self.touch_start = touch.client_x();
                };
                false
            }
            ChatsMsg::OnTouchEnd(event) => {
                // we can't use the .touches() to get the touch end
                // should use the changed_touches()
                if let Some(touch) = event.changed_touches().get(0) {
                    log::debug!("TouchEnd: {}", touch.client_x());
                    if self.touch_start - touch.client_x() > 50 {
                        // go to contacts
                        Dispatch::<ComponentTypeState>::global()
                            .reduce_mut(|s| s.component_type = ComponentType::Contacts);
                    }
                }
                self.touch_start = 0;
                false
            }
            ChatsMsg::KnockOff => {
                self.is_knocked = true;
                true
            }
            ChatsMsg::Logout => {
                if let Some(navigator) = ctx.link().navigator() {
                    navigator.push(&Page::Login);
                }
                //  record logout time
                spawn_local(async {
                    let now = chrono::Utc::now().timestamp_millis();
                    if let Err(err) = db::db_ins().offline_time.save(now).await {
                        log::error!("save offline time error: {:?}", err);
                    }
                });
                false
            }
            ChatsMsg::UpdateToken(token, is_refresh) => {
                // set refresh timer
                if let Some(claims) = Self::decode_jwt(&token) {
                    let ctx = ctx.link().clone();
                    let timeout = claims.exp - chrono::Utc::now().timestamp() - 60;
                    let callback = Some(Timeout::new((timeout as u32) * 1000, move || {
                        ctx.send_message(ChatsMsg::RefreshToken(is_refresh));
                    }));
                    if is_refresh {
                        self.refresh_token_getter = callback;
                    } else {
                        self.token_getter = callback;
                    }
                }
                false
            }
            ChatsMsg::RefreshToken(is_refresh) => {
                ctx.link().send_future(async move {
                    let refresh_token = utils::get_local_storage(REFRESH_TOKEN).unwrap();
                    match api::users().refresh_token(&refresh_token, is_refresh).await {
                        Ok(token) => {
                            let key = if is_refresh { REFRESH_TOKEN } else { TOKEN };
                            utils::set_local_storage(key, &token).unwrap();
                            ChatsMsg::UpdateToken(token, is_refresh)
                        }
                        Err(err) => {
                            log::error!("refresh token error: {:?}", err);
                            ChatsMsg::None
                        }
                    }
                });
                false
            }
            ChatsMsg::Unauthorized => {
                // todo give user a tip which need to relogin
                if let Some(navigator) = ctx.link().navigator() {
                    navigator.push(&Page::Login);
                }
                false
            }
            ChatsMsg::CreateConvStateChanged(state) => {
                let conv = state.conv.clone();
                self.list.shift_insert(0, conv.friend_id.clone(), conv);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let (ontouchstart, ontouchend) = if self.is_mobile {
            (
                Some(ctx.link().callback(ChatsMsg::OnTouchStart)),
                Some(ctx.link().callback(ChatsMsg::OnTouchEnd)),
            )
        } else {
            (None, None)
        };
        let content = if self.is_searching {
            if self.result.is_empty() {
                html! {<div class="no-result">{tr!(self.i18n, NO_RESULT)}</div>}
            } else {
                self.render_result(ctx)
            }
        } else {
            self.render_list(ctx)
        };
        let search_callback = ctx.link().callback(ChatsMsg::FilterConv);
        let clean_callback = ctx.link().callback(move |_| ChatsMsg::CleanupSearchResult);
        let plus_click = ctx.link().callback(|_| ChatsMsg::ShowSelectFriendList);

        // spawn friend list
        let mut friend_list = html!();
        if self.show_friend_list {
            let submit_back = ctx.link().callback(ChatsMsg::CreateGroup);
            friend_list = html! {
                <SelectFriendList
                    close_back={plus_click.clone()}
                    {submit_back}
                    lang={self.lang_state.lang}
                    from={ItemType::Friend}/>
            };
        }
        let mut context_menu = html!();
        if self.show_context_menu {
            context_menu = html! {
                <RightClickPanel
                    x={self.context_menu_pos.0}
                    y={self.context_menu_pos.1}
                    close={ctx.link().callback( |_|ChatsMsg::CloseContextMenu)}
                    mute={ctx.link().callback(|_| ChatsMsg::Mute)}
                    pin={ctx.link().callback(ChatsMsg::Pin)}
                    delete={ctx.link().callback(|_|ChatsMsg::DeleteItem)}
                    is_mute={self.context_menu_pos.3}
                    is_pinned={self.context_menu_pos.4}
                    lang={self.lang_state.lang}/>
            }
        }

        // show warning about knock off
        let mut warning = html!();
        if self.is_knocked {
            warning = html! {
                <div class="knock-off-warning">
                    <div class="warning-window box-shadow">
                        <div>{tr!(self.i18n, KNOCK_OFF_MSG)}</div>
                        <button onclick={ctx.link().callback(|_|ChatsMsg::Logout)}>{tr!(self.i18n, OK)}</button>
                    </div>
                </div>
            }
        }
        // PhoneCall send message callback
        let send_msg_callback = ctx
            .link()
            .callback(|msg| ChatsMsg::SendMessage(Msg::SingleCall(msg)));
        html! {
        <>
            {warning}
            <PhoneCall
                ws={self.ws.clone()}
                user_id={&ctx.props().user_id}
                msg={self.call_msg.clone()}
                send_msg={send_msg_callback}
                lang={self.lang_state.lang}/>

            <div class="list-wrapper" {ontouchstart} {ontouchend}>
                {context_menu}
                {friend_list}
                <TopBar
                    components_type={ComponentType::Messages}
                    {search_callback}
                    {clean_callback}
                    {plus_click}
                    lang={self.lang_state.lang}/>
                <div class="contacts-list">
                    {content}
                </div>
            </div>
        </>
        }
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        self.token_getter = None;
        self.refresh_token_getter = None;
        self.ws.borrow_mut().cleanup();
        // record the offline time
        spawn_local(async {
            // db::repository::Repository::delete_db().await;
            let now = chrono::Utc::now().timestamp_millis();
            if let Err(err) = db::db_ins().offline_time.save(now).await {
                log::error!("record offline time to local storage error: {:?}", err);
            }
        });
    }
}
