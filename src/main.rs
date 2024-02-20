mod api;
mod components;
mod config;
mod db;
mod error;
mod icons;
mod model;
mod pages;
mod utils;
mod web_rtc;
mod ws;

use crate::pages::register::Register;
use crate::pages::{home::Home, login::Login, Page};
use yew::prelude::*;
use yew_router::{BrowserRouter, Switch};

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Page> render={move |page|
                match page {
                    Page::Home{id} => html!{<Home id={id}/>},
                    Page::Login => html!{<Login/>},
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
