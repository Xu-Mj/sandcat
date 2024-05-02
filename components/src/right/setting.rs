use std::rc::Rc;

use fluent::{FluentBundle, FluentResource};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::{html, Component, Event, Properties};
use yewdux::Dispatch;

use abi::state::{FontSizeState, I18nState, ThemeState};
use i18n::{self, en_us, zh_cn, LanguageType};
use utils::tr;

pub struct Setting {
    i18n: FluentBundle<FluentResource>,
    lang: LanguageType,
    theme: Rc<ThemeState>,
    font_size: Rc<FontSizeState>,
}

pub enum SettingMsg {
    SwitchLanguage(Event),
    SwitchTheme(Event),
    SwitchFontSize(Event),
    None,
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct SettingProps {
    pub lang: LanguageType,
}

impl Component for Setting {
    type Message = SettingMsg;

    type Properties = SettingProps;

    fn create(ctx: &yew::prelude::Context<Self>) -> Self {
        let theme =
            Dispatch::<ThemeState>::global().subscribe(ctx.link().callback(|_| SettingMsg::None));
        // sub I18n
        let lang = ctx.props().lang;
        let content = match lang {
            LanguageType::ZhCN => i18n::zh_cn::SETTING,
            LanguageType::EnUS => i18n::en_us::SETTING,
        };
        let i18n = utils::create_bundle(content);

        let font_size_dis = Dispatch::<FontSizeState>::global()
            .subscribe_silent(ctx.link().callback(|_| SettingMsg::None));
        Self {
            i18n,
            lang,
            theme: theme.get(),
            font_size: font_size_dis.get(),
        }
    }

    fn update(&mut self, _ctx: &yew::prelude::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SettingMsg::SwitchLanguage(event) => {
                let input = event
                    .target()
                    .unwrap()
                    .dyn_into::<HtmlInputElement>()
                    .unwrap();
                let value = input.value();
                if value == "zh_cn" {
                    self.i18n = utils::create_bundle(zh_cn::SETTING);
                    // save language type with yewdux
                    Dispatch::<I18nState>::global().reduce_mut(|s| s.lang = LanguageType::ZhCN);
                    self.lang = LanguageType::ZhCN;
                } else if value == "en_us" {
                    self.i18n = utils::create_bundle(en_us::SETTING);
                    Dispatch::<I18nState>::global().reduce_mut(|s| s.lang = LanguageType::EnUS);
                    self.lang = LanguageType::EnUS;
                }
                true
            }
            SettingMsg::None => false,

            SettingMsg::SwitchTheme(event) => {
                let input = event
                    .target()
                    .unwrap()
                    .dyn_into::<HtmlInputElement>()
                    .unwrap();
                let value = input.value();
                let theme = ThemeState::from(value.as_str());
                self.theme = Rc::new(theme.clone());
                Dispatch::<ThemeState>::global().reduce_mut(|s| *s = theme);
                false
            }
            SettingMsg::SwitchFontSize(event) => {
                let input = event
                    .target()
                    .unwrap()
                    .dyn_into::<HtmlInputElement>()
                    .unwrap();
                let value = input.value();
                let font_size = FontSizeState::from(value.as_str());
                self.font_size = Rc::new(font_size.clone());
                Dispatch::<FontSizeState>::global().reduce_mut(|s| *s = font_size);
                false
            }
        }
    }

    fn view(&self, ctx: &yew::prelude::Context<Self>) -> yew::prelude::Html {
        let onchange = ctx.link().callback(SettingMsg::SwitchLanguage);
        let on_font_size_change = ctx.link().callback(SettingMsg::SwitchFontSize);
        let on_theme_change = ctx.link().callback(SettingMsg::SwitchTheme);

        html! {
            <div class="setting">
                <div class="rect">
                   <h2> { tr!(self.i18n, "setting") }</h2>

                    <div class="font-size">
                        <b>{tr!(self.i18n, "font_size")}</b>
                        <label for="small">
                            <input type="radio"
                                name="font_size"
                                id="en_us"
                                value="small"
                                onchange={on_font_size_change.clone()}
                                checked={*self.font_size==FontSizeState::Small}/>
                                {format!("\t{}", tr!(self.i18n, "small"))}
                        </label>
                        <label for="medium">
                            <input type="radio"
                            name="font_size"
                            id="medium"
                            value="medium"
                            onchange={on_font_size_change.clone()}
                            checked={*self.font_size==FontSizeState::Medium}/>
                            {format!("\t{}", tr!(self.i18n, "medium"))}
                        </label>
                        <label for="large">
                            <input type="radio"
                            name="font_size"
                            id="large"
                            value="large"
                            onchange={on_font_size_change.clone()}
                            checked={*self.font_size==FontSizeState::Large}/>
                            {format!("\t{}", tr!(self.i18n, "large"))}
                        </label>
                        <label for="larger">
                            <input type="radio"
                            name="font_size"
                            id="larger"
                            value="larger"
                            onchange={on_font_size_change}
                            checked={*self.font_size==FontSizeState::Larger}/>
                            {format!("\t{}", tr!(self.i18n, "larger"))}
                        </label>
                    </div>

                    <div class="language">
                        <b>{tr!(self.i18n, "language")}</b>
                        <label for="en_us">
                            <input type="radio" name="language" id="en_us" value="en_us" onchange={onchange.clone()} checked={self.lang==LanguageType::EnUS}/>{"\tENG"}
                        </label>
                        <label for="zh_cn">
                            <input type="radio" name="language" id="zh_cn" value="zh_cn" {onchange} checked={self.lang==LanguageType::ZhCN}/>{"\t中文"}
                        </label>
                    </div>

                    <div class="setting-theme">
                        <b>{tr!(self.i18n, "theme")}</b>
                        <label for="light">
                            <input type="radio" name="theme" id="light" value="light" onchange={on_theme_change.clone()} checked={*self.theme==ThemeState::Light}/>{format!("\t{}", tr!(self.i18n, "light"))}
                        </label>
                        <label for="dark">
                            <input type="radio" name="theme" id="dark" value="dark" onchange={on_theme_change} checked={*self.theme==ThemeState::Dark}/>{format!("\t{}", tr!(self.i18n, "dark"))}
                        </label>
                    </div>
                </div>
            </div>
        }
    }
}
