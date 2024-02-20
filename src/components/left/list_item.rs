#![allow(dead_code)]

use std::rc::Rc;

use chrono::TimeZone;
use yew::prelude::*;

use crate::{
    db::{current_item, RightContentType},
    pages::{CommonProps, ComponentType, ConvState, CurrentItem, FriendListState},
};

pub struct ListItem {
    conv_state: Rc<ConvState>,
    _conv_listener: ContextHandle<Rc<ConvState>>,
    friend_state: Rc<FriendListState>,
    _friend_listener: ContextHandle<Rc<FriendListState>>,
    unread_count: usize,
    node: NodeRef,
}

#[derive(Properties, PartialEq)]
pub struct ListItemProps {
    pub props: CommonProps,
    pub component_type: ComponentType,
    pub unread_count: usize,
    pub conv_type: RightContentType,
}

pub enum ListItemMsg {
    ConvStateChanged(Rc<ConvState>),
    FriendStateChanged(Rc<FriendListState>),
    GoToSetting,
    CleanUnreadCount,
    FriendItemClicked,
}

impl Component for ListItem {
    type Message = ListItemMsg;

    type Properties = ListItemProps;

    fn create(ctx: &Context<Self>) -> Self {
        log::debug!(
            "list item conv type:{:?}, friend id:{}",
            ctx.props().conv_type.clone(),
            ctx.props().props.id.clone()
        );
        let (conv_state, _conv_listener) = ctx
            .link()
            .context(ctx.link().callback(ListItemMsg::ConvStateChanged))
            .expect("need state in item");
        let (friend_state, _friend_listener) = ctx
            .link()
            .context(ctx.link().callback(ListItemMsg::FriendStateChanged))
            .expect("need state in item");
        let unread_count = ctx.props().unread_count;

        Self {
            conv_state,
            _conv_listener,
            friend_state,
            _friend_listener,
            unread_count,
            node: Default::default(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ListItemMsg::ConvStateChanged(state) => {
                self.conv_state = state;
                true
            }
            ListItemMsg::GoToSetting => false,
            ListItemMsg::CleanUnreadCount => {
                let conv = CurrentItem {
                    unread_count: self
                        .conv_state
                        .conv
                        .unread_count
                        .saturating_sub(self.unread_count),
                    item_id: ctx.props().props.id.clone(),
                    content_type: ctx.props().conv_type.clone(),
                };
                current_item::save_conv(&conv).unwrap();
                self.unread_count = 0;
                self.conv_state.state_change_event.emit(conv);
                true
            }
            ListItemMsg::FriendStateChanged(state) => {
                self.friend_state = state;
                true
            }
            ListItemMsg::FriendItemClicked => {
                if ctx.props().conv_type == RightContentType::UserInfo {
                    // 展示卡片

                    return false;
                }
                self.friend_state.state_change_event.emit(CurrentItem {
                    item_id: ctx.props().props.id.clone(),
                    content_type: ctx.props().conv_type.clone(),
                    unread_count: 0,
                });
                false
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.unread_count = ctx.props().unread_count;
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // 根据参数渲染组件
        let props = &ctx.props().props;
        let id = props.id.clone();
        let onclick;
        let mut unread_count = html! {};
        let mut classess = Classes::from("item");
        match ctx.props().component_type {
            ComponentType::Contacts => {
                onclick = ctx.link().callback(move |_| ListItemMsg::FriendItemClicked);
                if self.friend_state.friend.item_id == id {
                    classess.push("selected");
                } else {
                    classess.push("hover")
                }
            }
            ComponentType::Messages => {
                onclick = ctx.link().callback(move |_| ListItemMsg::CleanUnreadCount);
                if self.conv_state.conv.item_id == id {
                    classess.push("selected");
                } else {
                    classess.push("hover")
                }

                if self.unread_count > 0 {
                    let mut unread_str = self.unread_count.to_string();
                    if self.unread_count >= 100 {
                        unread_str = "99+".to_string();
                    }
                    unread_count = html! {
                        <span class="unread-count">{unread_str}</span>
                    }
                };
            }
            ComponentType::Setting => {
                onclick = ctx.link().callback(move |_| ListItemMsg::GoToSetting)
            }
        };

        // 判断距离现在多久
        let mut time_str = String::new();
        if props.time > 0 {
            let now = chrono::Local::now().timestamp_millis();
            let step = now - props.time;
            let time_flag = if step < 60 * 1000 * 24 {
                "%T"
            } else if step >= 60 * 1000 * 24 && step < 60 * 1000 * 48 {
                "昨天 %T"
            } else {
                "%a %b %e %T"
            };
            // a: week b: month e: day T: time Y: year
            time_str = chrono::Local
                .timestamp_millis_opt(props.time)
                .unwrap()
                .format(time_flag)
                .to_string();
        }
        let mut name = props.name.clone();
        if props.remark == AttrValue::default() {
            name = props.remark.clone();
        }
        let mut right = html!();
        match ctx.props().component_type {
            ComponentType::Contacts => {
                right = html! {
                    <div class="name-time">
                        <span>{props.name.clone()}</span>
                    </div>
                }
            }
            ComponentType::Messages => {
                right = html! {
                    <>
                        <div class="name-time">
                            <span>{props.name.clone()}</span>
                            <span class="time">{time_str}</span>
                        </div>
                        <div class="remark">{props.remark.clone()}</div>
                    </>
                }
            }
            ComponentType::Setting => {}
        }
        html! {
        <div class={classess} {onclick}>
            <div class="item-avatar">
                <img class="avatar" src={props.avatar.clone()} />
            </div>
            <div class="item-info">
                {unread_count}
                {right}
            </div>
        </div>
        }
    }
}
