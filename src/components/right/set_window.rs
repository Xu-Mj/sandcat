use yew::prelude::*;

use crate::model::RightContentType;

pub struct SetWindow {}

pub enum SetWindowMsg {}

#[derive(Properties, PartialEq)]
pub struct SetWindowProps {
    pub conv_type: RightContentType,
    pub id: AttrValue,
    pub avatar: AttrValue,
}

impl Component for SetWindow {
    type Message = SetWindowMsg;

    type Properties = SetWindowProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <div class="set-window">
                {"set window"}
            </div>
        }
    }
}
