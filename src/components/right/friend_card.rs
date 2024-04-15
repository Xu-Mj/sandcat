use fluent::{FluentBundle, FluentResource};
use gloo::utils::{document, window};
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlDivElement, HtmlInputElement};
use yew::prelude::*;

use crate::{
    api, db,
    i18n::{en_us, zh_cn, LanguageType},
    model::{
        friend::{FriendShipRequest, ReadStatus},
        user::{User, UserWithMatchType},
    },
    tr, utils,
};

#[derive(Default)]
pub struct FriendCard {
    friend: UserWithMatchType,
    node_ref: NodeRef,
    show_apply: bool,
    is_apply: bool,
    i18n: FluentBundle<FluentResource>,
    apply_node: NodeRef,
    remark_node: NodeRef,
}

pub enum FriendCardMsg {
    // Close,
    ShowApply,
    Apply,
    ApplyFriendResult(FriendShipRequestState),
    Destroy,
}

pub enum FriendShipRequestState {
    Pendding,
    Success,
    Fail,
}

#[derive(Properties, Clone, PartialEq)]
pub struct FriendCardProps {
    container: Element,
    friend_info: UserWithMatchType,
    user_info: Option<User>,
    lang: LanguageType,
    is_friend: bool,
    x: i32,
    y: i32,
}

impl FriendCard {
    fn mount(&self, ctx: &Context<Self>) {
        // 查询body节点
        let body = document()
            .get_element_by_id("app")
            .expect("body is not defined");
        // 将dialog渲染到容器中
        body.append_child(&ctx.props().container.clone()).unwrap();
    }

    pub fn container_with_position(x: i32, y: i32) -> Element {
        let container = document().create_element("div").unwrap();
        container.set_class_name("friend-card-container");
        // 设置容器绝对定位
        container
            .set_attribute("style", "position: fixed;")
            .unwrap();

        container.set_scroll_top(y);
        container.set_scroll_left(x);
        container
    }

    pub fn show(
        friend_info: UserWithMatchType,
        user_info: Option<User>,
        lang: LanguageType,
        is_friend: bool,
        x: i32,
        y: i32,
    ) {
        log::debug!("x: {}, y: {}", x, y);
        let container = FriendCard::container_with_position(x, y);
        let props = FriendCardProps {
            container: container.clone(),
            friend_info,
            user_info,
            lang,
            is_friend,
            x,
            y,
        };
        yew::Renderer::<FriendCard>::with_root_and_props(container, props).render();
    }
}

impl Component for FriendCard {
    type Message = FriendCardMsg;

    type Properties = FriendCardProps;

    fn create(ctx: &Context<Self>) -> Self {
        let friend = ctx.props().friend_info.clone();
        let res = match ctx.props().lang {
            LanguageType::ZhCN => zh_cn::FRIEND_CARD,
            LanguageType::EnUS => en_us::FRIEND_CARD,
        };
        let i18n = utils::create_bundle(res);
        let self_ = Self {
            i18n,
            friend,
            ..Default::default()
        };
        self_.mount(ctx);
        self_
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            FriendCardMsg::ShowApply => {
                self.show_apply = true;
                true
            }
            FriendCardMsg::Apply => {
                let friend_id = ctx.props().friend_info.id.clone();
                let source = ctx.props().friend_info.match_type.clone();
                let user_id = ctx.props().user_info.as_ref().unwrap().id.clone();
                let apply_node: HtmlInputElement = self.apply_node.cast().unwrap();
                let apply_msg = if apply_node.value().is_empty() {
                    None
                } else {
                    Some(AttrValue::from(apply_node.value()))
                };
                let remark: HtmlInputElement = self.remark_node.cast().unwrap();
                let remark = if remark.value().is_empty() {
                    None
                } else {
                    Some(AttrValue::from(remark.value()))
                };
                // 发送好友申请
                let new_friend = FriendShipRequest {
                    user_id,
                    friend_id,
                    apply_msg,
                    source,
                    remark,
                };

                log::debug!("发送好友申请:{:?}", &new_friend);
                ctx.link().send_message(FriendCardMsg::ApplyFriendResult(
                    FriendShipRequestState::Pendding,
                ));
                ctx.link().send_future(async move {
                    match api::friends().apply_friend(new_friend).await {
                        Err(err) => {
                            log::error!("发送好友申请错误: {:?}", err);
                            FriendCardMsg::ApplyFriendResult(FriendShipRequestState::Fail)
                        }
                        Ok(mut friendship) => {
                            friendship.is_self = true;
                            friendship.read = ReadStatus::True;
                            // 数据入库
                            db::friendships().await.put_friendship(&friendship).await;
                            FriendCardMsg::ApplyFriendResult(FriendShipRequestState::Success)
                        }
                    }
                });
                false
            }
            FriendCardMsg::ApplyFriendResult(_state) => {
                self.is_apply = true;
                self.show_apply = false;
                // 发送通知，右侧渲染申请列表
                true
            }
            FriendCardMsg::Destroy => {
                let div = self.node_ref.cast::<HtmlDivElement>().unwrap();
                div.parent_element().unwrap().remove();
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let apply = if ctx.props().is_friend {
            html!()
        } else if self.show_apply {
            let onclick = ctx.link().callback(|_| FriendCardMsg::Apply);
            let apply_msg = if self.is_apply {
                tr!(self.i18n, "applied")
            } else {
                tr!(self.i18n, "apply")
            };
            html! {
                <div class="apply-detail">
                    <div class="apply-msg">
                        <label>{tr!(self.i18n, "apply_msg")}</label>
                        <input class="apply-input" ref={self.apply_node.clone()} type="text"/>
                    </div>
                    <div class="apply-remark">
                        <label>{tr!(self.i18n, "remark")}</label>
                        <input class="apply-input" ref={self.remark_node.clone()} type="text"/>
                    </div>
                    <div class="apply-friend" >
                        <button {onclick} disabled={self.is_apply}>{apply_msg}</button>
                        <button /* onclick={cancel} */>{tr!(self.i18n, "cancel")}</button>
                    </div>
                </div>
            }
        } else {
            let onclick = ctx.link().callback(|_| FriendCardMsg::ShowApply);
            let cancel = ctx.link().callback(|_| FriendCardMsg::Destroy);
            html! {
                <div class="apply-friend" >
                    <button {onclick}>{tr!(self.i18n, "apply")}</button>
                    <button onclick={cancel}>{tr!(self.i18n, "cancel")}</button>
                </div>
            }
        };
        html! {
            <div
                class="friend-card box-shadow"
                tabindex="1"
                ref={self.node_ref.clone()}
                /* onblur={ctx.link().callback(|_| FriendCardMsg::Destroy)} */
                >
                <div class="friend-card-header">
                    <img src={&self.friend.avatar} class="friend-card-avatar"/>
                    <div class="friend-card-info">
                        // <span><b>{&self.friend.remark}</b></span>
                        <span>{tr!(self.i18n, "nickname")}{&self.friend.name}</span>
                        <span>{tr!(self.i18n, "account")}{&self.friend.account}</span>
                        <span>{tr!(self.i18n, "region")}{&self.friend.region.clone().unwrap_or_default()} </span>
                    </div>
                </div>
                // <div class="user-card-body">
                // dialog 已经脱离了整个文档了，无法使用context中的状态了
                // <Action id={&self.friend.friend_id}/>
                <div class="friend-card-body">
                    {apply}
                </div>
            </div>
            // </dialog>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let node = self.node_ref.cast::<HtmlDivElement>().unwrap();
            node.focus().unwrap();
            let node = node
                .parent_node()
                .unwrap()
                .dyn_into::<HtmlDivElement>()
                .unwrap();

            // 计算下边框
            let height = window().inner_height().unwrap().as_f64().unwrap() as i32;
            let width = window().inner_width().unwrap().as_f64().unwrap() as i32;
            let mut x = ctx.props().x;
            let mut y = ctx.props().y;
            let margin = 0;
            let offset = 0;
            if node.client_height() > height - y {
                y = height - node.client_height() - margin;
            }
            if node.client_width() > width - x {
                x = width - node.client_width() - margin - offset;
            } else {
                x = x + margin + offset;
            }

            node.style()
                .set_property("top", format!("{}px", y).as_str())
                .unwrap();
            node.style()
                .set_property("left", format!("{}px", x).as_str())
                .unwrap();
            // node.set_tab_index(1);
            // node.focus().unwrap();
        }
    }
}
