#![allow(dead_code)]

use crate::metrics;

#[derive(Clone)]
pub struct AppState {
    pub metrics_collector: actix_web::web::Data<metrics::prelude::MetricsCollector>,
    pub registry: prometheus::Registry,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct TestParamsStruct {
    pub param1: String,
    pub param2: String,
}

#[derive(serde::Deserialize)]
pub struct SpellCaster {
    pub username: String,
}

#[derive(serde::Deserialize)]
pub struct SpellChecker {
    pub username: String,
    pub token: String,
    pub key: String,
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
pub struct TokenUserServer {
    pub username: String,
    pub token: String,
    pub sid: String,
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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ServerCreatedResponse {
    pub token: String,
    pub sid: String
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
pub struct DeleteMessage {
    pub datetime: String,
    pub username: String,
    pub token: String,
}

#[derive(serde::Deserialize)]
pub struct SendMessage {
    pub token: String,
    pub m_content: String,
    pub username: String,
}

#[derive(serde::Deserialize)]
pub struct UpdateUserTTL {
    pub username: String,
    pub token: String,
    pub ttl: String,  
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
    pub name: String,
    pub color: Option<String>,
    pub permissions: i64,
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
    pub target_user: String,
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

#[derive(serde::Serialize)]
pub struct GetUserPfpResp {
    pub token: String,
    pub img_url: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct SetUserPfpReq {
    pub token: String,
    pub username: String,
    pub img_url: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{
        AcceptInviteReq, FriendListReq, LimitMessageTokenUser, LoginUser, NewUser, SendInviteReq,
        SendMessage, ServerRoleRequest,
    };

    #[test]
    fn new_user_deserializes_all_fields() {
        let json = r#"{"username":"alice","email":"a@b.c","password":"secret"}"#;
        let user: NewUser = serde_json::from_str(json).expect("deserialize NewUser");
        assert_eq!(user.username, "alice");
        assert_eq!(user.email, "a@b.c");
        assert_eq!(user.password, "secret");
    }

    #[test]
    fn new_user_rejects_missing_field() {
        let json = r#"{"username":"alice","email":"a@b.c"}"#;
        assert!(serde_json::from_str::<NewUser>(json).is_err());
    }

    #[test]
    fn login_user_deserializes() {
        let json = r#"{"username":"alice","password":"secret"}"#;
        let user: LoginUser = serde_json::from_str(json).expect("deserialize LoginUser");
        assert_eq!(user.username, "alice");
        assert_eq!(user.password, "secret");
    }

    #[test]
    fn send_message_deserializes() {
        let json = r#"{"token":"tok","m_content":"hello","username":"alice"}"#;
        let msg: SendMessage = serde_json::from_str(json).expect("deserialize SendMessage");
        assert_eq!(msg.token, "tok");
        assert_eq!(msg.m_content, "hello");
        assert_eq!(msg.username, "alice");
    }

    #[test]
    fn limit_message_token_user_deserializes() {
        let json = r#"{"username":"alice","token":"tok","limit":"100","offset":"0"}"#;
        let req: LimitMessageTokenUser =
            serde_json::from_str(json).expect("deserialize LimitMessageTokenUser");
        assert_eq!(req.username, "alice");
        assert_eq!(req.token, "tok");
        assert_eq!(req.limit, "100");
        assert_eq!(req.offset, "0");
    }

    #[test]
    fn server_role_request_deserializes_with_color() {
        let json = r##"{
            "token":"tok",
            "username":"alice",
            "server_id":"sid1",
            "name":"moderator",
            "color":"#ff0000",
            "permissions":1
        }"##;
        let req: ServerRoleRequest =
            serde_json::from_str(json).expect("deserialize ServerRoleRequest");
        assert_eq!(req.color.as_deref(), Some("#ff0000"));
        assert_eq!(req.permissions, 1);
    }

    #[test]
    fn server_role_request_deserializes_without_color() {
        let json = r#"{
            "token":"tok",
            "username":"alice",
            "server_id":"sid1",
            "name":"moderator",
            "permissions":3
        }"#;
        let req: ServerRoleRequest =
            serde_json::from_str(json).expect("deserialize ServerRoleRequest without color");
        assert!(req.color.is_none());
        assert_eq!(req.permissions, 3);
    }

    #[test]
    fn send_invite_req_deserializes() {
        let json = r#"{"token":"tok","sender":"alice","recipient":"bob"}"#;
        let req: SendInviteReq = serde_json::from_str(json).expect("deserialize SendInviteReq");
        assert_eq!(req.sender, "alice");
        assert_eq!(req.recipient, "bob");
    }

    #[test]
    fn accept_invite_req_deserializes() {
        let json = r#"{"token":"tok","recipient":"bob","sender":"alice"}"#;
        let req: AcceptInviteReq =
            serde_json::from_str(json).expect("deserialize AcceptInviteReq");
        assert_eq!(req.recipient, "bob");
        assert_eq!(req.sender, "alice");
    }

    #[test]
    fn friend_list_req_deserializes() {
        let json = r#"{"token":"tok","user":"alice","friend":"bob"}"#;
        let req: FriendListReq = serde_json::from_str(json).expect("deserialize FriendListReq");
        assert_eq!(req.user, "alice");
        assert_eq!(req.friend, "bob");
    }
}
