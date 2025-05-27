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
) -> Option<Result<()>> {
    let query_rows = session.query_unpaged(statics::SELECT_USER_USERNAME, (user.username.clone(),))
        .await.ok()?
        .into_rows_result().ok()?;
    match query_rows.rows::<(Option<&str>,)>() {
        Ok(row) => {
            if row.rows_remaining() > 0 { return None; }
            else {
                return Some(session
                    .query_unpaged(statics::INSERT_NEW_USER, (user.username, user.password_hash, user.email, user.key, user.bio))
                    .await
                    .map(|_|())
                    .map_err(From::from)); 
            }
        },
        _ => {
            return Some(session
                .query_unpaged(statics::INSERT_NEW_USER, (user.username, user.password_hash, user.email, user.key, user.bio))
                .await
                .map(|_|())
                .map_err(From::from));
        }
    }
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
    let query_rows = session
        .query_unpaged(statics::SELECT_USER_PASSWORD_HASH, ((user.username),))
        .await.ok()?
        .into_rows_result().ok()?;
    for row in query_rows.rows::<(Option<&str>,)>().ok()?{
        let (password_hash_str,): (Option<&str>,) = row.ok()?;
        match password_hash_str {
            Some(str) => {return Some(str.to_string());},
            _ => {println!("?");return None;}
        };
    }
    None
}

pub async fn check_token(
    session: &scylla::client::session::Session,
    token: String,
    un: Option<String>
) -> Option<()> {
    let query_rows: scylla::response::query_result::QueryRowsResult;
    if let Some(username) = un {
        query_rows = session
            .query_unpaged(statics::CHECK_TOKEN_USER, (token.clone(),username))
            .await.ok()?
            .into_rows_result().ok()?;
    } else {
        query_rows = session
            .query_unpaged(statics::CHECK_TOKEN, (token.clone(),))
            .await.ok()?
            .into_rows_result().ok()?;
    }
    println!(" db/check_token {:?}", token);
    match query_rows.rows::<(Option<&str>,)>() {
        Ok(row) => {
            if row.rows_remaining() > 0 {
                return Some(());
            } else {
                return None;
            }
        },
        _ => None
    }
}

pub async fn fetch_server_channels(
    session: &scylla::client::session::Session,
    sid: String
) -> Option<Vec<structures::Channel>> {
    let query_rows = session
        .query_unpaged(statics::SELECT_SERVER_CHANNELS, ((sid),))
        .await.ok()?
        .into_rows_result().ok()?;
    let mut channels = Vec::<structures::Channel>::new();
    for row in query_rows.rows::<(Option<&str>,)>().ok()? {
        let (channel_name,): (Option<&str>,) = row.ok()?;
        match channel_name {
            Some(str) => {
                channels.push(structures::Channel{channel_name: Some(str.to_string())});
            },
            None => {
                return None;
            }
        }
    }
    Some(channels)
}

pub async fn fetch_server_channel_messages(
    session: &scylla::client::session::Session,
    sid: String,
    channel_name: String
) -> Option<Vec<structures::Message>> {
    let query_rows = session
        .query_unpaged(statics::SELECT_SERVER_CHANNEL_MESSAGES, (sid,channel_name))
        .await.ok()?
        .into_rows_result().ok()?;
    let mut messages = Vec::<structures::Message>::new();
    for row in query_rows.rows::<(Option<&str>, Option<scylla::value::CqlTimestamp>,Option<&str>)>().ok()? {
        match row.ok()? {
            (Some(un), Some(dt), Some(mc)) => {
                messages.push(
                    structures::Message{
                        username: Some(un.to_string()),
                        datetime: Some(format!("{:?}", dt.0)),
                        m_content: Some(mc.to_string())
                    }
                );
            },
            _ => {
                return None;
            }
        }
    }
    Some(messages)
}

pub async fn send_message(
    session: &scylla::client::session::Session,
    sid: String,
    channel_name: String,
    m_content: String,
    username: String
) -> Option<Result<()>> {
    let mid = uuid::Uuid::new_v4().to_string();
    Some(
         session 
            .query_unpaged(statics::INSERT_SERVER_CHANNEL_MESSAGE, (mid, channel_name, m_content,sid,username ))
            .await
            .map(|_|())
            .map_err(From::from)
    )
}

pub async fn create_channel(
    session: &scylla::client::session::Session,
    sid: String,
    channel_name: String,
) -> Option<Result<()>> {
    Some(
         session 
            .query_unpaged(statics::INSERT_SERVER_CHANNEL, (sid, channel_name))
            .await
            .map(|_|())
            .map_err(From::from)
    )
}

pub async fn check_user_is_in_server(
    session: &scylla::client::session::Session,
    sid: String,
    token: String,
    un: String
) -> Option<Vec<structures::UserUsername>> {
    
    if let Some(_) = check_token(&session, token.clone(), Some(un.clone())).await {
        let query_rows = session
            .query_unpaged(statics::SELECT_SERVER_USER, (sid,un.clone()))
            .await.ok()?
            .into_rows_result().ok()?;
        let mut ret_vec = Vec::new();
        for row in query_rows.rows::<(Option<&str>,)>().ok()? {
            match row.ok()? {
                (Some(user),) => {
                    ret_vec.push(
                        structures::UserUsername {
                            username: Some(user.to_string())
                        }
                    );
                },
                _ => {
                    return None;
                }
            }
        }
        Some(ret_vec)
    } else {
        None
    }
}
