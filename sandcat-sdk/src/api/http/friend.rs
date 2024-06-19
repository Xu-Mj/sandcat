use gloo_net::http::Request;
use serde::{Deserialize, Serialize};

use crate::api::friend::FriendApi;
use crate::error::Result;
use crate::model::friend::FriendshipWithUser4Response;
use crate::pb::message::FriendInfo;
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
    async fn apply_friend(&self, new_friend: FriendShipRequest) -> Result<FriendShipWithUser> {
        let friendship: FriendshipWithUser4Response = Request::post("/api/friend")
            .header(&self.auth_header, &self.token)
            .json(&new_friend)?
            .send()
            .await?
            .success()?
            .json()
            .await?;
        Ok(FriendShipWithUser::from(friendship))
    }

    async fn query_friend(&self, friend_id: &str) -> Result<FriendInfo> {
        let user: FriendInfo = Request::get(format!("/api/friend/query/{}", friend_id).as_str())
            .header(&self.auth_header, &self.token)
            .send()
            .await?
            .success()?
            .json()
            .await?;
        Ok(user)
    }

    // 同意好友请求
    async fn agree_friend(&self, friendship: FriendShipAgree) -> Result<Friend> {
        let friend: Friend = Request::put("/api/friend/agree")
            .header(&self.auth_header, &self.token)
            .json(&friendship)?
            .send()
            .await?
            .success()?
            .json()
            .await?;
        Ok(friend)
    }

    // 获取好友列表, 服务端需要增加好友表及其逻辑，包括好友请求表，实际好友关系表（因为需要额外字段：备注，添加时间等）
    async fn get_friend_list_by_id(&self, id: String) -> Result<Vec<Friend>> {
        let friends: Vec<Friend> = Request::get(format!("/api/friend/{}", id).as_str())
            .header(&self.auth_header, &self.token)
            .send()
            .await?
            .success()?
            .json()
            .await?;
        Ok(friends)
    }

    async fn update_remark(
        &self,
        user_id: String,
        friend_id: String,
        remark: String,
    ) -> Result<()> {
        let data = UpdateRemarkRequest {
            user_id,
            friend_id,
            remark,
        };
        Request::put("/api/friend/remark")
            .header(&self.auth_header, &self.token)
            .json(&data)?
            .send()
            .await?
            .success()?;
        Ok(())
    }

    async fn delete_friend(&self, user_id: String, friend_id: String) -> Result<()> {
        Request::delete("/api/friend")
            .header(&self.auth_header, &self.token)
            .json(&DeleteFriend { user_id, friend_id })?
            .send()
            .await?
            .success()?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct DeleteFriend {
    user_id: String,
    friend_id: String,
}
