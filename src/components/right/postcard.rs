use crate::api::user::get_info_by_id;
use crate::model::user::User;
use crate::{
    components::action::Action,
    db::{friend::FriendRepo, RightContentType},
    model::friend::Friend,
};
use wasm_bindgen::JsValue;
use yew::prelude::*;

#[derive(Properties, Clone, PartialEq)]
pub struct PostCardProps {
    pub friend_id: AttrValue,
    pub user_id: AttrValue,
    pub conv_type: RightContentType,
}

pub enum PostCardMsg {
    QueryFriend(QueryState<Friend>),
    QueryUser(QueryState<User>),
    ApplyFriend,
    ApplyFriendRes,
}

pub enum QueryState<T> {
    Querying,
    Success(T),
    Failed(JsValue),
}

pub struct PostCard {
    pub friend_info: Friend,
    pub user_info: User,
}

impl PostCard {
    fn query(&self, ctx: &Context<Self>) {
        let friend_id = ctx.props().friend_id.clone();
        log::debug!("friend_id :{:?}", &friend_id);
        if friend_id != AttrValue::default() {
            match ctx.props().conv_type {
                RightContentType::UserInfo => {
                    // 通过网络获取
                    ctx.link().send_future(async move {
                        match get_info_by_id(friend_id.to_string()).await {
                            Ok(user) => {
                                log::debug!("查询成功:{:?}", &user);
                                PostCardMsg::QueryUser(QueryState::Success(user))
                            }
                            Err(err) => PostCardMsg::QueryUser(QueryState::Failed(err)),
                        }
                    });
                    ctx.link()
                        .send_message(PostCardMsg::QueryUser(QueryState::Querying))
                }
                RightContentType::Friend => {
                    ctx.link().send_future(async move {
                        let user_info = FriendRepo::new().await.get_friend(friend_id).await;
                        PostCardMsg::QueryFriend(QueryState::Success(user_info))
                    });
                }
                RightContentType::Group => {}
                _ => {}
            }
        }
    }

    fn reset(&mut self) {
        self.friend_info = Friend::default();
    }
}

impl Component for PostCard {
    type Message = PostCardMsg;
    type Properties = PostCardProps;

    fn create(ctx: &Context<Self>) -> Self {
        log::debug!("postcard conv type:{:?}", ctx.props().conv_type.clone());
        let self_ = PostCard {
            friend_info: Friend::default(),
            user_info: User::default(),
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
            PostCardMsg::QueryFriend(state) => match state {
                QueryState::Querying => true,
                QueryState::Success(user_info) => {
                    self.friend_info = user_info;
                    true
                }
                QueryState::Failed(_) => false,
            },
            PostCardMsg::QueryUser(state) => match state {
                QueryState::Querying => true,
                QueryState::Success(user_info) => {
                    let friend_info = Friend {
                        id: Default::default(),
                        friend_id: user_info.id,
                        remark: None,
                        status: Default::default(),
                        create_time: Default::default(),
                        update_time: Default::default(),
                        from: None,
                        name: user_info.name,
                        account: user_info.account,
                        avatar: user_info.avatar,
                        gender: user_info.gender,
                        age: user_info.age,
                        phone: user_info.phone,
                        email: user_info.email,
                        address: user_info.address,
                        birthday: user_info.birthday,
                        hello: None,
                    };
                    self.friend_info = friend_info;
                    true
                }
                QueryState::Failed(_) => false,
            },
            PostCardMsg::ApplyFriend => {
                // 发送好友请求
                /* let new_friend = FriendShipRequest {
                    user_id: ctx.props().user_id.clone(),
                    friend_id: ctx.props().friend_id.clone(),
                    status: AttrValue::from("2"),
                    apply_msg: None,
                    source: None,
                };
                ctx.link().send_future(async {
                    if let Err(err) = apply_friend(new_friend).await {
                        log::error!("发送好友申请错误: {:?}", err);
                    }
                    PostCardMsg::ApplyFriendRes
                }); */
                false
            }
            PostCardMsg::ApplyFriendRes => {
                // 请求结果
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let id = ctx.props().friend_id.clone();
        let apply_friend = ctx.link().callback(|_| PostCardMsg::ApplyFriend);
        let action = match ctx.props().conv_type {
            RightContentType::UserInfo => {
                html! {
                    <div class="apply" >
                        <button onclick={apply_friend} >
                            {"申请好友"}
                        </button>
                    </div>
                }
            }
            _ => {
                html! {
                    <Action id={id.clone()} conv_type={ctx.props().conv_type.clone()} />
                }
            }
        };
        html! {
        <div class="postcard">
            if id != AttrValue::default() {
                <div class="pc-wrapper">
                <div class="header">
                    <div class="header-info">
                        <div >
                            <img class="postcard-avatar" src={self.friend_info.avatar.clone()} />
                        </div>
                        <div class="info">
                            <span class="name">
                                {self.friend_info.name.clone()}
                            </span>
                            <span class="num">
                                {"编号: "}{""}
                            </span>
                            <span class="region">
                                {"地区: "}{self.friend_info.address.clone()}
                            </span>
                        </div>
                    </div>

                </div>
                <div class="postcard-remark">
                    {"备注: "}{self.friend_info.remark.clone()}
                </div>
                <div class="sign">
                    {"签名: "}{self.friend_info.remark.clone()}
                </div>

                {action}
            </div>
            }
        </div>
            }
    }
}
