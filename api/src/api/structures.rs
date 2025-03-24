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

#[derive(serde::Serialize, serde::Deserialize)]
pub struct TokenHolder {
    pub token: String
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
