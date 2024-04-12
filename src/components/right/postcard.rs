use std::rc::Rc;

use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use crate::components::action::Action;
use crate::components::right::set_drawer::SetDrawer;
use crate::model::group::GroupDelete;
use crate::model::{ItemInfo, RightContentType};
use crate::pages::{ItemType, RemoveConvState, RemoveFriendState};
use crate::{api, db};

#[derive(Properties, Clone, PartialEq)]
pub struct PostCardProps {
    pub id: AttrValue,
    pub user_id: AttrValue,
    pub conv_type: RightContentType,
}

pub enum PostCardMsg {
    QueryInformation(QueryState<Option<Box<dyn ItemInfo>>>),
    Delete,
    ShowSetDrawer,
    None,
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
    remove_conv_state: Rc<RemoveConvState>,
    _remove_conv_listener: ContextHandle<Rc<RemoveConvState>>,
    remove_friend_state: Rc<RemoveFriendState>,
    _remove_friend_listener: ContextHandle<Rc<RemoveFriendState>>,
}

impl Component for PostCard {
    type Message = PostCardMsg;
    type Properties = PostCardProps;

    fn create(ctx: &Context<Self>) -> Self {
        log::debug!("postcard conv type:{:?}", ctx.props().conv_type.clone());
        let (remove_conv_state, _remove_conv_listener) = ctx
            .link()
            .context(ctx.link().callback(|_| PostCardMsg::None))
            .expect("postcard remove_conv_state needed");
        let (remove_friend_state, _remove_friend_listener) = ctx
            .link()
            .context(ctx.link().callback(|_| PostCardMsg::None))
            .expect("postcard friend_state needed");
        let self_ = PostCard {
            info: None,
            show_set_drawer: false,
            is_group_owner: false,
            remove_conv_state,
            _remove_conv_listener,
            remove_friend_state,
            _remove_friend_listener,
        };
        self_.query(ctx);
        self_
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.reset();
        self.query(ctx);
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PostCardMsg::QueryInformation(state) => match state {
                QueryState::Querying => true,
                QueryState::Success(info) => {
                    if info.is_some() {
                        self.is_group_owner = info.as_ref().unwrap().owner() == ctx.props().user_id;
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
            PostCardMsg::Delete => {
                // delete data from local database
                if self.info.is_none() {
                    return false;
                }
                let info = self.info.as_ref().unwrap();
                match info.get_type() {
                    RightContentType::Friend => {}
                    RightContentType::Group => {
                        let is_dismiss = self.is_group_owner;

                        // delete data from local database
                        let id = info.id();
                        let user_id = ctx.props().user_id.clone().to_string();

                        spawn_local(async move {
                            if !is_dismiss {
                                log::debug!("group is dismissed already, only delete local data");
                                // check the group is dismissed already
                                let group = db::groups().await.get(&id).await.unwrap().unwrap();
                                if group.deleted {
                                    if let Err(e) = db::groups().await.delete(id.as_str()).await {
                                        log::error!("delete group failed: {:?}", e);
                                    }
                                    // delete conversation
                                    if let Err(e) = db::convs().await.delete(id.as_str()).await {
                                        log::error!("delete conversation failed: {:?}", e);
                                    }
                                    return;
                                }
                            }
                            // send leave group request
                            match api::groups()
                                .delete_group(GroupDelete {
                                    group_id: id.to_string(),
                                    user_id,
                                    is_dismiss,
                                })
                                .await
                            {
                                Ok(_) => {
                                    log::debug!("send delete group request success");
                                    if let Err(e) = db::groups().await.delete(id.as_str()).await {
                                        log::error!("delete group failed: {:?}", e);
                                    }
                                    // delete conversation
                                    if let Err(e) = db::convs().await.delete(id.as_str()).await {
                                        log::error!("delete conversation failed: {:?}", e);
                                    }
                                }
                                Err(e) => {
                                    log::error!("send delete group request error: {:?}", e);
                                }
                            }
                        });

                        // todo move below to single patch, handle the http request error
                        // send state message to remove conversation from conversation lis
                        self.remove_conv_state.remove_event.emit(info.id());

                        // send state message to remove friend from friend list
                        self.remove_friend_state
                            .remove_event
                            .emit((info.id(), ItemType::Group));
                    }
                    _ => {}
                }

                true
            }
            PostCardMsg::None => false,
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let id = ctx.props().id.clone();
        let mut set_drawer = html!();
        if self.show_set_drawer {
            let close = ctx.link().callback(|_| PostCardMsg::ShowSetDrawer);
            let delete = ctx.link().callback(|_| PostCardMsg::Delete);
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
                        // <div >
                        //     <img class="postcard-avatar" src={self.info.as_ref().unwrap().avatar()} />
                        // </div>
                        {self.get_avatar()}
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

impl PostCard {
    fn get_avatar(&self) -> Html {
        // deal with group avatars
        let avatar_str = self.info.as_ref().unwrap().avatar();

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
                    <img class="avatar" src={v.to_owned()} />
                }
            })
            .collect::<Html>();
        html! {
            <div class="item-avatar" style={avatar_style}>
                {avatar}
            </div>
        }
    }

    fn query(&self, ctx: &Context<Self>) {
        ctx.link()
            .send_message(PostCardMsg::QueryInformation(QueryState::Querying));
        let id = ctx.props().id.clone();
        log::debug!("friend_id :{:?}", &id);
        if !id.is_empty() {
            match ctx.props().conv_type {
                RightContentType::Friend => {
                    ctx.link().send_future(async move {
                        let user_info = db::friends().await.get_friend(id.as_str()).await;
                        log::debug!("user info :{:?}", user_info);
                        PostCardMsg::QueryInformation(QueryState::Success(Some(Box::new(
                            user_info,
                        ))))
                    });
                }
                RightContentType::Group => ctx.link().send_future(async move {
                    match db::groups().await.get(id.as_str()).await {
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
