#![allow(dead_code)]

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct TestParamsStruct {
    pub param1: String,
    pub param2: String,
}

#[derive(serde::Deserialize)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(serde::Deserialize)]
pub struct LoginUser {
    pub username: String,
    pub password: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PublicInfoUser {
    pub username: String,
    pub bio: String,
    pub img_url: String,
    pub roles: Vec<String>,
}

#[derive(serde::Deserialize)]
pub struct TokenLoginUser {
    pub username: String,
    pub password: String,
    pub token: String,
}

#[derive(serde::Deserialize)]
pub struct TokenUser {
    pub username: String,
    pub token: String,
}

#[derive(serde::Deserialize)]
pub struct LimitMessageTokenUser {
    pub username: String,
    pub token: String,
    pub limit: String,
    pub offset: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TokenHolder {
    pub token: String,
}

#[derive(serde::Serialize)]
pub struct Status {
    pub status: String,
}

use crate::db::structures::Channel;
#[derive(serde::Serialize)]
pub struct Channels {
    pub c_list: Vec<Channel>,
}

use crate::db::structures::Message;
#[derive(serde::Serialize)]
pub struct Messages {
    pub m_list: Vec<Message>,
}

#[derive(serde::Deserialize)]
pub struct SendMessage {
    pub token: String,
    pub m_content: String,
    pub username: String,
}

#[derive(serde::Deserialize)]
pub struct CreateChannel {
    pub token: String,
    pub channel_name: String,
    pub username: String,
}

#[derive(serde::Serialize)]
pub struct ServersList {
    pub token: String,
    pub s_list: Vec<String>,
}

#[derive(serde::Deserialize)]
pub struct CreateServer {
    pub token: String,
    pub desc: String,
    pub img_url: String,
    pub name: String,
    pub username: String,
}

#[derive(serde::Serialize)]
pub struct UsersList {
    pub u_list: Vec<PublicInfoUser>,
}

#[derive(serde::Deserialize)]
pub struct ServerRoleRequest {
    pub token: String,
    pub username: String,
    pub server_id: String,
    pub role_name: String,
    pub color: Option<String>,
    pub permissions: Option<Vec<String>>,
}

#[derive(serde::Deserialize)]
pub struct DeleteServerRoleRequest {
    pub token: String,
    pub username: String,
    pub server_id: String,
    pub role_name: String,
}

#[derive(serde::Deserialize)]
pub struct ServerRoleQuery {
    pub token: String,
    pub username: String,
    pub server_id: String,
}

#[derive(serde::Deserialize)]
pub struct UserRoleQuery {
    pub token: String,
    pub username: String,
    pub server_id: String,
}

#[derive(serde::Deserialize)]
pub struct UserServerRoleRequest {
    pub token: String,
    pub username: String,
    pub server_id: String,
    pub role_name: String,
}

#[derive(serde::Deserialize)]
pub struct SendInviteReq {
    pub token: String,
    pub sender: String,
    pub recipient: String,
}

#[derive(serde::Deserialize)]
pub struct AcceptInviteReq {
    pub token: String,
    pub recipient: String,
    pub sender: String,
}

#[derive(serde::Serialize)]
pub struct SendInviteResp {
    pub status: String,
    pub invite_id: Option<String>,
    pub u1: String,
    pub u2: String,
    pub sender: Option<String>,
}

#[derive(serde::Serialize)]
pub struct AcceptInviteResp {
    pub status: String,
    pub sid: Option<String>,
    pub invite_id: String,
    pub u1: String,
    pub u2: String,
    pub sender: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct RejectInviteReq {
    pub token: String,
    pub recipient: String,
    pub sender: String,
}

#[derive(serde::Serialize)]
pub struct RejectInviteResp {
    pub status: String,
    pub invite_id: String,
    pub u1: String,
    pub u2: String,
}

#[derive(serde::Serialize)]
pub struct PendingInvite {
    pub invite_id: String,
    pub sender: String,
}

#[derive(serde::Serialize)]
pub struct PendingInvitesResp {
    pub invites: Vec<PendingInvite>,
}

#[derive(serde::Serialize)]
pub struct FriendInfo {
    pub username: String,
    pub friends_since: String,
}

#[derive(serde::Deserialize)]
pub struct FriendListReq {
    pub token: String,
    pub user: String,
    pub friend: String,
}

#[derive(serde::Serialize)]
pub struct FriendListResp {
    pub friends: Vec<FriendInfo>,
}

#[derive(serde::Serialize)]
pub struct DeleteFriendResp {
    pub status: String,
    pub user: String,
    pub friend: String,
}
