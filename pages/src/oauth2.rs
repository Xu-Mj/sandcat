use web_sys::UrlSearchParams;
use yew::{function_component, html, AttrValue, Html, Properties};

#[derive(Debug, Properties, Clone, PartialEq)]
pub struct Props {
    pub code: AttrValue,
}

#[function_component]
pub fn OAuth2() -> Html {
    let location = gloo::utils::window().location();
    let search = location.search().unwrap_or_default();
    let search_params = UrlSearchParams::new_with_str(&search).unwrap();

    let code = search_params.get("code").unwrap_or_default();

    html! {
        <div>{"OAuth2"}{code}</div>
    }
}
