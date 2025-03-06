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
        .query_unpaged(statics::INSERT_NEW_USER, (user.username, user.password_hash, user.email, user.key, user.bio))
        .await
        .map(|_|())
        .map_err(From::from)
}

pub async fn update_user_key(
    session: &scylla::client::session::Session,
    keyuser: structures::KeyUser
) -> Result<()> {
    session
        .query_unpaged(statics::UPDATE_USER_KEY, (keyuser.key, keyuser.username))
        .await
        .map(|_|())
        .map_err(From::from)
}

pub async fn get_user_password_hash(
    session: &scylla::client::session::Session,
    user: structures::UserUsername
) -> Option<String> {
    todo!("Handle User Inexistent");
    let query_rows = session
        .query_unpaged(statics::SELECT_USER_PASSWORD_HASH, ((user.username),))
        .await.ok()?
        .into_rows_result().ok()?;
    for row in query_rows.rows::<(Option<&str>,)>().ok()?{
        let (password_hash_str,): (Option<&str>,) = row.ok()?;
        match password_hash_str {
            Some(str) => {return Some(str.to_string());}
            None => {return None;}
        }
    }
    None
}


