use gloo_net::http::Request;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

use crate::api::friend::FriendApi;
use crate::model::friend::FriendshipWithUser4Response;
use crate::pb::message::User;
use crate::{
    model::friend::{Friend, FriendShipAgree, FriendShipRequest, FriendShipWithUser},
    pb::message::UpdateRemarkRequest,
};

use super::RespStatus;

pub struct FriendHttp {
    token: String,
    auth_header: String,
}

impl FriendHttp {
    pub fn new(token: String, auth_header: String) -> Self {
        Self { token, auth_header }
    }
}

#[async_trait::async_trait(?Send)]
impl FriendApi for FriendHttp {
    // 请求添加好友
    async fn apply_friend(
        &self,
        new_friend: FriendShipRequest,
    ) -> Result<FriendShipWithUser, JsValue> {
        let friendship: FriendshipWithUser4Response = Request::post("/api/friend")
            .header(&self.auth_header, &self.token)
            .json(&new_friend)
            .map_err(|err| JsValue::from(err.to_string()))?
            .send()
            .await
            .map_err(|err| JsValue::from(err.to_string()))?
            .success()?
            .json()
            .await
            .map_err(|err| JsValue::from(err.to_string()))?;
        Ok(FriendShipWithUser::from(friendship))
    }

    // 同意好友请求
    async fn agree_friend(&self, friendship: FriendShipAgree) -> Result<Friend, JsValue> {
        let friend: Friend = Request::put("/api/friend/agree")
            .header(&self.auth_header, &self.token)
            .json(&friendship)
            .map_err(|err| JsValue::from(err.to_string()))?
            .send()
            .await
            .map_err(|err| {
                log::debug!("error: {:?}", &err);
                JsValue::from(err.to_string())
            })?
            .success()?
            .json()
            .await
            .map_err(|err| {
                log::debug!("error: {:?}", &err);
                JsValue::from(err.to_string())
            })?;
        Ok(friend)
    }

    async fn query_friend(&self, friend_id: &str) -> Result<User, JsValue> {
        let user: User = Request::get(format!("/api/friend/query/{}", friend_id).as_str())
            .header(&self.auth_header, &self.token)
            .send()
            .await
            .map_err(|err| JsValue::from(err.to_string()))?
            .success()?
            .json()
            .await
            .map_err(|err| JsValue::from(err.to_string()))?;
        Ok(user)
    }

    // 获取好友列表, 服务端需要增加好友表及其逻辑，包括好友请求表，实际好友关系表（因为需要额外字段：备注，添加时间等）
    async fn get_friend_list_by_id(&self, id: String) -> Result<Vec<Friend>, JsValue> {
        let friends: Vec<Friend> = Request::get(format!("/api/friend/{}", id).as_str())
            .header(&self.auth_header, &self.token)
            .send()
            .await
            .map_err(|err| JsValue::from(err.to_string()))?
            .success()?
            .json()
            .await
            .map_err(|err| JsValue::from(err.to_string()))?;
        Ok(friends)
    }

    async fn update_remark(
        &self,
        user_id: String,
        friend_id: String,
        remark: String,
    ) -> Result<(), JsValue> {
        let data = UpdateRemarkRequest {
            user_id,
            friend_id,
            remark,
        };
        Request::put("/api/friend/remark")
            .header(&self.auth_header, &self.token)
            .json(&data)
            .map_err(|err| JsValue::from(err.to_string()))?
            .send()
            .await
            .map_err(|err| {
                log::debug!("error: {:?}", &err);
                JsValue::from(err.to_string())
            })?
            .success()?;
        Ok(())
    }

    async fn delete_friend(&self, user_id: String, friend_id: String) -> Result<(), JsValue> {
        Request::delete("/api/friend")
            .header(&self.auth_header, &self.token)
            .json(&DeleteFriend { user_id, friend_id })
            .map_err(|err| JsValue::from(err.to_string()))?
            .send()
            .await
            .map_err(|err| JsValue::from(err.to_string()))?
            .success()?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct DeleteFriend {
    user_id: String,
    friend_id: String,
}
