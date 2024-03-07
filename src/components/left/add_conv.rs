use gloo::utils::document;
use indexmap::IndexMap;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::HtmlInputElement;
use web_sys::NodeList;
use yew::prelude::*;

use crate::api;
use crate::db::group::GroupRepo;
use crate::db::group_members::GroupMembersRepo;
use crate::model::group::Group;
use crate::model::group::GroupMember;
use crate::model::group::GroupRequest;
use crate::{db::friend::FriendRepo, model::friend::Friend};

#[derive(Debug, Default)]
pub struct AddConv {
    data: IndexMap<AttrValue, Friend>,
    querying: bool,
    err: JsValue,
}

pub enum AddConvMsg {
    Add,
    Close,
    QueryFriends(QueryStatus<IndexMap<AttrValue, Friend>>),
    SubmitGroupMembers(Group),
    SubmitEmpty,
    RequestCreateGroupFail(JsValue),
}
#[derive(Debug, Clone)]
pub enum QueryStatus<T> {
    // 正在查询
    Querying,
    // 查询成功
    Success(T),
    // 查询失败
    Fail(JsValue),
}

#[derive(Properties, Clone, PartialEq)]
pub struct AddConvProps {
    pub user_id: AttrValue,
    pub close_back: Callback<()>,
    pub submit_back: Callback<Group>,
}

impl Component for AddConv {
    type Message = AddConvMsg;

    type Properties = AddConvProps;

    fn create(ctx: &Context<Self>) -> Self {
        // query friend list
        ctx.link()
            .send_message(AddConvMsg::QueryFriends(QueryStatus::Querying));
        ctx.link().send_future(async {
            match FriendRepo::new().await.get_list().await {
                Ok(friends) => AddConvMsg::QueryFriends(QueryStatus::Success(friends)),
                Err(err) => AddConvMsg::QueryFriends(QueryStatus::Fail(err)),
            }
        });
        Self {
            ..Default::default()
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AddConvMsg::Add => {
                // get selected checkbox value
                match document().query_selector_all("input[type='checkbox']:checked") {
                    Ok(nodes) => {
                        self.get_group_mems(ctx, nodes);
                    }
                    Err(_) => {
                        ctx.props().close_back.emit(());
                    }
                }

                false
            }
            AddConvMsg::QueryFriends(result) => {
                match result {
                    QueryStatus::Querying => self.querying = true,
                    QueryStatus::Success(friends) => {
                        self.data = friends;
                        self.querying = false;
                    }
                    QueryStatus::Fail(err) => {
                        self.querying = false;
                        self.err = err;
                    }
                }
                true
            }
            AddConvMsg::Close => {
                ctx.props().close_back.emit(());
                false
            }
            AddConvMsg::SubmitGroupMembers(g) => {
                ctx.props().submit_back.emit(g);
                false
            }
            AddConvMsg::RequestCreateGroupFail(err) => {
                log::error!("request server to create group error: {:?}", err);
                false
            }
            AddConvMsg::SubmitEmpty => {
                ctx.props().close_back.emit(());
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let mut content = html!();
        if self.querying {
            content = html!(<div>{"正在查询..."}</div>)
        } else if !self.data.is_empty() {
            content = html! {
                <fieldset>
                    <legend>{"SELECT FRIENDS"}</legend>
                    {
                        self.data.iter().map(|(index,item)| {
                            let mut name = item.name.clone();
                            if item.remark.is_some(){
                                name = item.remark.as_ref().unwrap().clone();
                            }

                           index.to_string();
                            html!{
                                <div class="item" key={index.to_string()}>
                                    <input type="checkbox" id={index.to_string()} name="friend" value={index.to_string()} />
                                    <label for={index.to_string()}  class="item-card">
                                        <img src={item.avatar.clone()}/>
                                        {name}
                                    </label>
                                </div>
                            }

                        }).collect::<Html>()
                    }
                </fieldset>
            }
        } else if !self.err.is_null() {
            content = html!(<div>{format!("查询出错{:?}", self.err)}</div>)
        }
        let submit = ctx.link().callback(|_| AddConvMsg::Add);
        let close = ctx.link().callback(|_| AddConvMsg::Close);

        html! {
            <div class="add-conv box-shadow">
                {content}
                <div class="add-conv-actions">
                    <div onclick={submit} >{"submit"}</div>
                    <div onclick={close} >{"cancel"}</div>
                </div>
            </div>
        }
    }
}

impl AddConv {
    fn get_group_mems(&self, ctx: &Context<Self>, nodes: NodeList) {
        let user_id = ctx.props().user_id.clone();
        ctx.link().send_future(async move {
            let mut values = Vec::with_capacity(nodes.length() as usize);
            let mut ids = Vec::with_capacity(nodes.length() as usize);
            let mut avatar = Vec::with_capacity(nodes.length() as usize);
            let mut group_name = String::new();
            for i in 0..nodes.length() {
                let node = nodes.item(i).unwrap();
                if let Ok(node) = node.dyn_into::<HtmlInputElement>() {
                    let value = node.value();
                    if !value.is_empty() {
                        let friend = FriendRepo::new()
                            .await
                            .get_friend(value.clone().into())
                            .await;
                        if !friend.id.is_empty() {
                            ids.push(value);
                            let mut name = friend.name.clone();
                            if friend.remark.is_some() {
                                name = friend.remark.as_ref().unwrap().clone();
                            }
                            group_name.push_str(name.as_str());
                            avatar.push(friend.avatar.clone().to_string());
                            values.push(GroupMember::from(friend));
                        }
                    }
                }
            }
            if ids.is_empty() {
                return AddConvMsg::SubmitEmpty;
            }
            group_name.push_str("Group");
            let group_req = GroupRequest {
                owner: user_id.to_string(),
                avatar: avatar.join(","),
                group_name,
                members_id: ids,
                id: String::new(),
            };
            // send create request
            match api::group::create_group(group_req).await {
                Ok(g) => {
                    if let Err(err) = GroupRepo::new().await.put(&g).await {
                        return AddConvMsg::RequestCreateGroupFail(err);
                    }
                    for v in values.iter_mut() {
                        v.group_id = g.id.clone();
                        if let Err(e) = GroupMembersRepo::new().await.put(v).await {
                            log::error!("save group member error: {:?}", e);
                            continue;
                        }
                    }
                    AddConvMsg::SubmitGroupMembers(g)
                }
                Err(err) => AddConvMsg::RequestCreateGroupFail(err),
            }
        });
    }
}
