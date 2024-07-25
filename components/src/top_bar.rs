use fluent::{FluentBundle, FluentResource};
use web_sys::HtmlInputElement;
use yew::prelude::*;

use i18n::{en_us, zh_cn, LanguageType};
use icons::{PeoplePlusIcon, PlusIcon, SearchIcon};
use sandcat_sdk::{model::ComponentType, state::MobileState};
use utils::tr;

use crate::constant::{CANCEL, SEARCH};
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
    SearchInputChanged,
    PlusButtonClicked,
    SearchButtonClicked,
    SubmitSearch(SubmitEvent),
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
            is_mobile: MobileState::is_mobile(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TopBarMsg::SearchInputChanged => {
                self.send_value(ctx);
                false
            }
            TopBarMsg::PlusButtonClicked => {
                ctx.props().plus_click.emit(());
                false
            }
            TopBarMsg::SearchButtonClicked => {
                self.send_value(ctx);
                false
            }
            TopBarMsg::SubmitSearch(event) => {
                event.prevent_default();
                self.send_value(ctx);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let id = match ctx.props().components_type {
            ComponentType::Contacts => "contacts",
            ComponentType::Messages => "messages",
            ComponentType::Setting => "setting",
            ComponentType::Default => "default",
        };

        let icon = match ctx.props().components_type {
            ComponentType::Contacts => {
                html! (<PeoplePlusIcon/>)
            }
            ComponentType::Messages => {
                html!(<PlusIcon />)
            }
            ComponentType::Default => {
                html!({ tr!(self.i18n, CANCEL) })
            }
            ComponentType::Setting => html!(),
        };

        let onchange = ctx.link().callback(|_| TopBarMsg::SearchInputChanged);
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
                <form class="search" onsubmit={ctx.link().callback(TopBarMsg::SubmitSearch)}>
                   <label /* for={id} */ class="search-icon" {onclick}>
                    <SearchIcon />
                    </label>
                   <input
                        {id}
                        ref={self.search_node.clone()}
                        class="search-input"
                        type="search"
                        placeholder={tr!(self.i18n, SEARCH)}
                        {onchange} />
                </form>
                <div class={plus_class} onclick={click_plus}>
                    {icon}
                </div>
            </div>
        }
    }
}

impl TopBar {
    pub fn send_value(&self, ctx: &Context<Self>) {
        let input: HtmlInputElement = self.search_node.cast().unwrap();
        ctx.props().search_callback.emit(input.value().into());
    }
}
