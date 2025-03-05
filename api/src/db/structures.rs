
#[derive(Debug, scylla::SerializeValue, scylla::DeserializeValue)]
pub struct User {
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub key: String,
    pub bio: String
}


