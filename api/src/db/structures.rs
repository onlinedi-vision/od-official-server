#[derive(Debug, scylla::SerializeValue, scylla::DeserializeValue)]
pub struct User {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub key: Option<String>,
    pub bio: Option<String>,
    pub user_salt: Option<String>,
    pub password_salt: Option<String>
}

impl User {
    pub fn new(username: String, email: String, password_hash: String, key: String, salt: String) -> Self {
        Self {
            username: Some(username),
            email: Some(email),
            password_hash: Some(password_hash),
            key: Some(key),
            salt: Some(salt),
            bio: Some("".to_string()),
        }
    }
}

#[derive(Debug, scylla::SerializeValue)]
pub struct KeyUser {
    pub username: Option<String>,
    pub key: Option<String>,
}

#[derive(Debug, scylla::SerializeValue, serde::Serialize)]
pub struct Channel {
    pub channel_name: Option<String>,
}

#[derive(scylla::SerializeRow)]
pub struct UserUsername {
    pub username: Option<String>,
}

#[derive(Debug, scylla::SerializeValue, serde::Serialize)]
pub struct Message {
    pub username: Option<String>,
    pub datetime: Option<String>,
    pub m_content: Option<String>,
}

#[derive(serde::Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub desc: String,
    pub img_url: String,
}

#[derive(Debug, scylla::SerializeValue, serde::Serialize)]
pub struct ServerRole {
    pub role_name: String,
    pub server_id: String,
    pub color: Option<String>,
    pub permissions: std::collections::HashSet<String>,
}

#[derive(Debug, scylla::SerializeValue, serde::Serialize)]
pub struct UserServerRole {
    pub server_id: String,
    pub username: String,
    pub role_name: String,
}
