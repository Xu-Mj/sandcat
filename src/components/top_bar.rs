#![allow(dead_code)]
use std::rc::Rc;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::icons::{PeoplePlusIcon, PlusIcon, SearchIcon};
use crate::pages::AppState;
use crate::pages::ComponentType;

/// 左侧组件顶部选项栏
/// 包含搜索和设置按钮以及一个排序按钮
/// 可在联系人与消息列表中进行复用
/// 接收一个类型参数，用于指定当前被哪个组件复用
///
///
#[derive(Properties, Clone, PartialEq)]
pub struct TopBarProps {
    #[prop_or_default]
    pub components_type: ComponentType,
    pub search_callback: Callback<AttrValue>,
    pub clean_callback: Callback<AttrValue>,
    pub plus_click: Callback<bool>,
}

pub struct TopBar {
    search_node: NodeRef,
    search_value: AttrValue,
    state: Rc<AppState>,
    _listener: ContextHandle<Rc<AppState>>,
}

pub enum TopBarMsg {
    SearchInputChanged(Event),
    SearchInputEnterListener(KeyboardEvent),
    StateChanged(Rc<AppState>),
    PlusButtonClicked,
    SearchButtonClicked,
}

impl Component for TopBar {
    type Message = TopBarMsg;

    type Properties = TopBarProps;

    fn create(ctx: &Context<Self>) -> Self {
        let (state, _listener) = ctx
            .link()
            .context(ctx.link().callback(TopBarMsg::StateChanged))
            .expect("expect state");
        Self {
            search_node: NodeRef::default(),
            search_value: "".into(),
            state,
            _listener,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            // 搜索框输入事件
            TopBarMsg::SearchInputChanged(e) => {
                let input: HtmlInputElement = e.target_unchecked_into();
                self.search_value = input.value().into();
                true
            }
            // 搜索框回车事件
            TopBarMsg::SearchInputEnterListener(e) => {
                // web_sys::console::log_1(&e);
                // if e.key() == "Enter" {
                // let input: HtmlInputElement = e.target_unchecked_into();
                // self.search_value = input.value().into();
                // let search_value = self.search_value.clone();
                // ctx.props().search_callback.emit(search_value.clone());
                // } else if e.key() == "Escape" {
                // let input: HtmlInputElement = e.target_unchecked_into();
                // input.set_value("");
                // self.search_value = AttrValue::default();
                // ctx.props().clean_callback.emit(AttrValue::default());
                // }
                // true
                false
            }
            TopBarMsg::StateChanged(state) => {
                self.state = state;
                false
            }
            TopBarMsg::PlusButtonClicked => {
                ctx.props().plus_click.emit(true);
                true
            }
            TopBarMsg::SearchButtonClicked => {
                let input: HtmlInputElement = self.search_node.cast().unwrap();
                self.search_value = input.value().into();
                let search_value = self.search_value.clone();
                ctx.props().search_callback.emit(search_value.clone());
                true
            }
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        // input 输入框事件
        let onchange = ctx
            .link()
            .callback(move |e: Event| TopBarMsg::SearchInputChanged(e));
        // 按下回车键搜索，按下esc清空
        let onkeydown = ctx
            .link()
            .callback(move |e: KeyboardEvent| TopBarMsg::SearchInputEnterListener(e));
        let id = match ctx.props().components_type {
            ComponentType::Contacts => "contacts",
            ComponentType::Messages => "messages",
            ComponentType::Setting => "setting",
        };
        let icon = match ctx.props().components_type {
            ComponentType::Contacts => {
                html! {
                    <PeoplePlusIcon/>
                }
            }
            ComponentType::Messages => {
                html!(<PlusIcon />)
            }
            ComponentType::Setting => {
                html!({ "取消" })
            }
        };
        let click_plus = ctx.link().callback(|_| TopBarMsg::PlusButtonClicked);
        let onclick = ctx
            .link()
            .callback(move |_| TopBarMsg::SearchButtonClicked);
        html! {
            // 水平布局，从左到右分别为排序选项卡、搜索输入框、设置按钮
            <div class="top-bar">
                <div class="search">
                   <label for={id} class="search-icon" {onclick}>
                    <SearchIcon />
                    </label>
                   <input id={id} ref={self.search_node.clone()} class="search-input" type="search" placeholder="搜索" {onchange} {onkeydown} />
                </div>
                <div class="setting-button" onclick={click_plus}>
                    {icon}
                </div>
            </div>
        }
    }
}
