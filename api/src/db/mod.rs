
mod structures;
mod statics;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;


pub async fn new_scylla_session(
    uri: &str
) -> Result<scylla::Session> {
    scylla::SessionBuilder::new()
        .known_node(uri)
        .user("cassandra", &get_env_var("SCYLLA_CASSANDRA_PASSWORD"))
        .build()
        .await
        .map_err(From::from)
}

pub async fn insert_new_user(
    session: &scylla::Session,
    user: structures::User
) -> Result<()> {
    session 
        .query(statics::INSERT_NEW_USER, user)
        .await
        .map(|_|())
        .map_err(From::from)
} 
