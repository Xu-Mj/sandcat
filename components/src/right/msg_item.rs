use gloo::timers::callback::Timeout;
use nanoid::nanoid;
use yew::platform::spawn_local;
use yew::prelude::*;
use yewdux::Dispatch;

use abi::model::message::{
    GroupMsg, InviteMsg, InviteType, Message, Msg, SendStatus, ServerResponse,
};
use abi::model::user::UserWithMatchType;
use abi::model::ContentType;
use abi::model::RightContentType;
use abi::state::{SendCallState, SendMessageState};
use i18n::LanguageType;
use icons::{CycleIcon, MsgPhoneIcon, VideoRecordIcon};

use crate::right::friend_card::FriendCard;

pub struct MsgItem {
    avatar: AttrValue,
    show_img_preview: bool,
    show_friend_card: bool,
    // timeout for sending message
    timeout: Option<Timeout>,
    show_send_fail: bool,
    show_sending: bool,
    pointer: (i32, i32),
    friend_info: Option<UserWithMatchType>,
}

type FriendCardProps = (UserWithMatchType, i32, i32);
pub enum MsgItemMsg {
    PreviewImg,
    ShowFriendCard(MouseEvent),
    SpawnFriendCard(Box<FriendCardProps>),
    CallVideo,
    CallAudio,
    SendTimeout,
    ReSendMessage,
    QueryGroupMember(AttrValue),
    CloseFriendCard,
}

#[derive(Properties, Clone, PartialEq)]
pub struct MsgItemProps {
    pub user_id: AttrValue,
    pub friend_id: AttrValue,
    pub avatar: AttrValue,
    pub msg: Message,
    pub conv_type: RightContentType,
}

impl Component for MsgItem {
    type Message = MsgItemMsg;
    type Properties = MsgItemProps;

    fn create(ctx: &Context<Self>) -> Self {
        // query data by conv type
        if ctx.props().conv_type == RightContentType::Group && !ctx.props().msg.is_self {
            let friend_id = ctx.props().msg.send_id.clone();
            let group_id = ctx.props().msg.friend_id.clone();
            ctx.link().send_future(async move {
                let member = db::group_members()
                    .await
                    .get_by_group_id_and_friend_id(group_id.as_str(), friend_id.as_str())
                    .await
                    .unwrap();
                MsgItemMsg::QueryGroupMember(member.unwrap().avatar)
            });
        }

        let avatar = ctx.props().avatar.clone();
        let mut timeout = None;
        if ctx.props().msg.is_self && ctx.props().msg.send_status == SendStatus::Sending {
            let ctx = ctx.link().clone();
            timeout = Some(Timeout::new(3000, move || {
                ctx.send_message(MsgItemMsg::SendTimeout);
            }));
        }
        Self {
            timeout,
            show_img_preview: false,
            show_friend_card: false,
            avatar,
            show_send_fail: ctx.props().msg.send_status == SendStatus::Failed,
            show_sending: false,
            pointer: (0, 0),
            friend_info: None,
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        if ctx.props().msg.send_status != SendStatus::Sending {
            self.show_send_fail = false;
            self.timeout = None;
            self.show_sending = false;
        }
        true
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            MsgItemMsg::PreviewImg => {
                self.show_img_preview = !self.show_img_preview;
                true
            }
            MsgItemMsg::ShowFriendCard(event) => {
                // 获取xy
                let x = event.x();
                let y = event.y();

                // let friend_id = if ctx.props().msg.is_self {
                //     ctx.props().user_id.clone()
                // } else {
                //     ctx.props().friend_id.clone()
                // };
                log::debug!("friend id in msg item: {:?}", &ctx.props().msg);
                let is_self = ctx.props().msg.is_self;
                // the friend id is friend when msg type is single
                // because send id exchange the friend id when receive the message
                let friend_id = ctx.props().msg.friend_id.clone();
                let send_id = ctx.props().msg.send_id.clone();
                let user_id = ctx.props().user_id.clone();
                let conv_type = ctx.props().conv_type.clone();
                let group_id = ctx.props().friend_id.clone();
                ctx.link().send_future(async move {
                    // 查询好友信息
                    let user = if is_self {
                        let user = db::users().await.get(user_id.as_str()).await.unwrap();
                        UserWithMatchType::from(user)
                    } else {
                        match conv_type {
                            RightContentType::Friend => {
                                let friend = db::friends().await.get(friend_id.as_str()).await;
                                UserWithMatchType::from(friend)
                            }
                            // query group member
                            RightContentType::Group => {
                                let member = db::group_members()
                                    .await
                                    .get_by_group_id_and_friend_id(
                                        group_id.as_str(),
                                        send_id.as_str(),
                                    )
                                    .await
                                    .unwrap()
                                    .unwrap();
                                UserWithMatchType::from(member)
                            }
                            _ => UserWithMatchType::default(),
                        }
                    };

                    MsgItemMsg::SpawnFriendCard(Box::new((user, x, y)))
                    // FriendCard::show(
                    //     UserWithMatchType::from(user),
                    //     None,
                    //     LanguageType::EnUS,
                    //     true,
                    //     x,
                    //     y,
                    // );
                });
                false
            }
            MsgItemMsg::CallVideo => {
                Dispatch::<SendCallState>::global().reduce_mut(|s| {
                    s.msg = InviteMsg {
                        local_id: nanoid!().into(),
                        server_id: AttrValue::default(),
                        send_id: ctx.props().user_id.clone(),
                        friend_id: ctx.props().friend_id.clone(),
                        create_time: chrono::Local::now().timestamp_millis(),
                        invite_type: InviteType::Video,
                    }
                });
                false
            }
            MsgItemMsg::CallAudio => {
                Dispatch::<SendCallState>::global().reduce_mut(|s| {
                    s.msg = InviteMsg {
                        local_id: nanoid!().into(),
                        server_id: AttrValue::default(),
                        send_id: ctx.props().user_id.clone(),
                        friend_id: ctx.props().friend_id.clone(),
                        create_time: chrono::Local::now().timestamp_millis(),
                        invite_type: InviteType::Audio,
                    }
                });
                false
            }
            MsgItemMsg::QueryGroupMember(avatar) => {
                self.avatar = avatar;
                true
            }
            MsgItemMsg::SendTimeout => {
                if ctx.props().msg.send_status == SendStatus::Success {
                    self.timeout = None;
                    self.show_send_fail = false;
                    self.show_sending = false;
                    return true;
                }
                let msg_id = ctx.props().msg.local_id.clone();

                let conv_type = ctx.props().conv_type.clone();
                spawn_local(async move {
                    let msg = ServerResponse {
                        local_id: msg_id,
                        send_status: SendStatus::Failed,
                        err_msg: Some(AttrValue::from("TimeOut")),
                        ..Default::default()
                    };
                    match conv_type {
                        RightContentType::Friend => {
                            db::messages().await.update_msg_status(&msg).await.unwrap();
                        }
                        RightContentType::Group => {
                            db::group_msgs()
                                .await
                                .update_msg_status(&msg)
                                .await
                                .unwrap();
                        }
                        _ => {}
                    }
                });
                self.timeout = None;
                self.show_send_fail = true;
                self.show_sending = false;
                true
            }
            MsgItemMsg::ReSendMessage => {
                let mut msg = ctx.props().msg.clone();
                msg.send_status = SendStatus::Sending;
                let msg = match ctx.props().conv_type {
                    RightContentType::Friend => Msg::Single(msg),
                    RightContentType::Group => Msg::Group(GroupMsg::Message(msg)),
                    _ => return false,
                };

                // send message
                log::debug!("send message in message item ");
                Dispatch::<SendMessageState>::global().reduce_mut(|s| s.msg = msg);

                let ctx = ctx.link().clone();
                self.timeout = Some(Timeout::new(3000, move || {
                    ctx.send_message(MsgItemMsg::SendTimeout);
                }));
                self.show_send_fail = false;
                self.show_sending = true;
                true
            }
            MsgItemMsg::SpawnFriendCard(b) => {
                self.show_friend_card = true;
                self.friend_info = Some(b.0);
                self.pointer = (b.1, b.2);
                true
            }
            MsgItemMsg::CloseFriendCard => {
                self.show_friend_card = false;
                self.friend_info = None;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let id = ctx.props().msg.create_time;
        let mut classes = Classes::from("msg-item");
        let msg_type = ctx.props().msg.content_type;

        let mut msg_content_classes = Classes::from("msg-item-content");
        if ctx.props().msg.is_self {
            msg_content_classes.push("background-self");
            classes = Classes::from("msg-item-reverse");
        } else {
            msg_content_classes.push("background-other");
        }

        let content = match msg_type {
            ContentType::Text => {
                html! {
                    <div class={msg_content_classes}>
                        {ctx.props().msg.content.clone()}
                    </div>
                }
            }
            // todo 限制图片宽度，高度自适应，聊天列表展示缩略图，点击查看原图
            ContentType::Image => {
                let img_url = if ctx.props().msg.file_content.is_empty() {
                    AttrValue::from(format!("/api/file/get/{}", ctx.props().msg.content.clone()))
                } else {
                    ctx.props().msg.file_content.clone()
                };
                let src = img_url.clone();
                let onclick = ctx
                    .link()
                    .callback(move |_: MouseEvent| MsgItemMsg::PreviewImg);
                let img_preview = if self.show_img_preview {
                    html! {
                        <div class="img-preview pointer" onclick={onclick.clone()}>
                            <img src={src} />
                        </div>
                    }
                } else {
                    html!()
                };
                html! {
                <>
                    {img_preview}
                    <div class="msg-item-content pointer">
                        <div class="img-mask">
                        </div>
                        <img class="msg-item-img" src={img_url} {onclick}/>
                    </div>
                </>
                }
            }
            ContentType::Video => html! {
                <div class="msg-item-content">
                    <video class="msg-item-video">
                        <source src={ctx.props().msg.content.clone()} type="video/mp4" />
                    </video>
                </div>
            },
            ContentType::File => {
                let full = ctx.props().msg.content.clone();
                let file_name = ctx.props().msg.content.split('-').last().unwrap_or(&full);
                html! {
                    <div class="msg-item-content">
                        <span class="msg-item-file-name">
                            {file_name}
                        </span>
                    </div>
                }
            }
            ContentType::Emoji => {
                html! {
                    <div class="msg-item-emoji">
                        // <span class="msg-item-emoji">
                            <img class="emoji" src={ctx.props().msg.content.clone()} />
                        // </span>
                    </div>
                }
            }
            ContentType::VideoCall => {
                let onclick = ctx.link().callback(|_| MsgItemMsg::CallVideo);
                html! {
                    <div class={msg_content_classes} {onclick} style="cursor: pointer;">
                        {ctx.props().msg.content.clone()}
                        {"\t"}
                        <VideoRecordIcon/>
                    </div>
                }
            }
            ContentType::AudioCall => {
                let onclick = ctx.link().callback(|_| MsgItemMsg::CallAudio);
                html! {
                    <div class={msg_content_classes} {onclick} style="cursor: pointer;">
                        {ctx.props().msg.content.clone()}
                        {"\t"}
                         <MsgPhoneIcon />
                    </div>
                }
            }
            ContentType::Default => html!(),
            ContentType::Audio => html!(),
            ContentType::Error => html!(),
        };

        let _avatar_click = ctx.link().callback(MsgItemMsg::ShowFriendCard);

        // send status
        let mut send_status = html!();
        if self.show_send_fail {
            let onclick = ctx.link().callback(|_| MsgItemMsg::ReSendMessage);
            send_status = html! {
                <div class="msg-send-failed" {onclick}>
                    <span class="pointer">
                        {"!"}
                    </span>
                </div>
            };
        } else if self.show_sending {
            send_status = html! {
                <div class="msg-sending">
                    <CycleIcon/>
                </div>
            };
        }

        let mut friend_card = html!();
        if self.show_friend_card {
            friend_card = html! {
                <FriendCard
                    friend_info={self.friend_info.as_ref().unwrap().clone()}
                    user_id={ctx.props().user_id.clone()}
                    lang={LanguageType::ZhCN}
                    close={ctx.link().callback(|_| MsgItemMsg::CloseFriendCard)}
                    is_self={ctx.props().msg.is_self}
                    x={self.pointer.0}
                    y={self.pointer.1}
                />
            };
        }

        let avatar = if ctx.props().msg.is_self {
            html!(<img class="avatar" src={self.avatar.clone()} />)
        } else {
            html!(<img class="avatar pointer" src={self.avatar.clone()} onclick={_avatar_click} />)
        };

        html! {
            <>
            {friend_card}
            <div class={classes} id={id.to_string()} >
                <div class="msg-item-avatar">
                    {avatar}
                </div>
                <div class="content-wrapper">
                    {content}
                </div>
                {send_status}
            </div>
            </>
        }
    }
}
