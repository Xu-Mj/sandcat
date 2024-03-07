use serde::{Deserialize, Serialize};
use yew::AttrValue;

use super::{friend::Friend, message::CreateGroup, user::UserView, ItemInfo, ItemType};

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Group {
    pub id: AttrValue,
    pub owner: AttrValue,
    pub avatar: AttrValue,
    pub group_name: AttrValue,
    pub create_time: i64,
    pub announcement: AttrValue,
    pub members_id: Vec<String>,
}

impl From<CreateGroup> for Group {
    fn from(value: CreateGroup) -> Self {
        Self {
            id: value.info.id,
            owner: value.info.owner,
            avatar: value.info.avatar,
            group_name: value.info.group_name,
            create_time: chrono::Local::now().timestamp_millis(),
            announcement: value.info.announcement,
            members_id: vec![],
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct GroupInfo {
    pub id: AttrValue,
    pub owner: AttrValue,
    pub avatar: AttrValue,
    pub group_name: AttrValue,
    pub create_time: i64,
    pub announcement: AttrValue,
}

impl From<GroupRequest> for GroupInfo {
    fn from(value: GroupRequest) -> Self {
        Self {
            id: value.id.into(),
            owner: value.owner.into(),
            avatar: value.avatar.into(),
            group_name: value.group_name.into(),
            create_time: chrono::Local::now().timestamp_millis(),
            announcement: AttrValue::default(),
        }
    }
}

impl From<Group> for GroupInfo {
    fn from(value: Group) -> Self {
        Self {
            id: value.id,
            owner: value.owner,
            avatar: value.avatar,
            group_name: value.group_name,
            create_time: value.create_time,
            announcement: value.announcement,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct GroupRequest {
    pub id: String,
    pub owner: String,
    pub avatar: String,
    pub group_name: String,
    pub members_id: Vec<String>,
}
/// Group member information
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct GroupMember {
    #[serde(default)]
    pub id: i64,
    pub user_id: AttrValue,
    #[serde(default)]
    pub group_id: AttrValue,
    pub name: AttrValue,
    pub group_name: AttrValue,
    pub account: AttrValue,
    pub avatar: AttrValue,
    pub gender: AttrValue,
    pub region: Option<AttrValue>,
}

impl GroupMember {
    pub fn from_user_view_and_group(value: UserView, group_id: impl Into<AttrValue>) -> Self {
        Self {
            id: 0,
            user_id: value.id.into(),
            gender: value.gender.into(),
            group_id: group_id.into(),
            region: None,
            avatar: value.avatar.into(),
            account: value.account.into(),
            name: value.name.clone().into(),
            group_name: value.name.into(),
        }
    }
}

impl From<Friend> for GroupMember {
    fn from(value: Friend) -> Self {
        Self {
            id: 0,
            user_id: value.friend_id,
            group_id: AttrValue::default(),
            name: value.name.clone(),
            group_name: value.name,
            avatar: value.avatar,
            region: value.address,
            gender: value.gender,
            account: value.account,
        }
    }
}

impl ItemInfo for Group {
    fn name(&self) -> AttrValue {
        self.group_name.clone()
    }

    fn id(&self) -> AttrValue {
        self.id.clone()
    }

    fn get_type(&self) -> ItemType {
        ItemType::Group
    }

    fn avatar(&self) -> AttrValue {
        self.avatar.clone()
    }
}
