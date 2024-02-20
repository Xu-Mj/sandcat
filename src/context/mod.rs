use std::rc::Rc;

use yew::prelude::*;

use crate::pages::ComponentType;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AppState {
    
    pub component_type: ComponentType,
    // pub switch_com_event: Callback<ComponentType>,
}

impl Reducible for AppState {
    type Action = ComponentType;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        AppState { component_type: action }.into()
    }
}

pub type AppContext = UseReducerHandle<AppState>;

#[derive(Properties, Debug, PartialEq)]
pub struct AppContextProviderProps {
    #[prop_or_default]
    pub children: Html,
}

#[function_component]
pub fn AppContextProvider(props: &AppContextProviderProps) -> Html {
    let msg = use_reducer(|| AppState {
       component_type: ComponentType::Contacts
    });

    html! {
        <ContextProvider<AppContext> context={msg}>
            {props.children.clone()}
        </ContextProvider<AppContext>>
    }
}