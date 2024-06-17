use std::rc::Rc;

use indexmap::IndexMap;
use sandcat_sdk::api;
use sandcat_sdk::model::message::SendStatus;
use sandcat_sdk::state::AudioDownloadedState;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, HtmlAudioElement, HtmlElement, Url};
use yew::prelude::*;
use yewdux::Dispatch;

use i18n::LanguageType;
use sandcat_sdk::db;
use sandcat_sdk::model::friend::FriendStatus;
use sandcat_sdk::model::message::{GroupMsg, Message, Msg, SingleCall};
use sandcat_sdk::model::{ContentType, ItemInfo, RightContentType};
use sandcat_sdk::state::{
    MobileState, RecMessageState, RefreshMsgListState, SendAudioMsgState, SendMessageState,
    SendResultState,
};

use crate::dialog::Dialog;
use crate::right::{msg_item::MsgItem, sender::Sender};

pub struct MessageList {
    list: IndexMap<AttrValue, Message>,
    node_ref: NodeRef,
    audio_ref: NodeRef,
    is_playing_audio: AttrValue,
    page: u32,
    page_size: u32,
    is_all: bool,
    scroll_state: ScrollState,
    friend: Option<Box<dyn ItemInfo>>,
    new_msg_count: u32,
    is_black: bool,
    audio_on_stop: Option<Closure<dyn FnMut(Event)>>,
    audio_data_url: Option<String>,
    is_mobile: bool,

    // listen sync offline message, query message list
    _sync_msg_dis: Dispatch<RefreshMsgListState>,
    // listen rec message, update message list
    _rec_msg_dis: Dispatch<RecMessageState>,
    _sent_msg_dis: Dispatch<SendMessageState>,
    // listen send result, update message item status
    _send_result_dis: Dispatch<SendResultState>,
    _sent_audio_dis: Dispatch<SendAudioMsgState>,
    // listen audio downloaded state when content type is audio
    _audio_dis: Dispatch<AudioDownloadedState>,
}

pub enum MessageListMsg {
    QueryMsgList(IndexMap<AttrValue, Message>),
    NextPage,
    NextPageNone,
    SendFile(Message),
    ReceiveMsg(Rc<RecMessageState>),
    SentMsg(Rc<SendMessageState>),
    SentAudio(Rc<SendAudioMsgState>),
    SendResultCallback(Rc<SendResultState>),
    SyncOfflineMsg,
    GoBottom,
    QueryFriend(Option<Box<dyn ItemInfo>>),
    PlayAudio((AttrValue, Vec<u8>)),
    AudioOnStop,
    AudioDownloaded(Rc<AudioDownloadedState>),
    DelItem(AttrValue),
    MsgSendTimeout(AttrValue),
}

/// 接收对方用户信息即可，
/// 根据对方的id查询数据库消息记录
/// 如果存在未读消息，则清空未读消息
/// 清空步骤，向服务器发送已读消息请求
/// 服务器收到请求后，更新数据库消息记录
///
/// 直接将必要|的信息都传递进来就行，省略一次数据库查询
#[derive(Properties, Clone, PartialEq, Debug)]
pub struct MessageListProps {
    pub friend_id: AttrValue,
    pub conv_type: RightContentType,
    pub cur_user_avatar: AttrValue,
    pub cur_user_id: AttrValue,
    pub nickname: AttrValue,
    pub lang: LanguageType,
}

const MSG_LIST_MAX_LEN: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollState {
    None,
    Bottom,
}

impl MessageList {
    fn query(&self, ctx: &Context<Self>) {
        let id = ctx.props().friend_id.clone();
        log::debug!("message list props id: {} in query method", id);
        if !id.is_empty() {
            // 查询数据库
            let id = id.clone();
            let page = self.page;
            let page_size = self.page_size;
            let conv_type = ctx.props().conv_type.clone();
            log::debug!("props conv type :{:?}", conv_type);

            ctx.link().send_future(async move {
                match conv_type {
                    RightContentType::Friend => {
                        let list = db::db_ins()
                            .messages
                            .get_messages(id.as_str(), page, page_size)
                            .await
                            .unwrap();
                        MessageListMsg::QueryMsgList(list)
                    }
                    RightContentType::Group => {
                        let list = db::db_ins()
                            .group_msgs
                            .get_messages(id.as_str(), page, page_size)
                            .await
                            .unwrap();
                        MessageListMsg::QueryMsgList(list)
                    }
                    _ => {
                        todo!()
                    }
                }
            });
        }
    }

    fn query_friend(&self, ctx: &Context<Self>) {
        let id = ctx.props().friend_id.clone();
        log::debug!("message list props id: {} in query method", id.clone());
        if !id.is_empty() {
            // 查询数据库
            let conv_type = ctx.props().conv_type.clone();

            ctx.link().send_future(async move {
                // 查询朋友信息
                let mut friend: Option<Box<dyn ItemInfo>> = None;
                match conv_type {
                    RightContentType::Friend => {
                        let mut result = db::db_ins().friends.get(&id).await;
                        // update friend info

                        // todo optimize query time, we should load the friend info asynchronously
                        if let Ok(user) = api::friends().query_friend(&id).await {
                            if user.id == id {
                                let mut need_update = false;
                                if user.name != result.name {
                                    result.name = user.name.into();
                                    need_update = true;
                                }
                                if user.avatar != result.avatar {
                                    result.avatar = user.avatar.into();
                                    need_update = true;
                                }
                                result.region = user.region.map(|r| r.into());
                                if user.signature != result.signature {
                                    result.signature = user.signature.into();
                                }
                                if need_update {
                                    db::db_ins().friends.put_friend(&result).await;
                                }
                            }
                        }
                        friend = Some(Box::new(result));
                    }
                    RightContentType::Group => {
                        friend = Some(Box::new(
                            db::db_ins().groups.get(&id).await.unwrap().unwrap(),
                        ));
                    }
                    _ => {}
                }
                MessageListMsg::QueryFriend(friend)
            });
        }
    }

    fn reset(&mut self) {
        self.list = IndexMap::new();
        self.page = 1;
        self.page_size = 20;
        self.new_msg_count = 0;
        self.is_all = false;
        self.scroll_state = ScrollState::None;
    }

    fn insert_msg(&mut self, msg: Message, friend_id: AttrValue) -> bool {
        if msg.friend_id != friend_id {
            return false;
        }
        let is_self = msg.is_self;
        // there is only one possible situation about we can get the msg through local_id:
        // user send msg but failed and resend again
        if let Some(item) = self.list.get_mut(&msg.local_id) {
            item.send_status = msg.send_status;
        } else {
            self.list.shift_insert(0, msg.local_id.clone(), msg);
        }

        if is_self {
            self.new_msg_count = 0;
            self.scroll_state = ScrollState::Bottom;
        } else if let Some(node) = self.node_ref.cast::<HtmlElement>() {
            // if node scroll top is less than -20, then do not scroll, 20 is redundant
            if node.scroll_top() < -20 {
                // 如果消息列表没有拉到最下面，那么加一
                self.new_msg_count += 1;
                self.scroll_state = ScrollState::None;
            } else {
                self.new_msg_count = 0;
                self.scroll_state = ScrollState::Bottom;
            }
        }

        // calculate cache size
        while self.scroll_state == ScrollState::Bottom && self.list.len() > MSG_LIST_MAX_LEN {
            // we need to delete the last 100 cache messages from the past
            self.list.pop();
        }

        true
    }
    fn handle_rec_msg(&mut self, ctx: &Context<Self>, msg: Msg, friend_id: AttrValue) -> bool {
        match msg {
            Msg::Single(msg) => self.insert_msg(msg, friend_id),
            Msg::Group(msg) => {
                match msg {
                    GroupMsg::Message(msg) => self.insert_msg(msg, friend_id),
                    // need to handle, as system notify
                    GroupMsg::MemberExit(_) => false,
                    GroupMsg::Dismiss((group_id, _)) => {
                        if group_id == friend_id {
                            self.is_black = true;
                            return true;
                        }
                        false
                    }
                    _ => false,
                }
            }

            Msg::SingleCall(m) => match m {
                SingleCall::InviteCancel(msg) => self.insert_msg(msg.into(), friend_id),
                SingleCall::InviteAnswer(msg) => {
                    if !msg.agree {
                        return self.insert_msg(msg.into(), friend_id);
                    }
                    false
                }
                SingleCall::HangUp(msg) => self.insert_msg(Message::from_hangup(msg), friend_id),
                SingleCall::NotAnswer(msg) => {
                    self.insert_msg(Message::from_not_answer(msg), friend_id)
                }
                _ => false,
            },

            Msg::SendRelationshipReq(_)
            | Msg::RecRelationship(_)
            | Msg::ReadNotice(_)
            | Msg::SingleDeliveredNotice(_)
            | Msg::OfflineSync(_)
            | Msg::RelationshipRes(_)
            | Msg::FriendshipDeliveredNotice(_) => false,
            // todo query list item , update state
            Msg::ServerRecResp(_) => false,
            Msg::RecRelationshipDel((friend_id, _)) => {
                log::debug!(
                    "rec friendship del in msg list {}, ctx friend id {:?}",
                    friend_id,
                    ctx.props().friend_id
                );
                // judge if friend_id is current user
                if friend_id == ctx.props().friend_id {
                    self.is_black = true;
                    return true;
                }
                false
            }
        }
    }

    fn play_audio(&mut self, ctx: &Context<Self>, id: AttrValue, data: Vec<u8>) {
        let audio = self.audio_ref.clone();
        // query audio data
        if let Some(audio) = audio.cast::<HtmlAudioElement>() {
            if self.is_playing_audio == id {
                let _ = audio.pause();
                audio.set_src("");
                self.is_playing_audio = AttrValue::default();
                return;
            }
            self.is_playing_audio = id;
            let u8_array = js_sys::Uint8Array::from(data.as_slice());

            let array: js_sys::Array = js_sys::Array::new_with_length(1);
            array.set(0, u8_array.buffer().into());

            let mime_type = "audio/webm; codecs=opus";
            let mut property_bag = BlobPropertyBag::new();
            property_bag.type_(mime_type);

            // todo handle error
            let blob = match Blob::new_with_u8_array_sequence_and_options(&array, &property_bag) {
                Ok(blob) => blob,
                Err(e) => {
                    log::error!("create blob error: {:?}", e);
                    return;
                }
            };
            let data_url = match Url::create_object_url_with_blob(&blob) {
                Ok(url) => url,
                Err(e) => {
                    log::error!("create data url error: {:?}", e);
                    Dialog::error("play audio error");
                    return;
                }
            };

            audio.set_src(&data_url);

            self.audio_data_url = Some(data_url);

            // set stop event
            let on_stop = self.audio_on_stop.get_or_insert_with(|| {
                let ctx = ctx.link().clone();
                Closure::wrap(Box::new(move |_: Event| {
                    ctx.send_message(MessageListMsg::AudioOnStop);
                }) as Box<dyn FnMut(Event)>)
            });

            audio.set_onended(Some(on_stop.as_ref().unchecked_ref()));
            // todo handle error
            if let Err(e) = audio.play() {
                log::error!("play audio error: {:?}", e);
                Dialog::error("play audio error");
            };
        }
    }
}

impl Component for MessageList {
    type Message = MessageListMsg;
    type Properties = MessageListProps;

    fn create(ctx: &Context<Self>) -> Self {
        let _sync_msg_dis = Dispatch::global()
            .subscribe_silent(ctx.link().callback(|_| MessageListMsg::SyncOfflineMsg));
        let _rec_msg_dis =
            Dispatch::global().subscribe_silent(ctx.link().callback(MessageListMsg::ReceiveMsg));
        let _sent_msg_dis =
            Dispatch::global().subscribe_silent(ctx.link().callback(MessageListMsg::SentMsg));
        let _send_result_dis = Dispatch::global()
            .subscribe_silent(ctx.link().callback(MessageListMsg::SendResultCallback));
        let _sent_audio_dis =
            Dispatch::global().subscribe_silent(ctx.link().callback(MessageListMsg::SentAudio));

        let audio_dis = Dispatch::global()
            .subscribe_silent(ctx.link().callback(MessageListMsg::AudioDownloaded));
        let self_ = Self {
            list: IndexMap::new(),
            is_playing_audio: AttrValue::default(),
            node_ref: NodeRef::default(),
            audio_ref: NodeRef::default(),
            page_size: 20,
            page: 1,
            is_all: false,
            scroll_state: ScrollState::Bottom,
            friend: None,
            new_msg_count: 0,
            is_black: false,
            audio_on_stop: None,
            audio_data_url: None,
            is_mobile: Dispatch::<MobileState>::global().get().is_mobile(),

            _sync_msg_dis,
            _rec_msg_dis,
            _send_result_dis,
            _sent_audio_dis,
            _sent_msg_dis,
            _audio_dis: audio_dis,
        };
        self_.query_friend(ctx);
        self_.query(ctx);
        self_
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let friend_id = ctx.props().friend_id.clone();
        match msg {
            MessageListMsg::QueryMsgList(list) => {
                log::debug!("message list update: {}", list.len());
                // 判断是否是最后一页，优化查询次数
                if list.len() < self.page_size as usize {
                    self.is_all = true;
                }
                self.list.extend(list);
                true
            }
            MessageListMsg::NextPage => {
                log::debug!("next page");
                self.page += 1;
                self.scroll_state = ScrollState::None;
                self.query(ctx);
                false
            }
            MessageListMsg::NextPageNone => false,
            MessageListMsg::SendFile(msg) => {
                self.list.insert(msg.local_id.clone(), msg);
                self.scroll_state = ScrollState::Bottom;
                true
            }
            MessageListMsg::ReceiveMsg(msg_state) => {
                log::debug!("rec message in message list....");
                let msg = msg_state.msg.clone();
                self.handle_rec_msg(ctx, msg, friend_id)
            }
            MessageListMsg::SentMsg(msg_state) => {
                if let Msg::Single(msg) = &msg_state.msg {
                    if msg.content_type == ContentType::Audio {
                        return false;
                    }
                }
                let msg = msg_state.msg.clone();
                self.handle_rec_msg(ctx, msg, friend_id)
            }
            MessageListMsg::SentAudio(state) => {
                let msg = state.msg.clone();
                self.insert_msg(msg, friend_id)
            }
            MessageListMsg::GoBottom => {
                self.new_msg_count = 0;
                self.scroll_state = ScrollState::Bottom;
                true
            }
            MessageListMsg::QueryFriend(item) => {
                if let Some(item) = item.as_ref() {
                    self.is_black = item.status() == FriendStatus::Delete;
                }
                self.friend = item;
                true
            }
            MessageListMsg::SyncOfflineMsg => {
                log::debug!("sync offline msg in message list....");
                self.reset();
                self.query(ctx);
                false
            }
            MessageListMsg::SendResultCallback(state) => {
                if let Some(v) = self.list.get_mut(&state.msg.local_id) {
                    v.send_status = state.msg.send_status.clone();
                }
                true
            }
            MessageListMsg::PlayAudio((id, data)) => {
                self.play_audio(ctx, id, data);
                false
            }
            MessageListMsg::AudioOnStop => {
                self.is_playing_audio = AttrValue::default();
                self.audio_on_stop = None;
                if let Some(ref url) = self.audio_data_url {
                    if let Err(e) = Url::revoke_object_url(url) {
                        log::error!("revoke object url error: {:?}", e);
                    };
                }
                self.audio_data_url = None;
                false
            }
            MessageListMsg::AudioDownloaded(state) => {
                if let Some(item) = self.list.get_mut(&state.local_id) {
                    item.audio_downloaded = true;
                    return true;
                }
                false
            }
            MessageListMsg::DelItem(id) => {
                self.list.shift_remove(&id);

                true
            }
            MessageListMsg::MsgSendTimeout(id) => {
                log::debug!("msg send timeout: {}", id);
                if let Some(item) = self.list.get_mut(&id) {
                    item.send_status = SendStatus::Failed;
                    return true;
                }
                false
            }
        }
    }

    fn changed(&mut self, ctx: &Context<Self>, _old_props: &Self::Properties) -> bool {
        self.reset();
        self.query_friend(ctx);
        self.query(ctx);
        // 这里不能让组件重新绘制，
        // 因为query方法会触发组件的渲染，
        // 重复渲染会导致页面出现难以预料的问题
        false
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        if self.friend.is_none() {
            return html!();
        }

        let props = ctx.props();
        let is_all = self.is_all;

        let onscroll = ctx.link().callback(move |event: Event| {
            let node: HtmlElement = event.target().unwrap().dyn_into().unwrap();
            let height = node.client_height() - node.scroll_height() + 1;
            // 判断是否滑动到顶部，
            if node.scroll_top() == height && !is_all {
                // 翻页
                return MessageListMsg::NextPage;
            }
            MessageListMsg::NextPageNone
        });

        let on_file_send = ctx.link().callback(MessageListMsg::SendFile);

        // 未读消息数量
        let new_msg_count = if self.scroll_state != ScrollState::Bottom && self.new_msg_count > 0 {
            let onclick = ctx.link().callback(|_| MessageListMsg::GoBottom);
            html! {
                <div class="msg-list-new-msg-count" {onclick} >
                    {self.new_msg_count}
                </div>
            }
        } else {
            html!()
        };

        let conv_type = &props.conv_type;
        let friend_avatar = self.friend.as_ref().unwrap().avatar();
        let list = self
            .list
            .iter()
            .map(|(_, msg)| {
                // let mut avatar = friend.avatar().clone();
                let avatar = if msg.is_self {
                    &props.cur_user_avatar
                } else {
                    &friend_avatar
                };
                let mut play_audio = None;
                if msg.content_type == ContentType::Audio {
                    play_audio = Some(ctx.link().callback(MessageListMsg::PlayAudio));
                }
                let del_item = ctx.link().callback(MessageListMsg::DelItem);

                let send_timeout = ctx.link().callback(MessageListMsg::MsgSendTimeout);
                html! {
                    <MsgItem
                        user_id={&props.cur_user_id}
                        friend_id={&props.friend_id}
                        msg={msg.clone()}
                        {avatar}
                        nickname={&msg.nickname}
                        conv_type={conv_type.clone()}
                        {play_audio}
                        {del_item}
                        {send_timeout}
                        key={msg.id}
                    />
                }
            })
            .collect::<Html>();

        let class = if self.is_mobile {
            "resize"
        } else {
            "resize resize-size"
        };

        html! {
            <>
                <div {class}>
                    <audio ref={self.audio_ref.clone()}/>
                    {new_msg_count}
                    <div ref={self.node_ref.clone()} class="msg-list"  {onscroll}>
                        {list}
                    </div>
                </div>
                <Sender
                    friend_id={&props.friend_id}
                    cur_user_id={&props.cur_user_id}
                    avatar={&ctx.props().cur_user_avatar}
                    nickname={&ctx.props().nickname}
                    conv_type={ctx.props().conv_type.clone()}
                    disable = {self.is_black}
                    {on_file_send}
                    lang={ctx.props().lang}
                />
            </>
        }
    }

    fn rendered(&mut self, _ctx: &Context<Self>, _first_render: bool) {
        if self.scroll_state == ScrollState::Bottom {
            if let Some(node) = self.node_ref.cast::<HtmlElement>() {
                node.set_scroll_top(0);
                self.scroll_state = ScrollState::Bottom;
            }
        }
    }
}
