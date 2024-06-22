use fluent::FluentBundle;
use fluent::FluentResource;
use gloo::utils::document;
use indexmap::IndexMap;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use i18n::en_us;
use i18n::zh_cn;
use i18n::LanguageType;
use sandcat_sdk::db;
use sandcat_sdk::error::Error;
use sandcat_sdk::model::friend::Friend;
use sandcat_sdk::state::MobileState;
use utils::tr;
use yewdux::Dispatch;

use crate::constant::CANCEL;
use crate::constant::EMPTY_RESULT;
use crate::constant::ERROR;
use crate::constant::QUERYING;
use crate::constant::SELECT_FRIENDS;
use crate::constant::SUBMIT;

pub struct SelectFriendList {
    data: IndexMap<AttrValue, Friend>,
    querying: bool,
    err: Option<Error>,
    i18n: FluentBundle<FluentResource>,
    is_mobile: bool,
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
    Fail(Error),
}

#[derive(Properties, Clone, PartialEq)]
pub struct AddConvProps {
    // 该排除一个用户呢还是排除多个？
    #[prop_or_default]
    pub except: AttrValue,
    pub close_back: Callback<()>,
    pub submit_back: Callback<Vec<String>>,
    pub lang: LanguageType,
}

impl Component for SelectFriendList {
    type Message = AddConvMsg;

    type Properties = AddConvProps;

    fn create(ctx: &Context<Self>) -> Self {
        // query friend list
        ctx.link()
            .send_message(AddConvMsg::QueryFriends(QueryStatus::Querying));
        ctx.link().send_future(async {
            match db::db_ins().friends.get_list().await {
                Ok(friends) => AddConvMsg::QueryFriends(QueryStatus::Success(friends)),
                Err(err) => AddConvMsg::QueryFriends(QueryStatus::Fail(err)),
            }
        });
        let res = match ctx.props().lang {
            LanguageType::ZhCN => zh_cn::SELECT_FRIENDS,
            LanguageType::EnUS => en_us::SELECT_FRIENDS,
        };
        let i18n = utils::create_bundle(res);
        Self {
            i18n,
            data: IndexMap::new(),
            querying: false,
            err: None,
            is_mobile: Dispatch::<MobileState>::global().get().is_mobile(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AddConvMsg::Add => {
                // get selected checkbox value
                match document().query_selector_all("input[type='checkbox']:checked") {
                    Ok(nodes) => {
                        if nodes.length() == 0 {
                            ctx.props().close_back.emit(());
                            return false;
                        }
                        let mut v = Vec::with_capacity(nodes.length() as usize);
                        for i in 0..nodes.length() {
                            if let Ok(node) = nodes.item(i).unwrap().dyn_into::<HtmlInputElement>()
                            {
                                v.push(node.value());
                            };
                        }
                        ctx.props().submit_back.emit(v);
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
                    QueryStatus::Success(mut friends) => {
                        friends.shift_remove(&ctx.props().except);
                        self.data = friends;
                        self.querying = false;
                    }
                    QueryStatus::Fail(err) => {
                        self.querying = false;
                        self.err = Some(err);
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
        let class = if self.is_mobile {
            "add-conv add-conv-size-mobile"
        } else {
            "add-conv add-conv-size box-shadow"
        };
        let mut content = html!(<p class="empty-result">{tr!(self.i18n, EMPTY_RESULT)}</p>);
        if self.querying {
            content = html!(<div>{tr!(self.i18n, QUERYING)}</div>)
        } else if !self.data.is_empty() {
            content = self.data.iter().map(|(index,item)| {
                        let mut name = item.name.clone();
                        if item.remark.is_some(){
                            name = item.remark.as_ref().unwrap().clone();
                        }

                        index.to_string();
                        html!{
                            <div class="item" key={index.to_string()}>
                                <input type="checkbox" id={index.to_string()} name="friend" value={index.to_string()} />
                                <label for={index.to_string()}  class="item-card">
                                    <img src={utils::get_avatar_url(&item.avatar)}/>
                                    {name}
                                </label>
                            </div>
                        }

                    }).collect::<Html>()
        } else if self.err.is_some() {
            content = html!(<div>{format!("{}{:?}", tr!(self.i18n, ERROR), self.err)}</div>)
        }
        let submit = ctx.link().callback(|_| AddConvMsg::Add);
        let close = ctx.link().callback(|_| AddConvMsg::Close);

        html! {
            <div {class}>
                <fieldset>
                    <legend>{tr!(self.i18n, SELECT_FRIENDS)}</legend>
                    {content}
                </fieldset>
                <div class="add-conv-actions">
                    <div onclick={submit} >{tr!(self.i18n, SUBMIT)}</div>
                    <div onclick={close} >{tr!(self.i18n, CANCEL)}</div>
                </div>
            </div>
        }
    }
}
