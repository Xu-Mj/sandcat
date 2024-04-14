use fluent::{FluentBundle, FluentResource};
use yew::prelude::*;

use crate::components::left::user_info::UserInfoCom;
use crate::i18n::{en_us, zh_cn, LanguageType};
use crate::model::user::UserWithMatchType;
use crate::{api, tr, utils};
use crate::{components::top_bar::TopBar, model::ComponentType};

#[derive(Properties, PartialEq, Debug)]
pub struct AddFriendProps {
    pub plus_click: Callback<()>,
    pub user_id: AttrValue,
    pub lang: LanguageType,
}

pub struct AddFriend {
    // 维护一个查询结果集
    pub result: Vec<UserWithMatchType>,
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
    SearchFriends(SearchState<Vec<UserWithMatchType>>),
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
            result: vec![],
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
                    .send_message(AddFriendMsg::SearchFriends(SearchState::Searching));
                ctx.link().send_future(async move {
                    match api::users()
                        .search_friend(pattern.to_string(), user_id.as_str())
                        .await
                    {
                        Ok(list) => AddFriendMsg::SearchFriends(SearchState::Success(list)),
                        Err(err) => {
                            log::error!("搜索用户错误:{:?}", err);
                            AddFriendMsg::SearchFriends(SearchState::Failure)
                        }
                    }
                });
                true
            }
            // 清空搜索结果
            AddFriendMsg::CleanupSearchResult => {
                self.is_searching = false;
                self.result.clear();
                true
            }
            AddFriendMsg::SearchFriends(friends) => match friends {
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
        let content = if self.result.is_empty() {
            html! {<div class="no-result">{tr!(self.i18n, "no_result")}</div>}
        } else {
            self.result
                .iter()
                .map(|item| {
                    html! {
                        <UserInfoCom info={item.clone()} key={item.id.clone().as_str()} />
                    }
                })
                .collect::<Html>()
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
