use impls::indexed_db::offline_time::OfflineTimeRepo;
pub use impls::indexed_db::*;
use offline_time::OfflineTimes;
use once_cell::sync::OnceCell;

use self::{
    conv::ConvRepo,
    conversations::Conversations,
    friend::FriendRepo,
    friend_ship::FriendShipRepo,
    friends::Friends,
    friendships::Friendships,
    group::GroupRepo,
    group_members::GroupMembers,
    group_msg::GroupMessages,
    groups::GroupInterface,
    impls::indexed_db::{
        group_members::GroupMembersRepo, group_msg::GroupMsgRepo, seq::SeqRepo, voice::VoiceRepo,
    },
    message::MessageRepo,
    messages::Messages,
    seq::SeqInterface,
    user::UserRepo,
    users::Users,
    voice::Voices,
};

pub mod conversations;
pub mod friends;
pub mod friendships;
pub mod group_members;
pub mod group_msg;
pub mod groups;
pub mod impls;
pub mod messages;
pub mod offline_time;
pub mod seq;
pub mod users;
pub mod voice;

static DB_INSTANCE: OnceCell<Db> = OnceCell::new();

pub fn db_ins() -> &'static Db {
    DB_INSTANCE.get().unwrap()
}

pub async fn init_db() {
    if DB_INSTANCE.get().is_some() {
        return;
    }
    let db = Db::new().await;
    if let Err(err) = DB_INSTANCE.set(db) {
        log::error!("{:?}", err);
    }
}

unsafe impl Sync for Db {}
unsafe impl Send for Db {}
#[derive(Debug)]
pub struct Db {
    pub convs: Box<dyn Conversations>,
    pub groups: Box<dyn GroupInterface>,
    pub friends: Box<dyn Friends>,
    pub friendships: Box<dyn Friendships>,
    pub group_members: Box<dyn GroupMembers>,
    pub messages: Box<dyn Messages>,
    pub group_msgs: Box<dyn GroupMessages>,
    pub users: Box<dyn Users>,
    pub seq: Box<dyn SeqInterface>,
    pub voices: Box<dyn Voices>,
    pub offline_time: Box<dyn OfflineTimes>,
}

impl Db {
    pub async fn new() -> Self {
        let repo = repository::Repository::new().await;
        Self {
            convs: Box::new(ConvRepo::new(repo.clone())),
            groups: Box::new(GroupRepo::new(repo.clone())),
            friends: Box::new(FriendRepo::new(repo.clone())),
            friendships: Box::new(FriendShipRepo::new(repo.clone())),
            group_members: Box::new(GroupMembersRepo::new(repo.clone())),
            messages: Box::new(MessageRepo::new(repo.clone())),
            group_msgs: Box::new(GroupMsgRepo::new(repo.clone())),
            users: Box::new(UserRepo::new(repo.clone())),
            seq: Box::new(SeqRepo::new(repo.clone())),
            voices: Box::new(VoiceRepo::new(repo.clone())),
            offline_time: Box::new(OfflineTimeRepo::new(repo)),
        }
    }
}
