use chrono::TimeZone;
use gloo::{timers::callback::Timeout, utils::window};
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlElement;
use yew::prelude::*;
use yewdux::Dispatch;

use sandcat_sdk::{
    db,
    model::{CommonProps, ComponentType, CurrentItem, RightContentType},
    state::{ComponentTypeState, ConvState, FriendListState, MobileState, ShowRight, UnreadState},
};

pub struct ListItem {
    node_ref: NodeRef,
    conv_state: Rc<ConvState>,
    conv_dispatch: Dispatch<ConvState>,
    friend_state: Rc<FriendListState>,
    friend_dispatch: Dispatch<FriendListState>,
    unread_count: usize,
    is_mobile: bool,
    touch_start: i64,
    long_press_timer: Option<Timeout>,
}

#[derive(Properties, PartialEq)]
pub struct ListItemProps {
    pub props: CommonProps,
    pub component_type: ComponentType,
    pub unread_count: usize,
    pub conv_type: RightContentType,
    pub oncontextmenu: Callback<((i32, i32), AttrValue, bool)>,
    pub mute: bool,
}

pub enum ListItemMsg {
    ConvStateChanged(Rc<ConvState>),
    FriendStateChanged(Rc<FriendListState>),
    GoToSetting,
    CleanUnreadCount,
    FriendItemClicked,
    OnContextMenu(MouseEvent),
    TouchStart(TouchEvent),
    TouchEnd(TouchEvent),
    CleanTimer,
}

impl Component for ListItem {
    type Message = ListItemMsg;

    type Properties = ListItemProps;

    fn create(ctx: &Context<Self>) -> Self {
        let conv_dispatch =
            Dispatch::global().subscribe(ctx.link().callback(ListItemMsg::ConvStateChanged));
        let friend_dispatch =
            Dispatch::global().subscribe(ctx.link().callback(ListItemMsg::FriendStateChanged));

        let unread_count = ctx.props().unread_count;

        Self {
            node_ref: NodeRef::default(),
            friend_state: friend_dispatch.get(),
            friend_dispatch,
            unread_count,
            conv_state: conv_dispatch.get(),
            conv_dispatch,
            is_mobile: MobileState::is_mobile(),
            touch_start: 0,
            long_press_timer: None,
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
                // set message is_read to true
                let friend_id = ctx.props().props.id.clone();
                let unread_count = self.unread_count;
                log::debug!("clean unread count");
                spawn_local(async move {
                    if db::db_ins()
                        .messages
                        .update_read_status(&friend_id)
                        .await
                        .is_ok()
                    {
                        Dispatch::<UnreadState>::global().reduce_mut(|s| {
                            s.msg_count = s.msg_count.saturating_sub(unread_count);
                        });
                    }
                });
                self.unread_count = 0;
                // show right if mobile
                if self.is_mobile {
                    Dispatch::<ShowRight>::global().set(ShowRight::Show);
                    // return false;
                }
                // do not update if current item is the same
                if self.conv_state.conv.item_id == ctx.props().props.id {
                    return false;
                }
                self.conv_dispatch.reduce_mut(|s| {
                    s.conv = CurrentItem {
                        item_id: ctx.props().props.id.clone(),
                        content_type: ctx.props().conv_type.clone(),
                    };
                });

                Dispatch::<ComponentTypeState>::global()
                    .set(ComponentTypeState::from(ctx.props().component_type));
                true
            }
            ListItemMsg::FriendStateChanged(state) => {
                self.friend_state = state;
                true
            }
            ListItemMsg::FriendItemClicked => {
                if self.is_mobile {
                    Dispatch::<ShowRight>::global().set(ShowRight::Show);
                    // return false;
                }
                if self.friend_state.friend.item_id == ctx.props().props.id {
                    return false;
                }

                self.friend_dispatch.reduce_mut(|s| {
                    let friend = CurrentItem {
                        item_id: ctx.props().props.id.clone(),
                        content_type: ctx.props().conv_type.clone(),
                    };
                    // current_item::save_friend(&friend).unwrap();
                    s.friend = friend;
                });
                false
            }
            ListItemMsg::OnContextMenu(event) => {
                event.prevent_default();
                ctx.props().oncontextmenu.emit((
                    (event.client_x(), event.client_y()),
                    ctx.props().props.id.clone(),
                    ctx.props().mute,
                ));
                false
            }
            ListItemMsg::TouchStart(event) => {
                self.touch_start = chrono::Utc::now().timestamp_millis();
                let oncontextmenu = ctx.props().oncontextmenu.clone();
                let id = ctx.props().props.id.clone();
                let mute = ctx.props().mute;
                let ctx = ctx.link().clone();
                self.long_press_timer = Some(Timeout::new(500, move || {
                    if let Some(event) = event.changed_touches().get(0) {
                        window().navigator().vibrate_with_duration(100);
                        oncontextmenu.emit((
                            (event.client_x(), event.client_y()),
                            id.clone(),
                            mute,
                        ));
                    }
                    ctx.send_message(ListItemMsg::CleanTimer);
                }));

                // add hover class
                self.node_ref
                    .cast::<HtmlElement>()
                    .map(|div| div.class_list().add_1("hover"));
                false
            }
            ListItemMsg::TouchEnd(event) => {
                event.prevent_default();
                if self.touch_start != 0
                    && chrono::Utc::now().timestamp_millis() - self.touch_start > 500
                {
                    if let Some(event) = event.changed_touches().get(0) {
                        ctx.props().oncontextmenu.emit((
                            (event.client_x(), event.client_y()),
                            ctx.props().props.id.clone(),
                            ctx.props().mute,
                        ));
                    }
                } else {
                    ctx.link().send_message(ListItemMsg::CleanUnreadCount);
                }
                self.long_press_timer = None;
                self.touch_start = 0;
                false
            }
            ListItemMsg::CleanTimer => {
                self.long_press_timer = None;
                self.touch_start = 0;
                false
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.unread_count = ctx.props().unread_count;
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // handle mobile long press event
        let (touch_start, touch_end) = if self.is_mobile {
            (
                Some(ctx.link().callback(ListItemMsg::TouchStart)),
                Some(ctx.link().callback(ListItemMsg::TouchEnd)),
            )
        } else {
            (None, None)
        };
        // 根据参数渲染组件
        let props = &ctx.props().props;
        let onclick;
        let mut unread_count = html! {};
        let mut classes = Classes::from("item");
        match ctx.props().component_type {
            ComponentType::Contacts => {
                onclick = ctx.link().callback(move |_| ListItemMsg::FriendItemClicked);
                if !self.is_mobile {
                    if self.friend_state.friend.item_id == props.id {
                        classes.push("selected");
                    } else {
                        classes.push("hover")
                    }
                }
            }
            ComponentType::Messages => {
                onclick = ctx.link().callback(move |_| ListItemMsg::CleanUnreadCount);
                if !self.is_mobile {
                    if self.conv_state.conv.item_id == props.id {
                        classes.push("selected");
                    } else {
                        classes.push("hover")
                    }
                }

                if self.unread_count > 0 {
                    let mut unread_str = self.unread_count.to_string();
                    if self.unread_count >= 100 {
                        unread_str = "99+".to_string();
                    }
                    if ctx.props().mute {
                        unread_str = format!("[{}条]", unread_str);
                        unread_count = html! {
                        <span class="unread-count-mute">{unread_str}</span>
                        }
                    } else {
                        unread_count = html! {
                            <span class="unread-count">{unread_str}</span>
                        }
                    }
                };
            }
            ComponentType::Setting => {
                onclick = ctx.link().callback(move |_| ListItemMsg::GoToSetting)
            }
            ComponentType::Default => {
                onclick = ctx.link().callback(move |_| ListItemMsg::GoToSetting)
            }
        };

        // 判断距离现在多久
        let mut time_str = String::new();
        if props.time > 0 {
            let now = chrono::Utc::now().timestamp_millis();
            let step = now - props.time;
            let time_flag = if step < 60 * 1000 * 24 {
                "%H:%M"
            } else if (60 * 1000 * 24..60 * 1000 * 48).contains(&step) {
                "昨天 %H:%M"
            } else {
                "%a %b %e %H:%M"
            };
            // a: week b: month e: day T: time Y: year
            let utc_date = chrono::Utc
                .timestamp_millis_opt(props.time)
                .unwrap()
                .with_timezone(&chrono::Local);
            time_str = utc_date.format(time_flag).to_string();
        }
        let mut name = props.name.clone();
        if !props.remark.is_empty() {
            name = props.remark.clone();
        }
        let mut right = html!();
        match ctx.props().component_type {
            ComponentType::Contacts => {
                right = html! {
                    <div class="name-time">
                        <span>{name}</span>
                    </div>
                }
            }
            ComponentType::Messages => {
                right = html! {
                    <>
                        <div class="name-time">
                            <span>{&props.name}</span>
                            <span class="time">{time_str}</span>
                        </div>
                        <div class="remark">{&props.remark}</div>
                    </>
                }
            }
            ComponentType::Setting => {}
            ComponentType::Default => {}
        }
        let oncontextmenu = ctx.link().callback(ListItemMsg::OnContextMenu);
        html! {
        <div ref={self.node_ref.clone()}
            class={classes}
            {onclick}
            title={&props.name}
            {oncontextmenu}
            ontouchstart={touch_start}
            ontouchend={touch_end}>
            {self.get_avatar(ctx)}
            <div class="item-info">
                {unread_count}
                {right}
            </div>
        </div>
        }
    }
}

impl ListItem {
    fn get_avatar(&self, ctx: &Context<Self>) -> Html {
        // deal with group avatars
        let avatar_str = &ctx.props().props.avatar;

        let mut avatar_style = "--avatar-column: 1";
        // trim spliter
        let avatar_str = avatar_str.trim_matches(',');
        // get len
        let len = avatar_str.matches(',').count() + 1;
        let iter = avatar_str.split(',');
        if len > 1 && len < 5 {
            avatar_style = "--avatar-column: 2"
        } else if len >= 5 {
            avatar_style = "--avatar-column: 3"
        }

        let avatar = iter
            .map(|v| {
                html! {
                    <img class="avatar" src={utils::get_avatar_url(v)} />
                }
            })
            .collect::<Html>();
        html! {
            <div class="item-avatar" style={avatar_style}>
                {avatar}
            </div>
        }
    }
}
