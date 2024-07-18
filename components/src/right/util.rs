use log::error;
use wasm_bindgen_futures::spawn_local;
use yewdux::Dispatch;

use sandcat_sdk::{
    api, db,
    model::friend::Friend,
    state::{ItemType, UpdateFriendState},
};

pub fn update_friend_remark(user_id: String, friend: Friend) {
    spawn_local(async move {
        let remark = friend.remark.as_ref().unwrap();
        if api::friends()
            .update_remark(
                user_id,
                friend.friend_id.clone().to_string(),
                remark.to_string(),
            )
            .await
            .is_ok()
        {
            if let Err(err) = db::db_ins().friends.put_friend(&friend).await {
                error!("save friend error:{:?}", err);
                return;
            }
            Dispatch::<UpdateFriendState>::global().reduce_mut(|s| {
                s.id = friend.friend_id;
                s.name = None;
                s.remark.clone_from(&friend.remark);
                s.type_ = ItemType::Friend;
            });
        }
    });
}
