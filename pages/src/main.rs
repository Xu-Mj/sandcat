pub mod home;
pub mod login;
pub mod register;

use abi::model::page::Page;
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
