use serde::{Deserialize, Serialize};
use yew::AttrValue;

use super::{friend::Friend, ItemInfo, ItemType};

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
    #[serde(skip)]
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
/// Group member information
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq)]
pub struct GroupMember {
    #[serde(default)]
    pub id: i64,
    pub user_id: AttrValue,
    #[serde(default)]
    pub group_id: AttrValue,
    pub group_name: AttrValue,
    // pub account: AttrValue,
    pub avatar: AttrValue,
    pub gender: AttrValue,
    pub region: Option<AttrValue>,
    pub joined_at: chrono::NaiveDateTime,
}

impl From<Friend> for GroupMember {
    fn from(value: Friend) -> Self {
        Self {
            id: 0,
            user_id: value.friend_id,
            group_id: AttrValue::default(),
            group_name: value.name,
            avatar: value.avatar,
            region: value.address,
            gender: value.gender,
            joined_at: chrono::Local::now().naive_local(),
        }
    }
}

impl ItemInfo for Group {
    fn name(&self) -> AttrValue {
        self.name.clone()
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
