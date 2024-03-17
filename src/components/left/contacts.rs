use indexmap::IndexMap;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::components::left::add_friend::AddFriend;
use crate::db;
use crate::db::group::GroupRepo;
use crate::db::groups::GroupInterface;
use crate::model::group::Group;
use crate::model::{CurrentItem, FriendShipStateType, ItemInfo, RightContentType};
use crate::pages::{AddFriendState, FriendListState, FriendShipState, ItemType, RemoveFriendState};
use crate::{
    components::{left::list_item::ListItem, top_bar::TopBar},
    model::friend::Friend,
    model::{CommonProps, ComponentType},
};

#[derive(Properties, PartialEq, Debug)]
pub struct ContactsProps {
    pub user_id: AttrValue,
}

/// listen group invitation state to add group to group list
pub struct Contacts {
    friends: IndexMap<AttrValue, Friend>,
    result: IndexMap<AttrValue, Friend>,
    groups: IndexMap<AttrValue, Group>,
    // 未读消息数量
    friendships_unread_count: usize,
    // 是否正在搜索
    is_searching: bool,
    is_add_friend: bool,
    show_context_menu: bool,
    _friendship_state: Rc<FriendShipState>,
    _listener: ContextHandle<Rc<FriendShipState>>,
    friend_state: Rc<FriendListState>,
    _friend_listener: ContextHandle<Rc<FriendListState>>,
    _remove_friend_state: Rc<RemoveFriendState>,
    _remove_friend_listener: ContextHandle<Rc<RemoveFriendState>>,
    _add_friend_state: Rc<AddFriendState>,
    _add_friend_listener: ContextHandle<Rc<AddFriendState>>,
}

pub enum QueryState<T> {
    Querying,
    Success(T),
    // Failure,
}

pub enum ContactsMsg {
    FilterContact(AttrValue),
    CleanupSearchResult,
    QueryFriends(QueryState<IndexMap<AttrValue, Friend>>),
    QueryGroups(QueryState<IndexMap<AttrValue, Group>>),
    ShowAddFriend,
    RecFriendShipReq(Rc<FriendShipState>),
    FriendListStateChanged(Rc<FriendListState>),
    QueryFriendship(usize),
    NewFriendClicked,
    ShowContextMenu((i32, i32), AttrValue, bool),
    RemoveFriend(Rc<RemoveFriendState>),
    AddFriend(Rc<AddFriendState>),
}

impl Component for Contacts {
    type Message = ContactsMsg;

    type Properties = ContactsProps;

    fn create(ctx: &Context<Self>) -> Self {
        // 查询联系人列表
        ctx.link()
            .send_message(ContactsMsg::QueryFriends(QueryState::Querying));
        ctx.link().send_future(async {
            let friends = db::friends().await.get_list().await.unwrap_or_default();
            ContactsMsg::QueryFriends(QueryState::Success(friends))
        });
        ctx.link().send_future(async {
            let group_repo = GroupRepo::new().await;
            let friends = group_repo.get_list().await.unwrap_or_default();
            ContactsMsg::QueryGroups(QueryState::Success(friends))
        });
        // 查询好友请求列表
        ctx.link().send_future(async {
            let count = db::friendships().await.get_unread_count().await;
            log::debug!("查询好友请求列表, 未读数量{}", count);
            ContactsMsg::QueryFriendship(count)
        });
        // register state
        let (_friendship_state, _listener) = ctx
            .link()
            .context(ctx.link().callback(ContactsMsg::RecFriendShipReq))
            .expect("need friend ship state");
        let (friend_state, _friend_listener) = ctx
            .link()
            .context(ctx.link().callback(ContactsMsg::FriendListStateChanged))
            .expect("need friend ship state");
        let (_remove_friend_state, _remove_friend_listener) = ctx
            .link()
            .context(ctx.link().callback(ContactsMsg::RemoveFriend))
            .expect("postcard friend_state needed");
        let (_add_friend_state, _add_friend_listener) = ctx
            .link()
            .context(ctx.link().callback(ContactsMsg::AddFriend))
            .expect("postcard friend_state needed");
        Self {
            friends: IndexMap::new(),
            result: IndexMap::new(),
            groups: IndexMap::new(),
            friendships_unread_count: 0,
            is_searching: false,
            is_add_friend: false,
            show_context_menu: false,
            _friendship_state,
            _listener,
            friend_state,
            _friend_listener,
            _remove_friend_state,
            _remove_friend_listener,
            _add_friend_state,
            _add_friend_listener,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ContactsMsg::FilterContact(pattern) => {
                self.is_searching = true;
                // 过滤联系人列表
                if pattern.is_empty() {
                    self.result.clear();
                } else {
                    self.friends.iter().for_each(|(key, item)| {
                        if item.name.contains(pattern.as_str()) {
                            self.result.insert((*key).clone(), (*item).clone());
                        }
                    });
                }
                true
            }
            // 清空搜索结果
            ContactsMsg::CleanupSearchResult => {
                self.is_searching = false;
                self.result.clear();
                true
            }
            ContactsMsg::QueryFriends(friends) => match friends {
                QueryState::Success(list) => {
                    self.friends = list;
                    true
                }
                // QueryState::Failure => {
                //     false
                // }
                QueryState::Querying => false,
            },
            ContactsMsg::ShowAddFriend => {
                self.is_add_friend = !self.is_add_friend;
                true
            }
            ContactsMsg::RecFriendShipReq(friendship) => {
                match friendship.state_type {
                    FriendShipStateType::Req => {
                        self.friendships_unread_count += 1;
                    }
                    FriendShipStateType::Res => {
                        let friend = friendship.friend.as_ref().unwrap().clone();
                        self.friends.insert(friend.friend_id.clone(), friend);
                    }
                }
                true
            }
            ContactsMsg::QueryFriendship(count) => {
                self.friendships_unread_count = count;
                true
            }
            ContactsMsg::FriendListStateChanged(state) => {
                self.friend_state = state;
                true
            }
            ContactsMsg::NewFriendClicked => {
                log::debug!("new friend clicked");
                self.friendships_unread_count = 0;
                // send friendship list event
                self.friend_state.state_change_event.emit(CurrentItem {
                    item_id: AttrValue::default(),
                    content_type: RightContentType::FriendShipList,
                });
                // clean unread count
                spawn_local(async {
                    let _ = db::friendships().await.clean_unread_count().await;
                });
                true
            }
            ContactsMsg::ShowContextMenu((_x, _y), _id, _is_mute) => {
                // event.prevent_default();
                self.show_context_menu = true;
                true
            }
            ContactsMsg::QueryGroups(groups) => match groups {
                QueryState::Success(list) => {
                    self.groups = list;
                    true
                }
                // QueryState::Failure => {
                //     false
                // }
                QueryState::Querying => false,
            },
            ContactsMsg::RemoveFriend(state) => {
                let mut friend_id = AttrValue::default();
                match state.type_ {
                    ItemType::Group => {
                        if let Some(item) = self.groups.shift_remove(&state.id) {
                            friend_id = item.id;
                        }
                    }
                    ItemType::Friend => {
                        if let Some(item) = self.friends.shift_remove(&state.id) {
                            friend_id = item.id;
                        }
                    }
                }
                if !friend_id.is_empty() && friend_id == self.friend_state.friend.item_id {
                    self.friend_state
                        .state_change_event
                        .emit(CurrentItem::default());
                    return true;
                }
                false
            }
            ContactsMsg::AddFriend(state) => {
                match state.item.type_ {
                    RightContentType::Friend => {
                        if let Some(friend) = state.item.friend.clone() {
                            self.friends.insert(friend.friend_id.clone(), friend);
                            return true;
                        }
                    }
                    RightContentType::Group => {
                        if let Some(group) = state.item.group.clone() {
                            self.groups.insert(group.id.clone(), group);
                            return true;
                        }
                    }
                    _ => {
                        return false;
                    }
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // 根据搜索结果显示联系人列表，
        // 如果是搜索状态，那么搜索结果为空时需要提示用户没有结果
        let oncontextmenu = ctx
            .link()
            .callback(|((x, y), id, is_mute)| ContactsMsg::ShowContextMenu((x, y), id, is_mute));
        let content = if self.is_searching {
            if self.result.is_empty() {
                html! {<div class="no-result">{"没有搜索结果"}</div>}
            } else {
                self.result
                    .iter()
                    .map(|item| get_list_item(item.1, oncontextmenu.clone()))
                    .collect::<Html>()
            }
        } else {
            self.friends
                .iter()
                .map(|item| get_list_item(item.1, oncontextmenu.clone()))
                .collect::<Html>()
        };
        let search_callback = ctx.link().callback(ContactsMsg::FilterContact);
        let clean_callback = ctx
            .link()
            .callback(move |_| ContactsMsg::CleanupSearchResult);
        let plus_click = ctx.link().callback(|_| ContactsMsg::ShowAddFriend);
        let friendship_click = ctx.link().callback(|_| ContactsMsg::NewFriendClicked);
        let groups_con = self
            .groups
            .iter()
            .map(|item| get_list_item(item.1, oncontextmenu.clone()))
            .collect::<Html>();
        html! {
            <div class="list-wrapper">
                {
                    if self.is_add_friend {
                        html!{
                            <AddFriend user_id={ctx.props().user_id.clone()} {plus_click}/>
                        }
                    } else {
                        html!{
                            <>
                                <TopBar components_type={ComponentType::Contacts} {search_callback} {clean_callback} {plus_click} />
                                <div class="contacts-list">
                                    <div class="new-friends" onclick={friendship_click}>
                                        {"新的朋友: "}
                                        if self.friendships_unread_count > 0{
                                            {self.friendships_unread_count}
                                        }
                                    </div>
                                    {groups_con}
                                    {content}
                                </div>
                            </>
                        }
                    }
                }
            </div>
        }
    }
}

fn get_list_item(
    item: &impl ItemInfo,
    oncontextmenu: Callback<((i32, i32), AttrValue, bool)>,
) -> Html {
    html! {
        <ListItem
            component_type={ComponentType::Contacts}
            props={
                CommonProps{
                    name:item.name(),
                    avatar:item.avatar(),
                    time:item.time(),
                    remark:item.remark().unwrap_or_default(),
                    id: item.id()
                }
            }
            unread_count={0}
            conv_type={item.get_type()}
            {oncontextmenu}
            mute={false}
            key={item.id().as_str()} />
    }
}
