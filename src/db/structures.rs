use bitflags::bitflags;

/*
This provides some quality of life overloaded operators which makes the development process much easier. 
More about it at: https://docs.rs/flags/latest/bitflags/ 
*/
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    pub struct Permissions: i64 {
        const SEND_MESSAGES = 1 << 0;
        const ADD_ROLE =      1 << 1;
    }
}

#[derive(Debug, scylla::SerializeValue, scylla::DeserializeValue)]
pub struct User {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password_hash: Option<String>,
    pub key: Option<String>,
    pub bio: Option<String>,

    // TODO: when bored try and fix this
    #[allow(clippy::struct_field_names)]
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
            bio: Some(String::new()),
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



#[derive(Debug,scylla::SerializeValue,serde::Serialize)]
pub struct ServerRole {
    pub server_id:String,
    pub name:String,
    pub color:String,
    pub permissions:i64,

}

#[cfg(test)]
mod tests {
    use super::Permissions;

    #[test]
    fn permissions_individual_bits() {
        assert_eq!(Permissions::SEND_MESSAGES.bits(), 1);
        assert_eq!(Permissions::ADD_ROLE.bits(), 2);
    }

    #[test]
    fn permissions_combined_with_or() {
        let combined = Permissions::SEND_MESSAGES | Permissions::ADD_ROLE;
        assert!(combined.contains(Permissions::SEND_MESSAGES));
        assert!(combined.contains(Permissions::ADD_ROLE));
        assert_eq!(combined.bits(), 3);
    }

    #[test]
    fn permissions_check_required_bits() {
        let perms = Permissions::SEND_MESSAGES | Permissions::ADD_ROLE;
        let required = Permissions::SEND_MESSAGES.bits();
        assert_eq!((perms.bits() & required) == required, true);

        let perms = Permissions::ADD_ROLE;
        let required = Permissions::SEND_MESSAGES.bits();
        assert_eq!((perms.bits() & required) == required, false);
    }

    #[test]
    fn permissions_rejects_unknown_bits() {
        let requested: i64 = 9999;
        assert_ne!((requested & !Permissions::all().bits()), 0);
    }

    #[test]
    fn permissions_serde_roundtrip() {
        let perms = Permissions::SEND_MESSAGES | Permissions::ADD_ROLE;
        let json = serde_json::to_string(&perms).expect("serialize permissions");
        let decoded: Permissions = serde_json::from_str(&json).expect("deserialize permissions");
        assert_eq!(decoded, perms);
    }
}
