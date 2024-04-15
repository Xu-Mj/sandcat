pub mod add_friend;
pub mod contacts;
pub mod conv_com;
pub mod list_item;
pub mod right_click_panel;
pub mod top;
pub mod user_info;

use std::rc::Rc;

use yew::prelude::*;

use crate::components::left::contacts::Contacts;
use crate::components::left::conv_com::Chats;
use crate::components::left::top::Top;
use crate::model::ComponentType;
use crate::pages::AppState;

#[derive(Properties, PartialEq, Debug)]
pub struct LeftProps {
    pub user_id: AttrValue,
}

pub enum LeftMsg {
    ContextChanged(Rc<AppState>),
    RequestState,
}
pub struct Left {
    pub state: Rc<AppState>,
    _context_listener: ContextHandle<Rc<AppState>>,
}

impl Component for Left {
    type Message = LeftMsg;
    type Properties = LeftProps;

    fn create(ctx: &Context<Self>) -> Self {
        // 向服务器查询会话列表、联系人列表
        ctx.link().send_future(async { LeftMsg::RequestState });
        let (state, _context_listener) = ctx
            .link()
            .context(ctx.link().callback(LeftMsg::ContextChanged))
            .expect("init state error");
        Self {
            state,
            _context_listener,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            LeftMsg::ContextChanged(state) => {
                self.state = state;
                true
            }
            LeftMsg::RequestState => false,
        }
    }

    // 左侧面板总布局,包含顶部选项栏、左侧列表、
    // 点击顶部选项栏切换左侧列表
    fn view(&self, _ctx: &Context<Self>) -> Html {
        let mut classes = "slider";
        match self.state.component_type {
            ComponentType::Contacts => {
                classes = "slider move-left";
            }
            ComponentType::Messages => {
                classes = "slider ";
                // <Messages />
            }
            ComponentType::Setting => {}
        };
        html! {
            <div class="left-container">
                // 左侧顶部组件：包含头像以及功能切换
                <Top avatar={self.state.login_user.avatar.clone()} name={self.state.login_user.name.clone()} />
                <div class="left-down">
                    <div class={classes}>
                    <Chats user_id={_ctx.props().user_id.clone()}
                            avatar={self.state.login_user.avatar.clone()} />
                    <Contacts user_id={self.state.login_user.id.clone()}/>
                    </div>
                </div>
            </div>
        }
    }
}
