use fluent::{FluentBundle, FluentResource};
use indexmap::IndexMap;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yewdux::Dispatch;

use i18n::{en_us, zh_cn, LanguageType};
use sandcat_sdk::db;
use sandcat_sdk::model::group::Group;
use sandcat_sdk::model::{CurrentItem, FriendShipStateType, ItemInfo, RightContentType};
use sandcat_sdk::state::MobileState;
use sandcat_sdk::state::{
    AddFriendState, FriendListState, FriendShipState, I18nState, ItemType, RemoveFriendState,
    UnreadState,
};
use sandcat_sdk::state::{ComponentTypeState, RefreshMsgListState};
use sandcat_sdk::{
    model::friend::Friend,
    model::{CommonProps, ComponentType},
};
use utils::tr;

use crate::constant::NEW_FRIENDS;
use crate::constant::NO_RESULT;
use crate::left::add_friend::AddFriend;
use crate::{left::list_item::ListItem, top_bar::TopBar};

#[derive(Properties, PartialEq, Debug)]
pub struct ContactsProps {
    pub user_id: AttrValue,
    pub avatar: AttrValue,
    pub nickname: AttrValue,
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
    i18n: FluentBundle<FluentResource>,
    _fs_dis: Dispatch<FriendShipState>,
    friend_state: Rc<FriendListState>,
    friend_dispatch: Dispatch<FriendListState>,
    _remove_friend_dis: Dispatch<RemoveFriendState>,
    _add_friend_dis: Dispatch<AddFriendState>,
    lang_state: Rc<I18nState>,
    _lang_dispatch: Dispatch<I18nState>,
    _refresh_dis: Dispatch<RefreshMsgListState>,
    touch_start: i32,
    is_mobile: bool,
}

pub enum QueryState<T> {
    Querying,
    Success(T),
    // Failure,
}

type QueryProps = (
    IndexMap<AttrValue, Friend>,
    IndexMap<AttrValue, Group>,
    usize,
);

pub enum ContactsMsg {
    FilterContact(AttrValue),
    CleanupSearchResult,
    QueryList(QueryProps),
    ShowAddFriend,
    RecFriendShipReq(Rc<FriendShipState>),
    FriendListStateChanged(Rc<FriendListState>),
    QueryFriendship(usize),
    NewFriendClicked,
    ShowContextMenu((i32, i32), AttrValue, bool, bool),
    RemoveFriend(Rc<RemoveFriendState>),
    AddFriend(Rc<AddFriendState>),
    SwitchLanguage(Rc<I18nState>),
    OnTouchStart(TouchEvent),
    OnTouchEnd(TouchEvent),
    RefreshList,
}

impl Component for Contacts {
    type Message = ContactsMsg;

    type Properties = ContactsProps;

    fn create(ctx: &Context<Self>) -> Self {
        // query friend/group/count
        Self::query_list(ctx);

        // register state
        let fs_dis =
            Dispatch::global().subscribe_silent(ctx.link().callback(ContactsMsg::RecFriendShipReq));

        let friend_dispatch =
            Dispatch::global().subscribe(ctx.link().callback(ContactsMsg::FriendListStateChanged));

        let _remove_friend_dis =
            Dispatch::global().subscribe_silent(ctx.link().callback(ContactsMsg::RemoveFriend));
        let _add_friend_dis =
            Dispatch::global().subscribe_silent(ctx.link().callback(ContactsMsg::AddFriend));

        let _refresh_dis =
            Dispatch::global().subscribe_silent(ctx.link().callback(|_| ContactsMsg::RefreshList));

        let lang_dispatch =
            Dispatch::global().subscribe(ctx.link().callback(ContactsMsg::SwitchLanguage));
        let lang_state = lang_dispatch.get();
        let res = match lang_state.lang {
            LanguageType::ZhCN => zh_cn::CONTACTS,
            LanguageType::EnUS => en_us::CONTACTS,
        };
        let i18n = utils::create_bundle(res);
        Self {
            friends: IndexMap::new(),
            result: IndexMap::new(),
            groups: IndexMap::new(),
            friendships_unread_count: 0,
            is_searching: false,
            is_add_friend: false,
            show_context_menu: false,
            i18n,
            touch_start: 0,
            is_mobile: MobileState::is_mobile(),
            _fs_dis: fs_dis,
            friend_state: friend_dispatch.get(),
            friend_dispatch,
            _remove_friend_dis,
            _add_friend_dis,
            lang_state,
            _lang_dispatch: lang_dispatch,
            _refresh_dis,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
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
            ContactsMsg::ShowAddFriend => {
                self.is_add_friend = !self.is_add_friend;
                true
            }
            ContactsMsg::RecFriendShipReq(friendship) => {
                match friendship.state_type {
                    FriendShipStateType::Req => {
                        ctx.link().send_future(async {
                            let count = db::db_ins()
                                .friendships
                                .get_unread_count()
                                .await
                                .unwrap_or_default();
                            Dispatch::<UnreadState>::global()
                                .reduce_mut(|s| s.contacts_count = count);
                            ContactsMsg::QueryFriendship(count)
                        });
                    }
                    FriendShipStateType::RecResp | FriendShipStateType::Res => {
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
                self.friendships_unread_count = 0;
                // send friendship list event
                self.friend_dispatch.reduce_mut(|s| {
                    s.friend = CurrentItem {
                        item_id: AttrValue::default(),
                        content_type: RightContentType::FriendShipList,
                    };
                });
                // clean unread count
                spawn_local(async {
                    if let Ok(msg_id) = db::db_ins().friendships.clean_unread_count().await {
                        if !msg_id.is_empty() {
                            Dispatch::<UnreadState>::global().reduce_mut(|s| s.contacts_count = 0);
                            // send read request
                            // let user = AppState::get();
                            // let user_id = user.login_user.id.as_str();
                            // if let Err(err) = api::messages().del_msg(user_id, msg_id).await {
                            //     log::error!("{:?}", err);
                            // }
                        }
                    }
                });
                true
            }
            ContactsMsg::ShowContextMenu((_x, _y), _id, _is_mute, _pin) => {
                // event.prevent_default();
                self.show_context_menu = true;
                true
            }
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
                            friend_id = item.friend_id;
                        }
                    }
                }

                if !friend_id.is_empty() && friend_id == self.friend_state.friend.item_id {
                    self.friend_dispatch.reduce_mut(|s| {
                        let friend = CurrentItem::default();
                        // current_item::save_friend(&friend).unwrap();
                        s.friend = friend;
                    });
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
            ContactsMsg::SwitchLanguage(state) => {
                self.lang_state = state;
                true
            }
            ContactsMsg::OnTouchStart(event) => {
                if let Some(touch) = event.touches().get(0) {
                    log::debug!("TouchStart: {}", touch.client_x());
                    self.touch_start = touch.client_x();
                };
                false
            }
            ContactsMsg::OnTouchEnd(event) => {
                // we can't use the .touches() to get the touch end
                // should use the changed_touches()
                if let Some(touch) = event.changed_touches().get(0) {
                    log::debug!("TouchEnd: {}", touch.client_x());
                    if touch.client_x() - self.touch_start > 50 {
                        // go to contacts
                        Dispatch::<ComponentTypeState>::global()
                            .reduce_mut(|s| s.component_type = ComponentType::Messages);
                    }
                }
                self.touch_start = 0;
                false
            }
            ContactsMsg::RefreshList => {
                log::debug!("refresh list in contacts");
                Self::query_list(ctx);
                false
            }
            ContactsMsg::QueryList((friends, groups, count)) => {
                self.friends = friends;
                self.groups = groups;
                self.friendships_unread_count = count;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let (ontouchstart, ontouchend) = if self.is_mobile {
            (
                Some(ctx.link().callback(ContactsMsg::OnTouchStart)),
                Some(ctx.link().callback(ContactsMsg::OnTouchEnd)),
            )
        } else {
            (None, None)
        };
        // 根据搜索结果显示联系人列表，
        // 如果是搜索状态，那么搜索结果为空时需要提示用户没有结果
        let oncontextmenu = ctx.link().callback(|((x, y), id, is_mute, is_pined)| {
            ContactsMsg::ShowContextMenu((x, y), id, is_mute, is_pined)
        });
        let content = if self.is_searching {
            if self.result.is_empty() {
                html! {<div class="no-result">{tr!(self.i18n, NO_RESULT)}</div>}
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
            <div class="list-wrapper" {ontouchstart} {ontouchend}>
                {
                    if self.is_add_friend {
                        html!{
                            <AddFriend
                                user_id={&ctx.props().user_id}
                                avatar={&ctx.props().avatar}
                                nickname={&ctx.props().nickname}
                                {plus_click}
                                lang={self.lang_state.lang}/>
                        }
                    } else {
                        html!{
                            <>
                                <TopBar
                                    components_type={ComponentType::Contacts}
                                    {search_callback}
                                    {clean_callback}
                                    {plus_click}
                                    lang={self.lang_state.lang} />
                                <div class="contacts-list">
                                    <div class="new-friends" onclick={friendship_click}>
                                        {tr!(self.i18n, NEW_FRIENDS)}
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

impl Contacts {
    fn query_list(ctx: &Context<Self>) {
        ctx.link().send_future(async move {
            let friends = db::db_ins().friends.get_list().await.unwrap_or_default();
            let groups = db::db_ins().groups.get_list().await.unwrap_or_default();
            let count = db::db_ins()
                .friendships
                .get_unread_count()
                .await
                .unwrap_or_default();
            ContactsMsg::QueryList((friends, groups, count))
        });
    }
}

fn get_list_item(
    item: &impl ItemInfo,
    oncontextmenu: Callback<((i32, i32), AttrValue, bool, bool)>,
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
            pined={false}
            key={item.id().as_str()} />
    }
}
