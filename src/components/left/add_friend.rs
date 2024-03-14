use yew::prelude::*;

use crate::api::user::search_friend;
use crate::components::left::user_info::UserInfoCom;
use crate::model::user::User;
use crate::{components::top_bar::TopBar, model::ComponentType};

#[derive(Properties, PartialEq, Debug)]
pub struct AddFriendProps {
    pub plus_click: Callback<()>,
    pub user_id: AttrValue,
}

pub struct AddFriend {
    // 维护一个查询结果集
    pub result: Vec<User>,
    // 是否正在搜索
    pub is_searching: bool,
}

pub enum SearchState<T> {
    Searching,
    Success(T),
    Failure,
}

pub enum AddFriendMsg {
    SearchFriend(AttrValue),
    CleanupSearchResult,
    SearchFriends(SearchState<Vec<User>>),
    Cancel,
}

impl Component for AddFriend {
    type Message = AddFriendMsg;

    type Properties = AddFriendProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            result: vec![],
            is_searching: false,
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
                    match search_friend(pattern.to_string(), user_id).await {
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
            html! {<div class="no-result">{"没有搜索结果"}</div>}
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
                <TopBar components_type={ComponentType::Setting} {search_callback} {clean_callback} {plus_click} />
                <div class="contacts-list">
                    {content}
                </div>
            </>
        }
    }
}
