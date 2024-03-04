use std::collections::BTreeMap;

use yew::prelude::*;

use crate::model::friend::Friend;

pub struct AddConv {
    data: BTreeMap<AttrValue, Friend>,
}

pub enum AddConvMsg {
    Add,
}

#[derive(Properties, Clone, PartialEq)]
pub struct AddConvProps {
    pub close_back: Callback<()>,
}

impl Component for AddConv {
    type Message = AddConvMsg;

    type Properties = AddConvProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            data: BTreeMap::new(),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AddConvMsg::Add => {
                self.data.insert("1".into(), Friend::default());
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| AddConvMsg::Add);
        html! {
            <div class="add-conv">
                {"hello world"}
                <div onclick={submit} >{"submit"}</div>
            </div>
        }
    }
}
