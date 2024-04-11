use std::rc::Rc;

use indexmap::IndexMap;
use yew::prelude::*;

use super::Chats;
use crate::components::left::right_click_panel::RightClickPanel;
use crate::components::left::select_friends::SelectFriendList;
use crate::components::phone_call::PhoneCall;
use crate::components::top_bar::TopBar;
use crate::model::conversation::Conversation;
use crate::model::group::Group;
use crate::model::message::Msg;
use crate::model::seq::Seq;
use crate::model::{ComponentType, CurrentItem, RightContentType};
use crate::pages::{
    AddFriendStateItem, ConvState, CreateConvState, MuteState, RemoveConvState, SendMessageState,
};
use crate::pb::message::Msg as PbMsg;
use crate::ws::WebSocketManager;
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
    WaitStateChanged,
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
                log::debug!("query complete:{:?}", convs);
                self.list = convs;
                self.query_complete = true;
                // 数据查询完成，通知Home组件我已经做完必要的工作了
                self.wait_state.ready.emit(());
                self.seq = seq;
                // handle offline messages
                self.handle_offline_messages(ctx, messages);
                WebSocketManager::connect(self.ws.clone());
                true
            }
            /* ChatsMsg::ReceiveMessage(state) => {
                let msg = state.msg.clone();
                self.handle_received_messages(ctx, msg)
            } */
            ChatsMsg::InsertConv(conv) => {
                self.list.shift_insert(0, conv.friend_id.clone(), conv);
                true
            }
            ChatsMsg::ShowSelectFriendList => {
                self.show_friend_list = !self.show_friend_list;
                true
            }
            ChatsMsg::ConvStateChanged(state) => self.deal_with_conv_state_change(ctx, state),
            ChatsMsg::WaitStateChanged => false,
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
                self.add_friend_state
                    .add
                    .emit(AddFriendStateItem::from(group));
                false
            }
            ChatsMsg::DismissGroup(group_id, msg) => {
                if let Some(conv) = self.list.get_mut(&group_id) {
                    conv.last_msg = msg.into();
                    conv.last_msg_time = chrono::Local::now().timestamp_millis();
                }
                true
            }
            ChatsMsg::ReceiveMsg(msg) => self.handle_receive_message(ctx, msg),
            ChatsMsg::SendMsg(state) => {
                log::debug!("send message from sender in conversation");
                let msg = state.msg.clone();
                self.send_msg(msg.clone());
                self.rec_msg_state.notify.emit(msg.clone());
                self.handle_sent_msg(ctx, msg);
                true
            }
            ChatsMsg::RecMsgNotify(msg) => {
                self.rec_msg_state.notify.emit(msg);
                false
            }
            ChatsMsg::SendMessage(msg) => {
                self.send_msg(msg.clone());
                self.rec_msg_state.notify.emit(msg.clone());
                self.handle_sent_msg(ctx, msg);
                true
            }
            ChatsMsg::InsertConvWithoutUpdate(conv) => {
                self.list.shift_insert(0, conv.friend_id.clone(), conv);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        log::debug!("Chats::view");
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
                <SelectFriendList except={AttrValue::default()} close_back={plus_click.clone()} {submit_back}/>
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
                    is_mute={self.context_menu_pos.3}/>
            }
        }

        // PhoneCall send message callback
        let send_msg_callback = ctx
            .link()
            .callback(|msg| ChatsMsg::SendMessage(Msg::SingleCall(msg)));
        html! {
        // <ContextProvider<SingleCall> context={self.call_msg.clone()}>
        <>
            <PhoneCall ws={self.ws.clone()} user_id={ctx.props().user_id.clone()} msg={self.call_msg.clone()} send_msg={send_msg_callback}/>
            <div class="list-wrapper">
                {context_menu}
                {friend_list}
                <TopBar components_type={ComponentType::Messages} {search_callback} {clean_callback} {plus_click}/>
                <div class="contacts-list">
                    {content}
                </div>
            </div>
        </>
        // </ContextProvider<SingleCall>>

        }
    }

    // fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
    //     if first_render {
    //         WebSocketManager::connect(self.ws.clone());
    //     }
    // }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        self.ws.borrow_mut().cleanup();
    }
}
