use fluent::{FluentBundle, FluentResource};
use yew::prelude::*;

use i18n::{en_us, zh_cn, LanguageType};
use sandcat_sdk::api;
use sandcat_sdk::db;
use sandcat_sdk::model::user::UserWithMatchType;
use sandcat_sdk::model::ComponentType;
use utils::tr;

use crate::constant::NO_RESULT;
use crate::constant::SEARCH_PROMPT;
use crate::left::user_info::UserInfoCom;
use crate::top_bar::TopBar;

#[derive(Properties, PartialEq, Debug)]
pub struct AddFriendProps {
    pub plus_click: Callback<()>,
    pub user_id: AttrValue,
    pub avatar: AttrValue,
    pub nickname: AttrValue,
    pub lang: LanguageType,
}

pub struct AddFriend {
    result: Option<UserWithMatchType>,
    // 是否正在搜索
    is_searching: bool,
    i18n: FluentBundle<FluentResource>,
}

pub enum SearchState<T> {
    Searching,
    Success(T),
    Failure,
}

pub enum AddFriendMsg {
    SearchFriend(AttrValue),
    CleanupSearchResult,
    SearchFriends(Box<SearchState<Option<UserWithMatchType>>>),
    Cancel,
}

impl Component for AddFriend {
    type Message = AddFriendMsg;

    type Properties = AddFriendProps;

    fn create(ctx: &Context<Self>) -> Self {
        let res = match ctx.props().lang {
            LanguageType::ZhCN => zh_cn::ADD_FRIEND,
            LanguageType::EnUS => en_us::ADD_FRIEND,
        };
        let i18n = utils::create_bundle(res);
        Self {
            result: None,
            is_searching: false,
            i18n,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AddFriendMsg::SearchFriend(pattern) => {
                self.is_searching = true;
                let user_id = ctx.props().user_id.clone();
                ctx.link()
                    .send_message(AddFriendMsg::SearchFriends(Box::new(
                        SearchState::Searching,
                    )));
                ctx.link().send_future(async move {
                    // select local friend first
                    let friend = db::db_ins().friends.get(&pattern).await;
                    if !friend.friend_id.is_empty() {
                        return AddFriendMsg::SearchFriends(Box::new(SearchState::Success(Some(
                            UserWithMatchType::from(friend),
                        ))));
                    }
                    match api::users()
                        .search_friend(pattern.to_string(), user_id.as_str())
                        .await
                    {
                        Ok(mut user) => {
                            // check if user is already in friends
                            let user = match user {
                                Some(ref mut u) => {
                                    let friend = db::db_ins().friends.get(&u.id).await;
                                    if !friend.friend_id.is_empty() {
                                        u.is_friend = true;
                                    }
                                    user
                                }
                                None => None,
                            };
                            AddFriendMsg::SearchFriends(Box::new(SearchState::Success(user)))
                        }
                        Err(err) => {
                            log::error!("搜索用户错误:{:?}", err);
                            AddFriendMsg::SearchFriends(Box::new(SearchState::Failure))
                        }
                    }
                });
                true
            }
            // 清空搜索结果
            AddFriendMsg::CleanupSearchResult => {
                self.is_searching = false;
                self.result = None;
                true
            }
            AddFriendMsg::SearchFriends(friends) => match *friends {
                SearchState::Success(list) => {
                    self.result = list;
                    true
                }
                SearchState::Failure => {
                    log::debug!("query friends failure");
                    false
                }
                SearchState::Searching => false,
            },
            AddFriendMsg::Cancel => {
                ctx.props().plus_click.emit(());
                false
            }
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let content = if !self.is_searching {
            html! {
                <div class="hint">{tr!(self.i18n, SEARCH_PROMPT)}</div>
            }
        } else if self.result.is_none() {
            html! {<div class="no-result">{tr!(self.i18n, NO_RESULT)}</div>}
        } else {
            html! {
                <UserInfoCom
                    user_id={&ctx.props().user_id}
                    avatar={&ctx.props().avatar}
                    nickname={&ctx.props().nickname}
                    info={self.result.as_ref().unwrap().clone()}
                    lang={ctx.props().lang} />
            }
        };
        let search_callback = ctx.link().callback(AddFriendMsg::SearchFriend);
        let clean_callback = ctx
            .link()
            .callback(move |_| AddFriendMsg::CleanupSearchResult);
        let plus_click = ctx.link().callback(|_| AddFriendMsg::Cancel);

        html! {
            <>
                <TopBar
                    components_type={ComponentType::Default}
                    {search_callback}
                    {clean_callback}
                    {plus_click}
                    lang={ctx.props().lang} />
                <div class="contacts-list">
                    {content}
                </div>
            </>
        }
    }
}
