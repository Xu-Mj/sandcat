use std::rc::Rc;

use indexmap::IndexMap;
use web_sys::NodeList;
use yew::prelude::*;

use crate::components::left::right_click_panel::RightClickPanel;
use crate::components::left::select_friends::AddConv;
use crate::db::group::GroupRepo;
use crate::db::group_members::GroupMembersRepo;
use crate::model::conversation::Conversation;
use crate::model::message::{Msg, SingleCall};
use crate::model::{ComponentType, CurrentItem, RightContentType};
use crate::pages::{ConvState, RecSendMessageState, RemoveConvState};
use crate::{components::top_bar::TopBar, db::conv::ConvRepo};

use super::Conversations;

pub enum ConversationsMsg {
    FilterContact(AttrValue),
    CleanupSearchResult,
    QueryConvs(IndexMap<AttrValue, Conversation>),
    ReceiveMessage(Rc<RecSendMessageState>),
    InsertConv(Conversation),
    ConvStateChanged(Rc<ConvState>),
    WaitStateChanged,
    ShowSelectFriendList,
    CreateGroup(NodeList),
    SendBackGroupInvitation(AttrValue),
    ShowContextMenu((i32, i32), AttrValue, bool),
    CloseContextMenu,
    DeleteItem,
    Mute,
    None,
    RemoveConvStateChanged(Rc<RemoveConvState>),
}

#[derive(Properties, PartialEq, Debug)]
pub struct ConversationsProps {
    pub user_id: AttrValue,
    pub avatar: AttrValue,
}

impl Component for Conversations {
    type Message = ConversationsMsg;

    type Properties = ConversationsProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self::new(ctx)
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ConversationsMsg::FilterContact(pattern) => {
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
            ConversationsMsg::CleanupSearchResult => {
                self.is_searching = false;
                self.result.clear();
                true
            }
            ConversationsMsg::QueryConvs(convs) => {
                self.list = convs;
                self.query_complete = true;
                // 数据查询完成，通知Home组件我已经做完必要的工作了
                self.wait_state.ready.emit(());
                true
            }
            ConversationsMsg::ReceiveMessage(state) => {
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
                            clone_ctx.send_message(ConversationsMsg::SendBackGroupInvitation(
                                msg.info.id,
                            ));
                            ConversationsMsg::InsertConv(conv)
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
            ConversationsMsg::InsertConv(flag) => {
                // self.list.insert(0, flag);
                self.list.shift_insert(0, flag.friend_id.clone(), flag);
                true
            }
            ConversationsMsg::ShowSelectFriendList => {
                self.show_friend_list = !self.show_friend_list;
                true
            }
            ConversationsMsg::ConvStateChanged(state) => {
                self.deal_with_conv_state_change(ctx, state)
            }
            ConversationsMsg::WaitStateChanged => false,
            ConversationsMsg::CreateGroup(nodes) => {
                self.show_friend_list = false;
                if nodes.length() == 0 {
                    return true;
                }
                // create group conversation and send 'create group' message
                self.get_group_mems(ctx, nodes);
                false
            }
            ConversationsMsg::SendBackGroupInvitation(group_id) => {
                self.msg_state
                    .send_back_event
                    .emit(Msg::GroupInvitationReceived((
                        ctx.props().user_id.to_string(),
                        group_id.to_string(),
                    )));
                false
            }
            ConversationsMsg::ShowContextMenu((x, y), id, is_mute) => {
                // event.prevent_default();
                self.context_menu_pos = (x, y, id, is_mute);
                self.show_context_menu = true;
                true
            }
            ConversationsMsg::CloseContextMenu => {
                log::debug!("close context menu");
                self.show_context_menu = false;
                self.context_menu_pos = (0, 0, AttrValue::default(), false);
                true
            }
            ConversationsMsg::DeleteItem => {
                self.delete_item();
                true
            }
            ConversationsMsg::None => false,
            ConversationsMsg::RemoveConvStateChanged(state) => {
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
            ConversationsMsg::Mute => self.mute(),
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
        let search_callback = ctx.link().callback(ConversationsMsg::FilterContact);
        let clean_callback = ctx
            .link()
            .callback(move |_| ConversationsMsg::CleanupSearchResult);
        let plus_click = ctx
            .link()
            .callback(|_| ConversationsMsg::ShowSelectFriendList);
        let submit_back = ctx.link().callback(ConversationsMsg::CreateGroup);

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
                    close={ctx.link().callback( |_|ConversationsMsg::CloseContextMenu)}
                    mute={ctx.link().callback(|_| ConversationsMsg::Mute)}
                    delete={ctx.link().callback(|_|ConversationsMsg::DeleteItem)}
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
