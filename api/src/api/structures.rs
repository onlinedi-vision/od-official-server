#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct TestParamsStruct {
    pub param1: String,
    pub param2: String
}

#[derive(serde::Deserialize)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String 
}

#[derive(serde::Deserialize)]
pub struct LoginUser {
    pub username: String,
    pub password: String
}

#[derive(serde::Deserialize)]
pub struct TokenLoginUser {
    pub username: String,
    pub password: String,
    pub token: String
}

#[derive(serde::Deserialize)]
pub struct TokenUser {
    pub username: String,
    pub token: String
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TokenHolder {
    pub token: String
}

#[derive(serde::Serialize)]
pub struct Status {
    pub status: String
}


use crate::db::structures::Channel;
#[derive(serde::Serialize)]
pub struct Channels {
    pub c_list: Vec<Channel>
}

use crate::db::structures::Message;
#[derive(serde::Serialize)]
pub struct Messages {
    pub m_list: Vec<Message>
}

#[derive(serde::Deserialize)]
pub struct SendMessage {
    pub token: String,
    pub m_content: String,
    pub username: String
}

#[derive(serde::Deserialize)]
pub struct CreateChannel {
    pub token: String,
    pub channel_name: String,
    pub username: String
}

#[derive(serde::Serialize)]
pub struct ServersList {
    pub token: String,
    pub s_list: Vec<String>
}

#[derive(serde::Deserialize)]
pub struct CreateServer {
    pub token: String,
    pub desc: String,
    pub img_url: String,
    pub name: String,
    pub username: String
}

#[derive(serde::Serialize)]
pub struct UsersList {
    pub u_list: Vec<String>
}
    
#[derive(serde::Deserialize)]
pub struct File {
    pub cont: String
}

#[derive(serde::Serialize)]
pub struct FileURL {
    pub url: String
}
