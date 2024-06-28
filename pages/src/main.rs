mod home;
mod login;
mod oauth2;
mod register;

use oauth2::OAuth2;
use sandcat_sdk::model::page::Page;
use yew::prelude::*;
use yew_router::{BrowserRouter, Switch};

use crate::register::Register;
use crate::{home::Home, login::Login};

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Page> render={move |page|
                match page {
                    Page::Home{id} => html!{<Home {id}/>},
                    Page::Login => html!{<Login/>},
                    Page::ThirdLoginCallback => html!{<OAuth2/>},
                    Page::Register => html!{<Register />},
                    Page::Redirect => html!{<Login />}}
            }/>
        </BrowserRouter>
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    yew::Renderer::<App>::new().render();
}
