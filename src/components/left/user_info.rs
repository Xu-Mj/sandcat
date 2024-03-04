use crate::components::right::friend_card::FriendCard;
use crate::model::user::User;
use crate::pages::AppState;
use std::rc::Rc;
use web_sys::HtmlDivElement;
use yew::prelude::*;

pub struct UserInfoCom {
    node: NodeRef,
    app_state: Rc<AppState>,
    _listener: ContextHandle<Rc<AppState>>,
}

#[derive(Properties, PartialEq)]
pub struct UserInfoComProps {
    pub info: User,
}

pub enum UserInfoComMsg {
    FriendItemClicked,
    AppStateChange(Rc<AppState>),
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
            app_state,
            _listener,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            UserInfoComMsg::FriendItemClicked => {
                // 获取自身坐标
                if let Some(div) = self.node.cast::<HtmlDivElement>() {
                    let rect = div.get_bounding_client_rect();
                    let x = rect.x() + rect.width();
                    log::debug!("user info component x: {}, y: {}", x, rect.y());
                    FriendCard::show(
                        ctx.props().info.clone(),
                        Some(self.app_state.login_user.clone()),
                        false,
                        x as i32,
                        rect.y() as i32,
                    );
                }
                false
            }
            UserInfoComMsg::AppStateChange(state) => {
                self.app_state = state;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        // 根据参数渲染组件
        let props = &ctx.props().info;
        html! {
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
        }
    }
}
