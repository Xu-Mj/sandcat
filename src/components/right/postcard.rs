use fluent::{FluentBundle, FluentResource};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;
use yewdux::Dispatch;

use crate::components::action::Action;
use crate::components::right::set_drawer::SetDrawer;
use crate::i18n::{en_us, zh_cn, LanguageType};
use crate::model::group::GroupDelete;
use crate::model::{ItemInfo, RightContentType};
use crate::pages::ItemType;
use crate::state::{RemoveConvState, RemoveFriendState};
use crate::{api, db, tr, utils};

#[derive(Properties, Clone, PartialEq)]
pub struct PostCardProps {
    pub id: AttrValue,
    pub user_id: AttrValue,
    pub conv_type: RightContentType,
    pub lang: LanguageType,
}

pub enum PostCardMsg {
    QueryInformation(QueryState<Option<Box<dyn ItemInfo>>>),
    Delete,
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
    i18n: FluentBundle<FluentResource>,
}

impl Component for PostCard {
    type Message = PostCardMsg;
    type Properties = PostCardProps;

    fn create(ctx: &Context<Self>) -> Self {
        log::debug!("postcard conv type:{:?}", ctx.props().conv_type.clone());
        let res = match ctx.props().lang {
            LanguageType::ZhCN => zh_cn::POSTCARD,
            LanguageType::EnUS => en_us::POSTCARD,
        };
        let i18n = utils::create_bundle(res);
        let self_ = PostCard {
            info: None,
            show_set_drawer: false,
            is_group_owner: false,
            i18n,
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
                let user_id = ctx.props().user_id.clone().to_string();
                let id = info.id();

                match info.get_type() {
                    RightContentType::Friend => {
                        spawn_local(async move {
                            // send delete friend request
                            match api::friends().delete_friend(user_id, id.to_string()).await {
                                Ok(_) => {
                                    // delete data from local storage
                                    if let Err(err) = db::friends().await.delete_friend(&id).await {
                                        log::error!("delete friend failed: {:?}", err);
                                    } else {
                                        // delete conversation
                                        if let Err(e) = db::convs().await.delete(id.as_str()).await
                                        {
                                            log::error!("delete conversation failed: {:?}", e);
                                        }
                                        log::debug!("delete friend success");
                                        // send state message to remove conversation from conversation lis
                                        Dispatch::<RemoveConvState>::global()
                                            .reduce_mut(|s| s.id = id.clone());
                                        // send state message to remove friend from friend list
                                        Dispatch::<RemoveFriendState>::global().reduce_mut(|s| {
                                            s.id = id;
                                            s.type_ = ItemType::Friend;
                                        });
                                    }
                                }
                                Err(e) => {
                                    log::error!("delete friend failed: {:?}", e);
                                }
                            }
                        });
                    }
                    RightContentType::Group => {
                        let is_dismiss = self.is_group_owner;

                        // delete data from local database
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
                                    // send state message to remove conversation from conversation lis
                                    Dispatch::<RemoveConvState>::global()
                                        .reduce_mut(|s| s.id = id.clone());
                                    // send state message to remove friend from friend list
                                    Dispatch::<RemoveFriendState>::global().reduce_mut(|s| {
                                        s.id = id;
                                        s.type_ = ItemType::Group;
                                    });
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
                                    if let Err(e) = db::groups().await.delete(&id).await {
                                        log::error!("delete group failed: {:?}", e);
                                    }
                                    // delete conversation
                                    if let Err(e) = db::convs().await.delete(&id).await {
                                        log::error!("delete conversation failed: {:?}", e);
                                    }
                                    // send state message to remove conversation from conversation lis
                                    Dispatch::<RemoveConvState>::global()
                                        .reduce_mut(|s| s.id = id.clone());
                                    // send state message to remove friend from friend list
                                    Dispatch::<RemoveFriendState>::global().reduce_mut(|s| {
                                        s.id = id;
                                        s.type_ = ItemType::Group;
                                    });
                                }
                                Err(e) => {
                                    log::error!("send delete group request error: {:?}", e);
                                }
                            }
                        });
                    }
                    _ => {}
                }

                true
            }
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
                    {delete}
                    lang={ctx.props().lang}/>
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
                                {tr!(self.i18n, "account")}{self.info.as_ref().unwrap().id()}
                            </span>
                            <span class="region">
                                {tr!(self.i18n, "region")}{self.info.as_ref().unwrap().region()}
                            </span>
                        </div>
                    </div>

                </div>
                <div class="postcard-remark">
                    {tr!(self.i18n, "remark")}{self.info.as_ref().unwrap().remark()}
                </div>
                <div class="sign">
                    {tr!(self.i18n, "signature")}{self.info.as_ref().unwrap().signature()}
                </div>

                <Action id={id.clone()} conv_type={ctx.props().conv_type.clone()} lang={ctx.props().lang} />
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
                        let user_info = db::friends().await.get(id.as_str()).await;
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
