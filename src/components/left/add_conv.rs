use gloo::utils::document;
use indexmap::IndexMap;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::HtmlInputElement;
use yew::prelude::*;

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
    pub close_back: Callback<()>,
    pub submit_back: Callback<Vec<String>>,
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
                        let mut values = Vec::with_capacity(nodes.length() as usize);
                        for i in 0..nodes.length() {
                            let node = nodes.item(i).unwrap();
                            if let Ok(node) = node.dyn_into::<HtmlInputElement>() {
                                let value = node.value();
                                values.push(value);
                            }
                        }
                        ctx.props().submit_back.emit(values);
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
