use fluent::{FluentBundle, FluentResource};
use web_sys::HtmlInputElement;
use yew::prelude::*;

use i18n::{en_us, zh_cn, LanguageType};
use icons::{PeoplePlusIcon, PlusIcon, SearchIcon};
use sandcat_sdk::{model::ComponentType, state::MobileState};
use utils::tr;
use yewdux::Dispatch;
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
    pub plus_click: Callback<()>,
    pub lang: LanguageType,
}

pub struct TopBar {
    search_node: NodeRef,
    i18n: FluentBundle<FluentResource>,
    is_mobile: bool,
}

pub enum TopBarMsg {
    SearchInputChanged(Event),
    SearchInputEnterListener(KeyboardEvent),
    PlusButtonClicked,
    SearchButtonClicked,
}

impl Component for TopBar {
    type Message = TopBarMsg;

    type Properties = TopBarProps;

    fn create(ctx: &Context<Self>) -> Self {
        let res = match ctx.props().lang {
            LanguageType::ZhCN => zh_cn::SEARCH_DOCK,
            LanguageType::EnUS => en_us::SEARCH_DOCK,
        };
        let i18n = utils::create_bundle(res);
        Self {
            i18n,
            search_node: NodeRef::default(),
            is_mobile: Dispatch::<MobileState>::global().get().is_mobile(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            // 搜索框输入事件
            TopBarMsg::SearchInputChanged(_) => {
                ctx.link().send_message(TopBarMsg::SearchButtonClicked);
                true
            }
            // 搜索框回车事件
            TopBarMsg::SearchInputEnterListener(_e) => {
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
            TopBarMsg::PlusButtonClicked => {
                ctx.props().plus_click.emit(());
                true
            }
            TopBarMsg::SearchButtonClicked => {
                let input: HtmlInputElement = self.search_node.cast().unwrap();
                ctx.props().search_callback.emit(input.value().into());
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
            ComponentType::Default => "default",
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
            ComponentType::Default => {
                html!({ tr!(self.i18n, "cancel") })
            }
            ComponentType::Setting => html!(),
        };
        let click_plus = ctx.link().callback(|_| TopBarMsg::PlusButtonClicked);
        let onclick = ctx.link().callback(|_| TopBarMsg::SearchButtonClicked);
        let plus_class = if self.is_mobile {
            "plus-icon-mobile"
        } else {
            "plus-icon"
        };
        html! {
            // 水平布局，从左到右分别为排序选项卡、搜索输入框、设置按钮
            <div class="top-bar">
                <div class="search">
                   <label /* for={id} */ class="search-icon" {onclick}>
                    <SearchIcon />
                    </label>
                   <input id={id} ref={self.search_node.clone()} class="search-input" type="search" placeholder={tr!(self.i18n, "search")} {onchange} {onkeydown} />
                </div>
                <div class={plus_class} onclick={click_plus}>
                    {icon}
                </div>
            </div>
        }
    }
}
