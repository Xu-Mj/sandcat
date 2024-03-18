use std::rc::Rc;

use indexmap::IndexMap;
use yew::prelude::*;

use crate::components::left::right_click_panel::RightClickPanel;
use crate::components::left::select_friends::SelectFriendList;
use crate::components::top_bar::TopBar;
use crate::db;
use crate::model::conversation::Conversation;
use crate::model::group::Group;
use crate::model::message::{GroupMsg, Msg, SingleCall};
use crate::model::{ComponentType, CurrentItem, RightContentType};
use crate::pages::{
    AddFriendStateItem, ConvState, CreateConvState, MuteState, RecSendMessageState, RemoveConvState,
};

use super::Chats;

pub enum ChatsMsg {
    FilterContact(AttrValue),
    CleanupSearchResult,
    QueryConvs(IndexMap<AttrValue, Conversation>),
    ReceiveMessage(Rc<RecSendMessageState>),
    InsertConv(Conversation),
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
            ChatsMsg::QueryConvs(convs) => {
                self.list = convs;
                self.query_complete = true;
                // 数据查询完成，通知Home组件我已经做完必要的工作了
                self.wait_state.ready.emit(());
                true
            }
            ChatsMsg::ReceiveMessage(state) => {
                let msg = state.msg.clone();
                let conv_type = match msg {
                    Msg::Group(_) => RightContentType::Group,
                    Msg::Single(_) | Msg::SingleCall(_) => RightContentType::Friend,
                    _ => RightContentType::Default,
                };
                match msg {
                    Msg::Single(msg) | Msg::OfflineSync(msg) => {
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
                    Msg::Group(GroupMsg::Invitation(msg)) => {
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
                            clone_ctx.send_message(ChatsMsg::SendBackGroupInvitation(
                                msg.info.id.clone(),
                            ));

                            // send add friend state
                            clone_ctx.send_message(ChatsMsg::SendCreateGroupToContacts(msg.info));
                            ChatsMsg::InsertConv(conv)
                        });
                        // don't handle it now
                        // _ => {}
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
            ChatsMsg::InsertConv(flag) => {
                // self.list.insert(0, flag);
                self.list.shift_insert(0, flag.friend_id.clone(), flag);
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
            ChatsMsg::SendBackGroupInvitation(group_id) => {
                self.msg_state
                    .send_back_event
                    .emit(Msg::Group(GroupMsg::InvitationReceived((
                        ctx.props().user_id.to_string(),
                        group_id.to_string(),
                    ))));
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
                self._add_friend_state
                    .add
                    .emit(AddFriendStateItem::from(group));
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
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
