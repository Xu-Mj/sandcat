use fluent::{FluentBundle, FluentResource};
use yew::prelude::*;

use crate::components::left::user_info::UserInfoCom;
use crate::i18n::{en_us, zh_cn, LanguageType};
use crate::model::user::UserWithMatchType;
use crate::{api, db, tr, utils};
use crate::{components::top_bar::TopBar, model::ComponentType};

#[derive(Properties, PartialEq, Debug)]
pub struct AddFriendProps {
    pub plus_click: Callback<()>,
    pub user_id: AttrValue,
    pub lang: LanguageType,
}

pub struct AddFriend {
    // 维护一个查询结果集
    pub result: Option<UserWithMatchType>,
    // 是否正在搜索
    pub is_searching: bool,
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
                    let friend = db::friends().await.get(&pattern).await;
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
                                    let friend = db::friends().await.get(&u.id).await;
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
                <div>{tr!(self.i18n, "search_prompt")}</div>
            }
        } else if self.result.is_none() {
            html! {<div class="no-result">{tr!(self.i18n, "no_result")}</div>}
        } else {
            html! {
                <UserInfoCom info={self.result.as_ref().unwrap().clone()}  lang={ctx.props().lang} />
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
                    components_type={ComponentType::Setting}
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
