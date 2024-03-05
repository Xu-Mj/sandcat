use serde::Deserialize;
use yew::AttrValue;

use super::friend::{Friend, ItemInfo, ItemType};

#[derive(Deserialize, Debug, Clone, Default)]
pub struct Group {
    pub id: AttrValue,
    pub name: AttrValue,
    pub avatar: AttrValue,
    pub members_id: Vec<GroupMember>,
    pub create_time: chrono::NaiveDateTime,
    pub publish_msg: AttrValue,
}

/// Group member information
#[derive(Debug, Clone, Default, Deserialize)]
pub struct GroupMember {
    pub id: AttrValue,
    pub name: AttrValue,
    pub group_name: Option<AttrValue>,
    pub avatar: AttrValue,
    pub region: Option<AttrValue>,
}

impl From<Friend> for GroupMember {
    fn from(value: Friend) -> Self {
        Self {
            id: AttrValue::default(),
            name: value.name,
            group_name: None,
            avatar: value.avatar,
            region: value.address,
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
