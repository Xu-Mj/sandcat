use std::rc::Rc;

use fluent::{FluentBundle, FluentResource};
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::{classes, html, Component, Event, InputEvent, Properties};
use yewdux::Dispatch;

use i18n::{self, en_us, zh_cn, LanguageType};
use sandcat_sdk::state::{
    FontSizeState, I18nState, MobileState, Notify, ThemeState, TransparentState,
};
use utils::tr;

use crate::constant::{
    DARK, FONT_SIZE, LANGUAGE, LARGE, LARGER, LIGHT, MEDUIM, SETTING, SMALL, THEME, TRANSPARENT,
};

pub struct Setting {
    i18n: FluentBundle<FluentResource>,
    lang: LanguageType,
    theme: Rc<ThemeState>,
    font_size: Rc<FontSizeState>,
    transparent: f32,
}

pub enum SettingMsg {
    SwitchLanguage(Event),
    SwitchTheme(Event),
    SwitchFontSize(Event),
    TransparentChange(InputEvent),
}

#[derive(Debug, Clone, PartialEq, Properties)]
pub struct SettingProps {
    pub lang: LanguageType,
}

impl Component for Setting {
    type Message = SettingMsg;

    type Properties = SettingProps;

    fn create(ctx: &yew::prelude::Context<Self>) -> Self {
        let theme = ThemeState::get();
        // sub I18n
        let lang = ctx.props().lang;
        let content = match lang {
            LanguageType::ZhCN => i18n::zh_cn::SETTING,
            LanguageType::EnUS => i18n::en_us::SETTING,
        };
        let i18n = utils::create_bundle(content);

        let font_size = FontSizeState::get();
        let transparent = TransparentState::get();
        Self {
            i18n,
            lang,
            theme,
            font_size,
            transparent,
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
            SettingMsg::SwitchTheme(event) => {
                let input = event
                    .target()
                    .unwrap()
                    .dyn_into::<HtmlInputElement>()
                    .unwrap();
                let value = input.value();
                let theme = ThemeState::from(value.as_str());
                // use yewdux to save theme to local storage
                self.theme = Rc::new(theme.clone());
                theme.notify();
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
                font_size.notify();
                false
            }
            SettingMsg::TransparentChange(event) => {
                let input = event
                    .target()
                    .unwrap()
                    .dyn_into::<HtmlInputElement>()
                    .unwrap();
                let value = input.value();
                self.transparent = value.parse::<f32>().unwrap();
                TransparentState::set(self.transparent);
                true
            }
        }
    }

    fn view(&self, ctx: &yew::prelude::Context<Self>) -> yew::prelude::Html {
        let onchange = ctx.link().callback(SettingMsg::SwitchLanguage);
        let on_font_size_change = ctx.link().callback(SettingMsg::SwitchFontSize);
        let on_theme_change = ctx.link().callback(SettingMsg::SwitchTheme);
        let on_transparent_change = ctx.link().callback(SettingMsg::TransparentChange);

        let mut class = classes!("rect");
        let mut font_class = classes!("font-size");
        match *MobileState::get() {
            MobileState::Desktop => {
                class.push("rect-size");
                font_class.push("font-size-desktop");
            }
            MobileState::Mobile => {
                class.push("rect-size-mobile");
                font_class.push("font-size-mobile");
            }
        }
        html! {
            <div class="setting">
                <div {class}>
                   <h1> { tr!(self.i18n, SETTING) }</h1>

                    <div class={font_class}>
                        <b>{tr!(self.i18n, FONT_SIZE)}</b>
                        <div>
                        <label for="small">
                            <input type="radio"
                                name="font_size"
                                id="small"
                                value="small"
                                onchange={on_font_size_change.clone()}
                                checked={*self.font_size==FontSizeState::Small}/>
                                {format!("\t{}", tr!(self.i18n, SMALL))}
                        </label>
                        <label for="medium">
                            <input type="radio"
                                name="font_size"
                                id="medium"
                                value="medium"
                                onchange={on_font_size_change.clone()}
                                checked={*self.font_size==FontSizeState::Medium}/>
                            {format!("\t{}", tr!(self.i18n, MEDUIM))}
                        </label>
                        <label for="large">
                            <input type="radio"
                                name="font_size"
                                id="large"
                                value="large"
                                onchange={on_font_size_change.clone()}
                                checked={*self.font_size==FontSizeState::Large}/>
                            {format!("\t{}", tr!(self.i18n, LARGE))}
                        </label>
                        <label for="larger">
                            <input type="radio"
                                name="font_size"
                                id="larger"
                                value="larger"
                                onchange={on_font_size_change}
                                checked={*self.font_size==FontSizeState::Larger}/>
                            {format!("\t{}", tr!(self.i18n, LARGER))}
                        </label>
                        </div>
                    </div>

                    <div class="language">
                        <b>{tr!(self.i18n, LANGUAGE)}</b>
                        <label for="en_us">
                            <input type="radio"
                                name="language"
                                id="en_us"
                                value="en_us"
                                onchange={onchange.clone()}
                                checked={self.lang==LanguageType::EnUS}/>
                            {"\tENG"}
                        </label>
                        <label for="zh_cn">
                            <input type="radio"
                                name="language"
                                id="zh_cn"
                                value="zh_cn"
                                {onchange}
                                checked={self.lang==LanguageType::ZhCN}/>
                            {"\t中文"}
                        </label>
                    </div>

                    <div class="setting-theme">
                        <b>{tr!(self.i18n, THEME)}</b>
                        <label for="light">
                            <input type="radio"
                                name="theme"
                                id="light"
                                value="light"
                                onchange={on_theme_change.clone()}
                                checked={*self.theme==ThemeState::Light}/>
                            {format!("\t{}", tr!(self.i18n, LIGHT))}
                        </label>
                        <label for="dark">
                            <input type="radio"
                                name="theme"
                                id="dark"
                                value="dark"
                                onchange={on_theme_change}
                                checked={*self.theme==ThemeState::Dark}/>
                            {format!("\t{}", tr!(self.i18n, DARK))}
                        </label>
                    </div>

                    <div class="setting-transparent">
                        <b>{tr!(self.i18n, TRANSPARENT)}</b>
                        <label for="transparent">
                            <input
                                type="range"
                                name="transparent"
                                id="transparent"
                                min="0"
                                max="1"
                                step="0.01"
                                placeholder="transparent"
                                value={self.transparent.to_string()}
                                oninput={on_transparent_change} />
                        </label>
                            <span>{self.transparent}</span>
                    </div>
                </div>
            </div>
        }
    }
}
