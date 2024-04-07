use serde::{Deserialize, Serialize};
use yew::AttrValue;

use super::{
    friend::{Friend, FriendStatus},
    user::User,
    ItemInfo, RightContentType,
};

#[derive(Deserialize, Serialize, Debug, Clone, Default, PartialEq)]
pub struct Group {
    pub id: AttrValue,
    pub owner: AttrValue,
    pub avatar: AttrValue,
    pub name: AttrValue,
    pub create_time: chrono::NaiveDateTime,
    pub update_time: chrono::NaiveDateTime,
    pub description: AttrValue,
    pub announcement: AttrValue,
    // mark this group if deleted, local only
    #[serde(default)]
    pub deleted: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct GroupRequest {
    pub id: String,
    pub owner: String,
    pub avatar: String,
    pub group_name: String,
    pub members_id: Vec<String>,
}
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct GroupDelete {
    pub user_id: String,
    pub group_id: String,
    pub is_dismiss: bool,
}

fn is_zero(id: &i32) -> bool {
    *id == 0
}

/// Group member information
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct GroupMember {
    #[serde(skip_serializing_if = "is_zero")]
    pub id: i32,
    pub user_id: AttrValue,
    #[serde(default)]
    pub group_id: AttrValue,
    pub group_name: AttrValue,
    // pub account: AttrValue,
    pub avatar: AttrValue,
    pub gender: AttrValue,
    pub region: Option<AttrValue>,
    pub joined_at: i64,
    pub signature: AttrValue,
}

impl From<Friend> for GroupMember {
    fn from(value: Friend) -> Self {
        Self {
            id: 0,
            user_id: value.friend_id,
            group_id: AttrValue::default(),
            group_name: value.name,
            avatar: value.avatar,
            region: value.region,
            gender: value.gender,
            joined_at: chrono::Local::now().timestamp_millis(),
            signature: value.signature,
        }
    }
}

impl From<User> for GroupMember {
    fn from(value: User) -> Self {
        Self {
            id: 0,
            user_id: value.id,
            group_id: AttrValue::default(),
            group_name: value.name,
            avatar: value.avatar,
            region: value.address,
            gender: value.gender,
            joined_at: chrono::Local::now().timestamp_millis(),
            signature: value.signature,
        }
    }
}

impl ItemInfo for GroupMember {
    fn name(&self) -> AttrValue {
        self.group_name.clone()
    }

    fn id(&self) -> AttrValue {
        self.user_id.clone()
    }

    fn get_type(&self) -> RightContentType {
        RightContentType::Group
    }

    fn avatar(&self) -> AttrValue {
        self.avatar.clone()
    }

    fn time(&self) -> i64 {
        self.joined_at
    }

    fn remark(&self) -> Option<AttrValue> {
        None
    }

    fn signature(&self) -> AttrValue {
        self.signature.clone()
    }

    fn region(&self) -> Option<AttrValue> {
        self.region.clone()
    }

    fn owner(&self) -> AttrValue {
        self.user_id.clone()
    }

    fn status(&self) -> FriendStatus {
        FriendStatus::Accepted
    }
}

impl ItemInfo for Group {
    fn name(&self) -> AttrValue {
        self.name.clone()
    }

    fn id(&self) -> AttrValue {
        self.id.clone()
    }

    fn get_type(&self) -> RightContentType {
        RightContentType::Group
    }

    fn avatar(&self) -> AttrValue {
        self.avatar.clone()
    }

    fn time(&self) -> i64 {
        self.create_time.timestamp_millis()
    }

    fn remark(&self) -> Option<AttrValue> {
        if self.announcement.is_empty() {
            None
        } else {
            Some(self.announcement.clone())
        }
    }

    fn signature(&self) -> AttrValue {
        self.description.clone()
    }

    fn region(&self) -> Option<AttrValue> {
        None
    }

    fn owner(&self) -> AttrValue {
        self.owner.clone()
    }

    fn status(&self) -> FriendStatus {
        if self.deleted {
            FriendStatus::Blacked
        } else {
            FriendStatus::Accepted
        }
    }
}
