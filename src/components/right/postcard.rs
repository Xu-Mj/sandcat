use yew::prelude::*;

use crate::components::right::set_drawer::SetDrawer;
use crate::db::group::GroupRepo;
use crate::model::{ItemInfo, RightContentType};
use crate::{components::action::Action, db::friend::FriendRepo};

#[derive(Properties, Clone, PartialEq)]
pub struct PostCardProps {
    pub id: AttrValue,
    pub user_id: AttrValue,
    pub conv_type: RightContentType,
}

pub enum PostCardMsg {
    QueryInformation(QueryState<Option<Box<dyn ItemInfo>>>),
    // QueryGroup(QueryState<Group>),
    ShowSetDrawer,
}

pub enum QueryState<T> {
    Querying,
    Success(T),
    Failed,
}

pub struct PostCard {
    info: Option<Box<dyn ItemInfo>>,
    is_group_owner: bool,
    // user_info: User,
    show_set_drawer: bool,
}

impl PostCard {
    fn query(&self, ctx: &Context<Self>) {
        ctx.link()
            .send_message(PostCardMsg::QueryInformation(QueryState::Querying));
        let id = ctx.props().id.clone();
        log::debug!("friend_id :{:?}", &id);
        if !id.is_empty() {
            match ctx.props().conv_type {
                RightContentType::Friend => {
                    ctx.link().send_future(async move {
                        let user_info = FriendRepo::new().await.get(id).await;
                        log::debug!("user info :{:?}", user_info);
                        PostCardMsg::QueryInformation(QueryState::Success(Some(Box::new(
                            user_info,
                        ))))
                    });
                }
                RightContentType::Group => ctx.link().send_future(async move {
                    match GroupRepo::new().await.get(id).await {
                        Ok(Some(group)) => PostCardMsg::QueryInformation(QueryState::Success(
                            Some(Box::new(group)),
                        )),
                        _ => PostCardMsg::QueryInformation(QueryState::Failed),
                    }
                }),
                _ => {}
            }
        }
    }

    fn reset(&mut self) {
        self.info = None;
    }
}

impl Component for PostCard {
    type Message = PostCardMsg;
    type Properties = PostCardProps;

    fn create(ctx: &Context<Self>) -> Self {
        log::debug!("postcard conv type:{:?}", ctx.props().conv_type.clone());
        let self_ = PostCard {
            info: None,
            show_set_drawer: false,
            is_group_owner: false,
        };
        self_.query(ctx);
        self_
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.reset();
        self.query(ctx);
        true
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PostCardMsg::QueryInformation(state) => match state {
                QueryState::Querying => true,
                QueryState::Success(info) => {
                    if info.is_some() {
                        self.is_group_owner =
                            info.as_ref().unwrap().owner() == _ctx.props().user_id;
                    }
                    self.info = info;
                    true
                }
                QueryState::Failed => false,
            },
            PostCardMsg::ShowSetDrawer => {
                self.show_set_drawer = !self.show_set_drawer;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let id = ctx.props().id.clone();
        let mut set_drawer = html!();
        if self.show_set_drawer {
            let close = ctx.link().callback(|_| PostCardMsg::ShowSetDrawer);
            let delete = ctx.link().callback(|_| PostCardMsg::ShowSetDrawer);
            set_drawer = html! {
                <SetDrawer
                    conv_type={ctx.props().conv_type.clone()}
                    is_owner={self.is_group_owner}
                    {close}
                    {delete}/>
            }
        }
        html! {
        <div class="postcard">
            if !id.is_empty() && self.info.is_some(){
                <div class="pc-wrapper">
                    <span class="postcard-setting" onclick={ctx.link().callback(|_| PostCardMsg::ShowSetDrawer)}>
                        {"···"}
                    </span>
                    {set_drawer}
                <div class="header">
                    <div class="header-info">
                        <div >
                            <img class="postcard-avatar" src={self.info.as_ref().unwrap().avatar()} />
                        </div>
                        <div class="info">
                            <span class="name">
                                {self.info.as_ref().unwrap().name()}
                            </span>
                            <span class="num">
                                {"编号: "}{""}
                            </span>
                            <span class="region">
                                {"地区: "}{self.info.as_ref().unwrap().region()}
                            </span>
                        </div>
                    </div>

                </div>
                <div class="postcard-remark">
                    {"备注: "}{self.info.as_ref().unwrap().remark()}
                </div>
                <div class="sign">
                    {"签名: "}{self.info.as_ref().unwrap().signature()}
                </div>

                <Action id={id.clone()} conv_type={ctx.props().conv_type.clone()} />
            </div>
            }
        </div>
            }
    }
}
