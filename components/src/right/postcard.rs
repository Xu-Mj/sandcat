use fluent::{FluentBundle, FluentResource};
use log::error;
use sandcat_sdk::state::UpdateFriendState;
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use i18n::{en_us, zh_cn, LanguageType};
use sandcat_sdk::api;
use sandcat_sdk::db;
use sandcat_sdk::model::friend::Friend;
use sandcat_sdk::model::group::{Group, GroupDelete};
use sandcat_sdk::model::RightContentType;
use sandcat_sdk::pb::message::FriendInfo;
use sandcat_sdk::state::MobileState;
use sandcat_sdk::state::Notify;
use sandcat_sdk::state::{ItemType, RemoveConvState, RemoveFriendState};
use utils::tr;
use yewdux::Dispatch;

use crate::action::Action;
use crate::constant::ACCOUNT;
use crate::constant::ANNOUNCEMENT;
use crate::constant::REGION;
use crate::constant::REMARK;
use crate::constant::SIGNATURE;
use crate::right::set_drawer::SetDrawer;

use super::util;

#[derive(Properties, Clone, PartialEq)]
pub struct PostCardProps {
    pub id: AttrValue,
    pub user_id: AttrValue,
    pub avatar: AttrValue,
    pub nickname: AttrValue,
    pub conv_type: RightContentType,
    pub lang: LanguageType,
}

pub enum PostCardMsg {
    QueryFriend(QueryState<Option<Friend>>),
    QueryGroup(QueryState<Option<Group>>),
    Delete,
    ShowSetDrawer,
    QueryFriendByHttp(QueryState<FriendInfo>),
    UpdateRemark,
}

pub enum QueryState<T> {
    Querying,
    Success(T),
    Failed,
}

pub struct PostCard {
    node: NodeRef,
    group: Option<Group>,
    friend: Option<Friend>,
    is_group_owner: bool,
    show_set_drawer: bool,
    i18n: FluentBundle<FluentResource>,
}

impl Component for PostCard {
    type Message = PostCardMsg;
    type Properties = PostCardProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self::query(ctx);
        log::debug!("postcard conv type:{:?}", ctx.props().conv_type.clone());
        let res = match ctx.props().lang {
            LanguageType::ZhCN => zh_cn::POSTCARD,
            LanguageType::EnUS => en_us::POSTCARD,
        };
        let i18n = utils::create_bundle(res);

        PostCard {
            node: NodeRef::default(),
            show_set_drawer: false,
            is_group_owner: false,
            i18n,
            group: None,
            friend: None,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.reset();
        Self::query(ctx);
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            PostCardMsg::QueryFriend(state) => match state {
                QueryState::Querying => true,
                QueryState::Success(info) => {
                    self.friend = info;
                    true
                }
                QueryState::Failed => false,
            },
            PostCardMsg::QueryGroup(state) => match state {
                QueryState::Querying => true,
                QueryState::Success(info) => {
                    if info.is_some() {
                        self.is_group_owner = info.as_ref().unwrap().owner == ctx.props().user_id;
                    }
                    self.group = info;
                    true
                }
                QueryState::Failed => false,
            },
            PostCardMsg::QueryFriendByHttp(state) => match state {
                QueryState::Querying => true,
                QueryState::Success(info) => self.update_friend(info),
                QueryState::Failed => false,
            },
            PostCardMsg::ShowSetDrawer => {
                self.show_set_drawer = !self.show_set_drawer;
                true
            }
            PostCardMsg::Delete => {
                // delete data from local database
                let user_id = ctx.props().user_id.clone().to_string();
                match ctx.props().conv_type {
                    RightContentType::Friend => {
                        if let Some(ref friend) = self.friend {
                            self.delete_friend(
                                friend.fs_id.clone(),
                                user_id,
                                friend.friend_id.clone(),
                            );
                        }
                    }
                    RightContentType::Group => {
                        if let Some(ref group) = self.group {
                            self.delete_group(user_id, group.id.clone());
                        }
                    }
                    _ => {}
                }

                true
            }
            PostCardMsg::UpdateRemark => {
                let user_id = ctx.props().user_id.clone().to_string();
                let remark = self.node.cast::<HtmlInputElement>().unwrap().value();
                match ctx.props().conv_type {
                    RightContentType::Friend => {
                        let mut friend = self.friend.as_ref().unwrap().clone();
                        friend.remark = Some(remark.into());
                        util::update_friend_remark(user_id, friend);
                    }
                    RightContentType::Group => self.update_group(remark),
                    _ => {}
                }
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
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
        let content = match ctx.props().conv_type {
            RightContentType::Friend => self.get_friend_html(ctx, set_drawer),
            RightContentType::Group => self.get_group_html(ctx, set_drawer),
            _ => html! {},
        };
        html! {
        <div class="postcard">
            {content}
        </div>
        }
    }
}

impl PostCard {
    fn get_avatar(&self, avatar_str: &AttrValue) -> Html {
        // deal with group avatars

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
                    <img class="avatar" alt="avatar" src={utils::get_avatar_url(v)} />
                }
            })
            .collect::<Html>();
        html! {
            <div class="item-avatar" style={avatar_style}>
                {avatar}
            </div>
        }
    }

    fn query(ctx: &Context<Self>) {
        let id = ctx.props().id.clone();
        if id.is_empty() {
            return;
        }
        match ctx.props().conv_type {
            RightContentType::Friend => {
                ctx.link()
                    .send_message(PostCardMsg::QueryFriend(QueryState::Querying));
                let clone_id = id.clone();
                ctx.link().send_future(async move {
                    let user_info = db::db_ins().friends.get(&clone_id).await.unwrap().unwrap();
                    log::debug!("user info :{:?}", user_info);
                    PostCardMsg::QueryFriend(QueryState::Success(Some(user_info)))
                });
                ctx.link().send_future(async move {
                    // send http request
                    match api::friends().query_friend(&id).await {
                        Ok(friend) => PostCardMsg::QueryFriendByHttp(QueryState::Success(friend)),
                        Err(e) => {
                            error!("query friend error: {:?}", e);
                            PostCardMsg::QueryFriendByHttp(QueryState::Failed)
                        }
                    }
                })
            }
            RightContentType::Group => {
                ctx.link()
                    .send_message(PostCardMsg::QueryFriend(QueryState::Querying));
                ctx.link().send_future(async move {
                    match db::db_ins().groups.get(id.as_str()).await {
                        Ok(Some(group)) => {
                            log::debug!("group info :{:?}", group);
                            PostCardMsg::QueryGroup(QueryState::Success(Some(group)))
                        }
                        _ => PostCardMsg::QueryGroup(QueryState::Failed),
                    }
                })
            }
            _ => {}
        }
    }

    fn reset(&mut self) {
        self.group = None;
        self.friend = None;
    }

    fn update_group(&self, remark: String) {
        let mut group = self.group.as_ref().unwrap().clone();
        group.remark = Some(remark.into());
        spawn_local(async move {
            if let Err(err) = db::db_ins().groups.put(&group).await {
                error!("update group error: {:?}", err);
            }
            Dispatch::<UpdateFriendState>::global().reduce_mut(|s| {
                s.id = group.id;
                s.name = None;
                s.remark = group.remark;
                s.avatar = None;
                s.type_ = ItemType::Group
            });
        });
    }

    fn update_friend(&mut self, info: FriendInfo) -> bool {
        let mut need_update = false;
        if let Some(ref mut friend) = self.friend {
            if friend.friend_id == info.id {
                let email = info.email.map(|v| v.into());
                let region = info.region.map(|v| v.into());
                if friend.name != info.name
                    || friend.account != info.account
                    || friend.avatar != info.avatar
                    || friend.gender != info.gender
                    || friend.signature != info.signature
                    || friend.email != email
                    || friend.region != region
                {
                    need_update = true;
                }
                if need_update {
                    friend.account = info.account.into();
                    friend.name = info.name.into();
                    friend.avatar = info.avatar.into();
                    friend.email = email;
                    friend.gender = info.gender.into();
                    friend.region = region;
                    friend.signature = info.signature.into();
                    let friend = friend.clone();
                    spawn_local(async move {
                        if let Err(err) = db::db_ins().friends.put_friend(&friend).await {
                            error!("save friend error:{:?}", err);
                        }
                    });
                }
            }
        }
        need_update
    }

    fn delete_friend(&self, fs_id: AttrValue, user_id: String, id: AttrValue) {
        spawn_local(async move {
            // send delete friend request
            match api::friends()
                .delete_friend(fs_id.to_string(), user_id, id.to_string())
                .await
            {
                Ok(_) => {
                    // delete data from local storage
                    if let Err(err) = db::db_ins().friends.delete_friend(&id).await {
                        log::error!("delete friend failed: {:?}", err);
                    } else {
                        // delete conversation
                        if let Err(e) = db::db_ins().convs.delete(id.as_str()).await {
                            log::error!("delete conversation failed: {:?}", e);
                        }

                        // record the offline time, delete friend is same as friendships synchronized
                        if let Err(err) = db::db_ins()
                            .offline_time
                            .save(chrono::Utc::now().timestamp_millis())
                            .await
                        {
                            error!("save offline time error: {:?}", err);
                        }
                        log::debug!("delete friend success");
                        // send state message to remove conversation from conversation lis
                        RemoveConvState::remove(id.clone());
                        // send state message to remove friend from friend list
                        RemoveFriendState::remove(id, ItemType::Friend);
                    }
                }
                Err(e) => {
                    log::error!("delete friend failed: {:?}", e);
                }
            }
        });
    }

    fn delete_group(&self, user_id: String, id: AttrValue) {
        let is_dismiss = self.is_group_owner;

        // delete data from local database
        spawn_local(async move {
            if !is_dismiss {
                log::debug!("group is dismissed already, only delete local data");
                // check the group is dismissed already
                let group = db::db_ins().groups.get(&id).await.unwrap().unwrap();
                if group.deleted {
                    if let Err(e) = db::db_ins().groups.delete(id.as_str()).await {
                        log::error!("delete group failed: {:?}", e);
                    }
                    // delete conversation
                    if let Err(e) = db::db_ins().convs.delete(id.as_str()).await {
                        log::error!("delete conversation failed: {:?}", e);
                    }
                    // send state message to remove conversation from conversation lis
                    RemoveConvState::remove(id.clone());
                    // send state message to remove friend from friend list
                    RemoveFriendState::remove(id, ItemType::Group);
                    return;
                }
            }
            // send leave group request
            match api::groups()
                .delete(GroupDelete {
                    group_id: id.to_string(),
                    user_id,
                    is_dismiss,
                })
                .await
            {
                Ok(_) => {
                    log::debug!("send delete group request success");
                    if let Err(e) = db::db_ins().groups.delete(&id).await {
                        log::error!("delete group failed: {:?}", e);
                    }
                    // delete conversation
                    if let Err(e) = db::db_ins().convs.delete(&id).await {
                        log::error!("delete conversation failed: {:?}", e);
                    }
                    // send state message to remove conversation from conversation lis
                    RemoveConvState::remove(id.clone());
                    // send state message to remove friend from friend list
                    RemoveFriendState::remove(id, ItemType::Group);
                }
                Err(e) => {
                    log::error!("send delete group request error: {:?}", e);
                }
            }
        });
    }

    fn get_friend_html(&self, ctx: &Context<Self>, set_drawer: Html) -> Html {
        if let Some(friend) = self.friend.as_ref() {
            let class = match *MobileState::get() {
                MobileState::Desktop => "pc-wrapper pc-wrapper-size",
                MobileState::Mobile => "pc-wrapper pc-wrapper-size-mobile",
            };

            let onchange = ctx.link().callback(|_| PostCardMsg::UpdateRemark);

            html! {
                <div {class}>
                    <span class="postcard-setting" onclick={ctx.link().callback(|_| PostCardMsg::ShowSetDrawer)}>
                        {"···"}
                    </span>
                    {set_drawer}
                    <div class="header-info">
                        {self.get_avatar(&friend.avatar)}
                        <div class="info">
                            <span class="name">
                                {&friend.name}
                            </span>
                            <span class="num">
                                {tr!(self.i18n, ACCOUNT)}{&friend.friend_id}
                            </span>
                            <span class="region">
                                {tr!(self.i18n, REGION)}{friend.region.clone()}
                            </span>
                        </div>
                    </div>
                <div class="postcard-remark">
                    <b>{tr!(self.i18n, REMARK)}{":\t\t"}</b>
                    <input ref={self.node.clone()} value={friend.remark.clone()} placeholder={tr!(self.i18n, REMARK)} {onchange} />
                </div>
                <div class="sign">
                    <b>{tr!(self.i18n, SIGNATURE)}{":\t\t"}</b>{friend.signature.clone()}
                </div>

                <Action friend_id={&friend.friend_id}
                    user_id={&ctx.props().user_id}
                    avatar={&ctx.props().avatar}
                    nickname={&ctx.props().nickname}
                    conv_type={ctx.props().conv_type.clone()}
                    lang={ctx.props().lang} />
            </div>
            }
        } else {
            html!()
        }
    }

    fn get_group_html(&self, ctx: &Context<Self>, set_drawer: Html) -> Html {
        if let Some(group) = self.group.as_ref() {
            let class = match *MobileState::get() {
                MobileState::Desktop => "pc-wrapper pc-wrapper-size",
                MobileState::Mobile => "pc-wrapper pc-wrapper-size-mobile",
            };

            let onchange = ctx.link().callback(|_| PostCardMsg::UpdateRemark);
            html! {
                <div {class}>
                    <span class="postcard-setting" onclick={ctx.link().callback(|_| PostCardMsg::ShowSetDrawer)}>
                        {"···"}
                    </span>
                    {set_drawer}
                <div class="header">
                    <div class="header-info">
                        {self.get_avatar(&group.avatar)}
                        <div class="info">
                            <span class="name">
                                {&group.name}
                            </span>
                            <span class="num">
                                <b>{tr!(self.i18n, ACCOUNT)}</b>{&group.id}
                            </span>
                        </div>
                    </div>

                </div>
                <div class="postcard-remark">
                    <b>{tr!(self.i18n, REMARK)}{":\t\t"}</b>
                    <input ref={self.node.clone()} value={group.remark.clone()} placeholder={tr!(self.i18n, REMARK)} {onchange} />
                </div>
                <div class="sign">
                    <b>{tr!(self.i18n, ANNOUNCEMENT)}{":\t\t"}</b>{&group.announcement}
                </div>

                <Action friend_id={&group.id}
                    user_id={&ctx.props().user_id}
                    avatar={&ctx.props().avatar}
                    nickname={&ctx.props().nickname}
                    conv_type={ctx.props().conv_type.clone()}
                    lang={ctx.props().lang} />
            </div>
            }
        } else {
            html!()
        }
    }
}
