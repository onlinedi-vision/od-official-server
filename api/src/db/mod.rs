use crate::env::get_env_var;
pub mod structures;
pub mod statics;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;


pub async fn new_scylla_session(
    uri: &str
) -> Result<scylla::client::session::Session> {
    scylla::client::session_builder::SessionBuilder::new()
        .known_node(uri)
        .user("cassandra", &get_env_var("SCYLLA_CASSANDRA_PASSWORD"))
        .build()
        .await
        .map_err(From::from)
}

pub async fn insert_new_user(
    session: &scylla::client::session::Session,
    user: structures::User
) -> Result<()> {
    session 
        .query_unpaged(statics::INSERT_NEW_USER, (user,))
        .await
        .map(|_|())
        .map_err(From::from)
} 
