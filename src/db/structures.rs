use bitflags::bitflags;

#[derive(Debug, scylla::SerializeValue, scylla::DeserializeValue)]
pub struct User {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub key: Option<String>,
    pub bio: Option<String>,
    pub user_salt: Option<String>,
    pub password_salt: Option<String>,
}

impl User {
    pub fn new(
        username: String,
        email: String,
        password_hash: String,
        key: String,
        user_salt: String,
        password_salt: String,
    ) -> Self {
        Self {
            username: Some(username),
            email: Some(email),
            password_hash: Some(password_hash),
            key: Some(key),
            user_salt: Some(user_salt),
            password_salt: Some(password_salt),
            bio: Some("".to_string()),
        }
    }
}

#[derive(Debug, scylla::SerializeValue, serde::Serialize)]
pub struct Spell {
    pub key: Option<String>,
    pub spell: Option<String>,
}

#[derive(Debug, scylla::SerializeValue)]
pub struct UserInfo {
    pub pfp: Option<String>,
    pub bio: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct UserPfp {
    pub pfp: Option<String>,
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

#[derive(Debug, scylla::SerializeValue, serde::Serialize)]
pub struct UserSecrets {
    pub password_hash: Option<String>,
    pub user_salt: Option<String>,
    pub password_salt: Option<String>,
}

#[derive(serde::Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub desc: String,
    pub img_url: String,
}


bitflags! {
    #[derive(Debug, Clone, Copy, serde::Serialize,serde::Deserialize)]
    pub struct Permissions: u32 {
        const Read = 0b1000;
    }
}

impl scylla::serialize::value::SerializeValue for Permissions {
    fn serialize<'b>(
        &self,
        typ: &scylla::_macro_internal::ColumnType,
        writer: scylla::_macro_internal::CellWriter<'b>,
    ) -> Result<
        scylla::_macro_internal::WrittenCellProof<'b>,
        scylla::_macro_internal::SerializationError,
    > {
        <i32 as scylla::serialize::value::SerializeValue>::serialize(
            &(self.bits() as i32),
            typ,
            writer,
        )
    }
}


#[derive(Debug, scylla::SerializeValue, serde::Serialize)]
pub struct ServerRole {
    pub role_name: String,
    pub server_id: String,
    pub color: Option<String>, //hex string rgba color
    pub permissions: Permissions,
}

