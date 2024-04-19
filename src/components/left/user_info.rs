use std::rc::Rc;

use web_sys::HtmlDivElement;
use yew::prelude::*;

use crate::components::right::friend_card::FriendCard;
use crate::i18n::LanguageType;
use crate::model::user::UserWithMatchType;
use crate::pages::AppState;

pub struct UserInfoCom {
    node: NodeRef,
    app_state: Rc<AppState>,
    _listener: ContextHandle<Rc<AppState>>,
    show_friend_card: bool,
    x: i32,
    y: i32,
}

#[derive(Properties, PartialEq)]
pub struct UserInfoComProps {
    pub lang: LanguageType,
    pub info: UserWithMatchType,
}

pub enum UserInfoComMsg {
    FriendItemClicked,
    AppStateChange(Rc<AppState>),
    CloseFriendCard,
}

impl Component for UserInfoCom {
    type Message = UserInfoComMsg;

    type Properties = UserInfoComProps;

    fn create(ctx: &Context<Self>) -> Self {
        let (app_state, _listener) = ctx
            .link()
            .context(ctx.link().callback(UserInfoComMsg::AppStateChange))
            .expect("app state needed");
        Self {
            node: Default::default(),
            show_friend_card: false,
            x: 0,
            y: 0,
            app_state,
            _listener,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            UserInfoComMsg::FriendItemClicked => {
                // 获取自身坐标
                if let Some(div) = self.node.cast::<HtmlDivElement>() {
                    let rect = div.get_bounding_client_rect();
                    let x = rect.x() + rect.width();
                    log::debug!("user info component x: {}, y: {}", x, rect.y());
                    self.show_friend_card = true;
                    self.x = x as i32;
                    self.y = rect.y() as i32;
                    return true;
                }
                false
            }
            UserInfoComMsg::AppStateChange(state) => {
                self.app_state = state;
                true
            }
            UserInfoComMsg::CloseFriendCard => {
                self.show_friend_card = false;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // 根据参数渲染组件
        let props = &ctx.props().info;
        let mut friend_card = html!();
        if self.show_friend_card {
            friend_card = html!(
                <FriendCard
                    friend_info={ctx.props().info.clone()}
                    user_id={self.app_state.login_user.id.clone()}
                    lang={ctx.props().lang}
                    close={ctx.link().callback(|_| UserInfoComMsg::CloseFriendCard)}
                    is_self={false}
                    x={self.x}
                    y={self.y}
                />
            )
        }
        html! {
        <>
        <div class={"user-info"} ref={self.node.clone()} onclick={ctx.link().callback(|_|UserInfoComMsg::FriendItemClicked)}>
            <div class="item-avatar">
                <img class="avatar" src={props.avatar.clone()} />
            </div>
            <div class="item-info">
                <div class="name-time">
                    <span>{props.name.clone()}</span>
                </div>
            </div>
        </div>
            {friend_card}
        </>
        }
    }
}
