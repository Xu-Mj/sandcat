use fluent::{FluentBundle, FluentResource};
use gloo::utils::document;
use wasm_bindgen::{closure::Closure, JsCast};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlDivElement;
use yew::prelude::*;
use yewdux::Dispatch;

use abi::{
    model::{conversation::Conversation, ItemInfo, RightContentType},
    state::MuteState,
};
use i18n::{en_us, zh_cn, LanguageType};
use icons::PlusRectIcon;
use utils::tr;

pub struct SetWindow {
    list: Vec<Box<dyn ItemInfo>>,
    info: Option<Box<dyn ItemInfo>>,
    conv: Conversation,
    node: NodeRef,
    i18n: FluentBundle<FluentResource>,
    mute_dispatch: Dispatch<MuteState>,
    click_closure: Option<Closure<dyn FnMut(web_sys::MouseEvent)>>,
}

pub enum SetWindowMsg {
    QueryInfo(
        Option<Box<dyn ItemInfo>>,
        Vec<Box<dyn ItemInfo>>,
        Conversation,
    ),
    MuteClicked,
    None,
}

#[derive(Properties, PartialEq)]
pub struct SetWindowProps {
    pub conv_type: RightContentType,
    pub id: AttrValue,
    pub close: Callback<()>,
    pub plus_click: Callback<()>,
    pub lang: LanguageType,
}

/// query friend/group information by props id
/// layout:
/// top: list people's avatar
/// middle: some settings
/// bottom: delete
impl Component for SetWindow {
    type Message = SetWindowMsg;

    type Properties = SetWindowProps;

    fn create(ctx: &Context<Self>) -> Self {
        let id = ctx.props().id.clone();
        let conv_type = ctx.props().conv_type.clone();
        ctx.link().send_future(async move {
            // init interfaces
            let mut list: Vec<Box<dyn ItemInfo>> = vec![];
            let mut info: Option<Box<dyn ItemInfo>> = None;
            match conv_type {
                RightContentType::Friend => {
                    let friend = db::db_ins().friends.get(id.as_str()).await;
                    info = Some(Box::new(friend.clone()));
                    list.push(Box::new(friend));
                }
                RightContentType::Group => {
                    // query group information
                    let group = db::db_ins().groups.get(id.as_str()).await.unwrap().unwrap();
                    info = Some(Box::new(group.clone()));
                    // query members by group id
                    if let Ok(members) = db::db_ins()
                        .group_members
                        .get_list_by_group_id(id.as_str())
                        .await
                    {
                        for v in members.into_iter() {
                            list.push(Box::new(v));
                        }
                    }
                }
                _ => {}
            }
            // qeury conversation is mute
            let conv = db::db_ins().convs.get_by_frined_id(id.as_str()).await;
            SetWindowMsg::QueryInfo(info, list, conv)
        });
        let res = match ctx.props().lang {
            LanguageType::ZhCN => zh_cn::SET_WINDOW,
            LanguageType::EnUS => en_us::SET_WINDOW,
        };
        let i18n = utils::create_bundle(res);
        let mute_dispatch =
            Dispatch::global().subscribe(ctx.link().callback(|_| SetWindowMsg::None));
        Self {
            i18n,
            mute_dispatch,
            list: Vec::new(),
            info: None,
            conv: Conversation::default(),
            node: NodeRef::default(),
            click_closure: None,
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, first_render: bool) {
        if first_render {
            if let Some(node) = self.node.cast::<HtmlDivElement>() {
                node.focus().unwrap();
                let onclose = _ctx.props().close.clone();
                // register click event to document
                let func = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                    if let Some(target) = event.target() {
                        let target_node = target.dyn_into::<web_sys::Node>().unwrap();
                        let node = document().get_element_by_id("setting-window").unwrap();
                        if !node.contains(Some(&target_node)) {
                            onclose.emit(());
                            // 卸载这个onclick 事件
                            document().set_onclick(None);
                        }
                    }
                }) as Box<dyn FnMut(web_sys::MouseEvent)>);
                document().set_onclick(Some(func.as_ref().unchecked_ref()));
                self.click_closure = Some(func);
            }
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            SetWindowMsg::QueryInfo(info, list, mute) => {
                self.list = list;
                self.info = info;
                self.conv = mute;
                true
            }
            SetWindowMsg::MuteClicked => {
                self.conv.mute = !self.conv.mute;
                let conv = self.conv.clone();
                // update conversation
                spawn_local(async move {
                    db::db_ins().convs.mute(&conv).await.unwrap();
                });
                // todo send mute message to conversation component
                if let Some(info) = self.info.as_ref() {
                    self.mute_dispatch.reduce_mut(|s| s.conv_id = info.id());
                }
                true
            }
            SetWindowMsg::None => false,
        }
    }
    fn view(&self, ctx: &Context<Self>) -> Html {
        let avatars = self
            .list
            .iter()
            .map(|item| {
                html! {
                    <div class="avatar-name">
                        <img src={item.avatar()} />
                        <span>{item.name()}</span>
                    </div>
                }
            })
            .collect::<Html>();
        let add_click = ctx.props().plus_click.reform(|_| ());
        let add_friend = html! {
            <div class="avatar-name pointer" onclick={add_click}>
                <PlusRectIcon/>
                <span>{tr!(self.i18n, "add")}</span>
            </div>
        };
        let mute_click = ctx.link().callback(|_| SetWindowMsg::MuteClicked);
        let mut switch = classes!("switch", "pointer");
        let mut slider = classes!("slider");
        if self.conv.mute {
            switch.push("background-change");
            slider.push("right");
        } else {
            slider.push("left");
        }
        let mut info = html!();
        if ctx.props().conv_type == RightContentType::Group {
            if let Some(v) = self.info.as_ref() {
                info = html! {
                    <div class="info">
                        <div class="group-name">
                            <div>
                                {tr!(self.i18n, "group_name")}
                            </div>
                            <input type="text" value={v.name()} />
                        </div>
                        <div class="group-announcement">
                            <div>
                                {tr!(self.i18n, "group_announcement")}
                            </div>
                            <input type="text" value={v.remark()} />
                        </div>
                        <div class="group-desc">
                            <div>
                                {tr!(self.i18n, "group_desc")}
                            </div>
                            <input type="text" value={v.signature()} />
                        </div>
                    </div>
                }
            }
        }
        let setting = html! {
            <div class="setting-item">
            {tr!(self.i18n, "mute")}
            <span class={switch} onclick={mute_click}>
                <span class={slider}></span>
            </span>
            </div>
        };
        // let onblur = ctx.props().close.reform(|_| ());
        html! {
            <div ref={self.node.clone()} tabindex="1" id="setting-window" /* {onblur} */ class="set-window box-shadow">
                <div class="people">
                    {avatars}
                    {add_friend}
                </div>
                {info}
                <div class="setting">
                    {setting}
                </div>
                <div class="bottom" >
                    {tr!(self.i18n, "delete")}
                </div>
            </div>
        }
    }
}
