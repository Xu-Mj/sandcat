use gloo::timers::callback::Timeout;
use gloo::utils::{document, window};
use log::error;
use web_sys::Node;
use yew::platform::spawn_local;
use yew::prelude::*;
use yewdux::Dispatch;

use i18n::LanguageType;
use icons::HangUpLoadingIcon;
use sandcat_sdk::db;
use sandcat_sdk::model::message::{GroupMsg, InviteType, Message, Msg, SendStatus, ServerResponse};
use sandcat_sdk::model::user::UserWithMatchType;
use sandcat_sdk::model::ContentType;
use sandcat_sdk::model::RightContentType;
use sandcat_sdk::state::{I18nState, Notify, RelatedMsgState, SendMessageState};

use crate::right::friend_card::FriendCard;
use crate::right::msg_item::related_msg::RelatedMsg;
use crate::right::msg_right_click::MsgRightClick;
use crate::select_friends::SelectFriendList;

use super::{AudioDownloadStage, MsgItem};

pub enum MsgItemMsg {
    PreviewImg,
    ShowFriendCard(MouseEvent),
    SpawnFriendCard(Box<FriendCardProps>),
    CallVideo,
    CallAudio,
    SendTimeout,
    ReSendMessage,
    QueryGroupMember(AttrValue, AttrValue),
    CloseFriendCard,
    TextDoubleClick(MouseEvent),
    PlayAudio,
    ShowAudioDownload,
    AudioDownloadTimeout,
    OnContextMenu(MouseEvent),
    CloseContextMenu,
    DeleteItem,
    ShowForwardMsg,
    ForwardMsg(Vec<String>),
    RelatedMsg,
}

type FriendCardProps = (UserWithMatchType, i32, i32);

#[derive(Properties, Clone, PartialEq)]
pub struct MsgItemProps {
    pub user_id: AttrValue,
    pub friend_id: AttrValue,
    pub avatar: AttrValue,
    pub nickname: AttrValue,
    pub msg: Message,
    pub conv_type: RightContentType,
    pub del_item: Callback<AttrValue>,
    pub play_audio: Option<Callback<(AttrValue, Vec<u8>)>>,
    pub send_timeout: Callback<AttrValue>,
}

impl Component for MsgItem {
    type Message = MsgItemMsg;
    type Properties = MsgItemProps;

    fn create(ctx: &Context<Self>) -> Self {
        Self::new(ctx)
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
                        let user = db::db_ins().users.get(user_id.as_str()).await.unwrap();
                        UserWithMatchType::from(user)
                    } else {
                        match conv_type {
                            RightContentType::Friend => {
                                let friend = db::db_ins().friends.get(friend_id.as_str()).await;
                                UserWithMatchType::from(friend)
                            }
                            // query group member
                            RightContentType::Group => {
                                let member = db::db_ins()
                                    .group_members
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
                });
                false
            }
            MsgItemMsg::CallVideo => {
                self.make_call(ctx, InviteType::Video);
                false
            }
            MsgItemMsg::CallAudio => {
                self.make_call(ctx, InviteType::Audio);
                false
            }
            MsgItemMsg::QueryGroupMember(avatar, nickname) => {
                self.avatar = avatar;
                self.nickname = nickname;
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

                ctx.props().send_timeout.emit(msg_id.clone());
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
                            db::db_ins().messages.update_msg_status(&msg).await.unwrap();
                        }
                        RightContentType::Group => {
                            db::db_ins()
                                .group_msgs
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
                msg.is_resend = true;
                msg.send_time = chrono::Utc::now().timestamp_millis();
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
            MsgItemMsg::TextDoubleClick(event) => {
                event.prevent_default();
                if let Ok(Some(selection)) = window().get_selection() {
                    if selection.range_count() > 0 {
                        selection.remove_all_ranges().unwrap();
                    }
                    if let Ok(range) = document().create_range() {
                        let div = self.text_node.cast::<Node>().unwrap();
                        let _ = range.select_node_contents(&div);
                        let _ = selection.add_range(&range);
                    }
                }
                false
            }
            MsgItemMsg::PlayAudio => {
                if let Some(paly_audio) = ctx.props().play_audio.clone() {
                    let voice_id = ctx.props().msg.local_id.clone();
                    self.play_audio_animation();
                    spawn_local(async move {
                        let voice = match db::db_ins().voices.get(&voice_id).await {
                            Ok(voice) => voice,
                            Err(e) => {
                                error!("Failed to get voice: {:?}", e);
                                // todo download voice again
                                return;
                            }
                        };
                        paly_audio.emit((voice_id, voice.data));
                    });
                }
                false
            }
            MsgItemMsg::ShowAudioDownload => {
                self.download_stage = AudioDownloadStage::Downloading;
                let ctx = ctx.link().clone();
                self.show_audio_download_timer = None;
                self.audio_download_timeout = Some(Timeout::new(3000, move || {
                    ctx.send_message(MsgItemMsg::AudioDownloadTimeout);
                }));
                true
            }
            MsgItemMsg::AudioDownloadTimeout => {
                self.download_stage = AudioDownloadStage::Timeout;
                true
            }
            MsgItemMsg::OnContextMenu(event) => {
                event.prevent_default();
                self.context_menu_pos = (event.client_x(), event.client_y());
                self.show_context_menu = true;
                true
            }
            MsgItemMsg::CloseContextMenu => {
                self.show_context_menu = false;
                true
            }
            MsgItemMsg::DeleteItem => {
                let del_item = ctx.props().del_item.clone();
                let id = ctx.props().msg.id;
                let local_id = ctx.props().msg.local_id.clone();
                let content_type = ctx.props().msg.content_type;
                spawn_local(async move {
                    if content_type == ContentType::Audio {
                        // delete audio file
                        if let Err(e) = db::db_ins().voices.del(&local_id).await {
                            log::error!("delete audio file error: {:?}", e);
                            return;
                        }
                    }
                    if let Err(e) = db::db_ins().messages.delete(id).await {
                        log::error!("delete message error: {:?}", e);
                        return;
                    }
                    del_item.emit(local_id);
                });
                false
            }
            MsgItemMsg::ShowForwardMsg => {
                self.show_context_menu = false;
                self.show_friendlist = !self.show_friendlist;
                true
            }
            MsgItemMsg::ForwardMsg(list) => {
                log::info!("forward msg: {:?}", list);
                list.into_iter().for_each(|item| {
                    let mut msg = ctx.props().msg.clone();
                    msg.send_id.clone_from(&ctx.props().user_id);
                    msg.friend_id = item.into();
                    msg.server_id = AttrValue::default();
                    msg.local_id = nanoid::nanoid!().into();
                    msg.is_read = 1;
                    msg.is_self = true;
                    Dispatch::<SendMessageState>::global().reduce_mut(|s| s.msg = Msg::Single(msg));
                });
                self.show_friendlist = false;
                true
            }
            MsgItemMsg::RelatedMsg => {
                self.show_context_menu = false;
                let msg = ctx.props().msg.clone();
                let nickname = self.nickname.clone();
                RelatedMsgState::notify(nickname, msg);
                true
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        if ctx.props().msg.send_status == SendStatus::Success {
            self.show_send_fail = false;
            self.timeout = None;
            self.show_sending = false;
        }
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let id = ctx.props().msg.create_time;
        let mut classes = "msg-item";
        let mut msg_content_classes = Classes::from("msg-item-text");
        msg_content_classes.push("content-wrapper");
        if ctx.props().msg.is_self {
            msg_content_classes.push("background-self");
            classes = "msg-item-reverse";
        } else {
            msg_content_classes.push("background-other");
        }

        let oncontextmenu = ctx.link().callback(MsgItemMsg::OnContextMenu);
        let content = self.get_content(
            ctx,
            &ctx.props().msg,
            Some(oncontextmenu),
            msg_content_classes,
        );
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
                    <HangUpLoadingIcon fill={AttrValue::from("#000000")}/>
                </div>
            };
        }

        let mut friend_card = html!();
        if self.show_friend_card {
            friend_card = html! {
                <FriendCard
                    friend_info={self.friend_info.as_ref().unwrap().clone()}
                    user_id={&ctx.props().user_id}
                    avatar={&ctx.props().avatar}
                    nickname={&ctx.props().msg.nickname}
                    lang={LanguageType::ZhCN}
                    close={ctx.link().callback(|_| MsgItemMsg::CloseFriendCard)}
                    is_self={ctx.props().msg.is_self}
                    x={self.pointer.0}
                    y={self.pointer.1}
                />
            };
        }

        let avatar = if ctx.props().msg.is_self {
            html!(<img class="avatar" src={utils::get_avatar_url(&self.avatar)} />)
        } else {
            html!(<img class="avatar pointer" src={utils::get_avatar_url(&self.avatar)} onclick={_avatar_click} />)
        };

        // context menu
        let mut context_menu = html!();
        if self.show_context_menu {
            context_menu = html! {
                <MsgRightClick
                    content_type={ctx.props().msg.content_type}
                    x={self.context_menu_pos.0}
                    y={self.context_menu_pos.1}
                    close={ctx.link().callback( |_|MsgItemMsg::CloseContextMenu)}
                    delete={ctx.link().callback(|_|MsgItemMsg::DeleteItem)}
                    forward={ctx.link().callback(|_|MsgItemMsg::ShowForwardMsg)}
                    related={ctx.link().callback(|_|MsgItemMsg::RelatedMsg)}
                    />
            }
        }

        // forward msg
        let mut friendlist = html!();
        if self.show_friendlist {
            let close_back = ctx.link().callback(|_| MsgItemMsg::ShowForwardMsg);
            let submit_back = ctx.link().callback(MsgItemMsg::ForwardMsg);
            friendlist = html!(
                <SelectFriendList
                    except={&ctx.props().friend_id}
                    {close_back}
                    {submit_back}
                    lang={I18nState::get().lang} />)
        }

        // related message
        let mut related_msg = html!(<>{content}{send_status}</>);
        if let Some(ref local_id) = ctx.props().msg.related_msg_id {
            log::debug!("related msg: {:?}", ctx.props().msg.related_msg_id);
            let (position, float) = if ctx.props().msg.is_self {
                ("related-msg-right", "colunm-float-right")
            } else {
                ("related-msg-left", "colunm-float-left")
            };
            related_msg = html! {
                <div class={format!("related-msg-wrapper {float}")}>
                    <div class={format!("related-msg-content {position}")}>
                        {related_msg}
                    </div>
                    <RelatedMsg local_id={local_id.clone()} />
                </div>
            };
        }

        html! {
            <>
            {friend_card}
            {context_menu}
            {friendlist}
            <div class={classes} id={id.to_string()} >
                <div class="msg-item-avatar">
                    {avatar}
                </div>
                // {content}
                // {send_status}
                {related_msg}
            </div>
            </>
        }
    }
}
