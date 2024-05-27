use std::rc::Rc;

use i18n::{en_us, zh_cn, LanguageType};
use indexmap::IndexMap;
use sandcat_sdk::model::page::Page;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yew_router::scope_ext::RouterScopeExt;
use yewdux::Dispatch;

use sandcat_sdk::db;
use sandcat_sdk::model::conversation::Conversation;
use sandcat_sdk::model::group::Group;
use sandcat_sdk::model::message::Msg;
use sandcat_sdk::model::seq::Seq;
use sandcat_sdk::model::{ComponentType, CurrentItem, RightContentType};
use sandcat_sdk::pb::message::Msg as PbMsg;
use sandcat_sdk::state::{
    AddFriendState, AddFriendStateItem, ComponentTypeState, CreateConvState, I18nState, MuteState,
    RemoveConvState, SendMessageState, UpdateConvState,
};
use sandcat_sdk::state::{ConvState, UnreadState};

use crate::left::right_click_panel::RightClickPanel;
use crate::phone_call::PhoneCall;
use crate::select_friends::SelectFriendList;
use crate::top_bar::TopBar;
use ws::WebSocketManager;

use super::Chats;

#[derive(Debug)]
pub enum ChatsMsg {
    FilterContact(AttrValue),
    CleanupSearchResult,
    QueryConvs((IndexMap<AttrValue, Conversation>, Vec<PbMsg>, Seq)),
    // ReceiveMessage(Rc<RecSendMessageState>),
    ReceiveMsg(Msg),
    // send message from sender
    SendMsg(Rc<SendMessageState>),
    // send message for self
    SendMessage(Msg),
    InsertConv(Conversation),
    InsertConvWithoutUpdate(Conversation),
    ConvStateChanged(Rc<ConvState>),
    ShowSelectFriendList,
    CreateGroup(Vec<String>),
    SendBackGroupInvitation(AttrValue),
    ShowContextMenu((i32, i32), AttrValue, bool),
    CloseContextMenu,
    DeleteItem,
    Mute,
    None,
    RemoveConvStateChanged(Rc<RemoveConvState>),
    CreateConvStateChanged(Rc<CreateConvState>),
    MuteStateChanged(Rc<MuteState>),
    SendCreateGroupToContacts(Group),
    DismissGroup(AttrValue, String),
    RecMsgNotify(Msg),
    /// handle the lack messages
    HandleLackMessages(Vec<PbMsg>),
    SwitchLanguage(Rc<I18nState>),
    UpdateConvStateChanged(Rc<UpdateConvState>),
    OnTouchStart(TouchEvent),
    OnTouchEnd(TouchEvent),
    KnockOff,
    Logout,
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
            ChatsMsg::FilterContact(pattern) => {
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
            ChatsMsg::CleanupSearchResult => {
                self.is_searching = false;
                self.result.clear();
                true
            }
            ChatsMsg::QueryConvs((convs, messages, seq)) => {
                self.list = convs;
                self.query_complete = true;
                // 数据查询完成，通知Home组件我已经做完必要的工作了
                self.seq = seq;
                // handle offline messages
                self.handle_offline_messages(ctx, messages);
                WebSocketManager::connect(self.ws.clone());
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
                self.create_group(ctx, nodes);
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
            ChatsMsg::ShowContextMenu((x, y), id, is_mute) => {
                // event.prevent_default();
                self.context_menu_pos = (x, y, id, is_mute);
                self.show_context_menu = true;
                true
            }
            ChatsMsg::CloseContextMenu => {
                log::debug!("close context menu");
                self.show_context_menu = false;
                self.context_menu_pos = (0, 0, AttrValue::default(), false);
                true
            }
            ChatsMsg::DeleteItem => {
                self.delete_item();
                true
            }
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
            ChatsMsg::CreateConvStateChanged(state) => {
                match state.type_ {
                    RightContentType::Friend => {}
                    RightContentType::Group => {
                        if state.group.is_some() {
                            let list = state.group.clone();
                            self.create_group(ctx, list.unwrap());
                            return true;
                        }
                    }
                    _ => {}
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
                // self.rec_msg_dis.reduce_mut(|s| s.msg = msg_inner.clone());
                true
            }
            ChatsMsg::RecMsgNotify(msg) => {
                self.rec_msg_dis.reduce_mut(|s| s.msg = msg.clone());
                false
            }
            ChatsMsg::SendMessage(msg) => {
                self.handle_sent_msg(ctx, &msg);
                self.send_msg(msg);
                // self.rec_msg_dis.reduce_mut(|s| s.msg = msg_inner.clone());
                true
            }
            ChatsMsg::InsertConvWithoutUpdate(conv) => {
                self.list.shift_insert(0, conv.friend_id.clone(), conv);
                false
            }
            ChatsMsg::HandleLackMessages(messages) => {
                self.handle_offline_messages(ctx, messages);
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
                if let Some(item) = self.list.get_mut(&state.id) {
                    if let Some(name) = state.name.clone() {
                        item.name = name;
                    }
                    if let Some(avatar) = state.avatar.clone() {
                        item.avatar = avatar;
                    }
                    let conv = item.clone();
                    spawn_local(async move {
                        if let Err(err) = db::db_ins().convs.put_conv(&conv).await {
                            log::error!("update conv error: {:?}", err);
                        }
                    });
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
                false
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
                html! {<div class="no-result">{"没有搜索结果"}</div>}
            } else {
                self.render_result(ctx)
            }
        } else {
            self.render_list(ctx)
        };
        let search_callback = ctx.link().callback(ChatsMsg::FilterContact);
        let clean_callback = ctx.link().callback(move |_| ChatsMsg::CleanupSearchResult);
        let plus_click = ctx.link().callback(|_| ChatsMsg::ShowSelectFriendList);
        let submit_back = ctx.link().callback(ChatsMsg::CreateGroup);

        // spawn friend list
        let mut friend_list = html!();
        if self.show_friend_list {
            friend_list = html! {
                <SelectFriendList
                    except={AttrValue::default()}
                    close_back={plus_click.clone()}
                    {submit_back}
                    lang={self.lang_state.lang}/>
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
                    delete={ctx.link().callback(|_|ChatsMsg::DeleteItem)}
                    is_mute={self.context_menu_pos.3}
                    lang={self.lang_state.lang}/>
            }
        }

        // show warning about knock off
        let mut warning = html!();
        if self.is_knocked {
            warning = html! {
                <div>
                    {"GoodBye"}
                    <button onclick={ctx.link().callback(|_|ChatsMsg::Logout)}>{"OK"}</button>
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
            <PhoneCall ws={self.ws.clone()} user_id={ctx.props().user_id.clone()} msg={self.call_msg.clone()} send_msg={send_msg_callback}/>
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
        self.ws.borrow_mut().cleanup();
    }
}
