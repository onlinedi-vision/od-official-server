pub mod friends;
pub mod invites;
pub mod prelude;
pub mod roles;
pub mod server;
pub mod statics;
pub mod structures;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub async fn insert_new_user(
    session: &scylla::client::session::Session,
    user: structures::User,
) -> Option<Result<()>> {
    let query_rows = session
        .query_unpaged(statics::SELECT_USER_USERNAME, (user.username.clone(),))
        .await
        .ok()?
        .into_rows_result()
        .ok()?;
    match query_rows.rows::<(Option<&str>,)>() {
        Ok(row) => {
            if row.rows_remaining() > 0 {
                return None;
            } else {
                return Some(
                    session
                        .query_unpaged(
                            statics::INSERT_NEW_USER,
                            (
                                user.username,
                                user.password_hash,
                                user.email,
                                user.key,
                                user.bio,
                                user.user_salt,
                                user.password_salt,
                            ),
                        )
                        .await
                        .map(|_| ())
                        .map_err(From::from),
                );
            }
        }
        _ => {
            return Some(
                session
                    .query_unpaged(
                        statics::INSERT_NEW_USER,
                        (
                            user.username,
                            user.password_hash,
                            user.email,
                            user.key,
                            user.bio,
                        ),
                    )
                    .await
                    .map(|_| ())
                    .map_err(From::from),
            );
        }
    }
}

pub async fn get_user_password_hash(
    session: &scylla::client::session::Session,
    user: structures::UserUsername,
) -> Option<Vec<structures::UserSecrets>> {
    // ) -> (Option<String>, Option<String>, Option<String>) {
    let query_rows = session
        .query_unpaged(statics::SELECT_USER_PASSWORD_HASH, ((user.username),))
        .await
        .ok()?
        .into_rows_result()
        .ok()?;
    let mut secrets = Vec::<structures::UserSecrets>::new();
    for row in query_rows
        .rows::<(Option<&str>, Option<&str>, Option<&str>)>()
        .ok()?
    {
        match row.ok()? {
            (Some(password_hash), Some(user_salt), Some(password_salt)) => {
                secrets.push(structures::UserSecrets {
                    password_hash: Some(password_hash.to_string()),
                    user_salt: Some(user_salt.to_string()),
                    password_salt: Some(password_salt.to_string()),
                });
                // return (Some(password_hash.to_string()), Some(user_salt.to_string()), Some(password_salt.to_string()));
            }
            _ => {
                println!("[get_user_password_hash] wasn't able to retrieve user info"); // TODO: FIX DEBUG LOGS FUCK ME
                return None;
            }
        };
    }
    Some(secrets)
}

pub async fn fetch_server_channels(
    session: &scylla::client::session::Session,
    sid: String,
) -> Option<Vec<structures::Channel>> {
    let query_rows = session
        .query_unpaged(statics::SELECT_SERVER_CHANNELS, ((sid),))
        .await
        .ok()?
        .into_rows_result()
        .ok()?;
    let mut channels = Vec::<structures::Channel>::new();
    for row in query_rows.rows::<(Option<&str>,)>().ok()? {
        let (channel_name,): (Option<&str>,) = row.ok()?;
        match channel_name {
            Some(str) => {
                channels.push(structures::Channel {
                    channel_name: Some(str.to_string()),
                });
            }
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
    channel_name: String,
) -> Option<Vec<structures::Message>> {
    let query_rows = session
        .query_unpaged(statics::SELECT_SERVER_CHANNEL_MESSAGES, (sid, channel_name))
        .await
        .ok()?
        .into_rows_result()
        .ok()?;
    let mut messages = Vec::<structures::Message>::new();
    for row in query_rows
        .rows::<(
            Option<&str>,
            Option<scylla::value::CqlTimestamp>,
            Option<&str>,
        )>()
        .ok()?
    {
        match row.ok()? {
            (Some(un), Some(dt), Some(mc)) => {
                messages.push(structures::Message {
                    username: Some(un.to_string()),
                    datetime: Some(format!("{:?}", dt.0)),
                    m_content: Some(mc.to_string()),
                });
            }
            _ => {
                return None;
            }
        }
    }
    Some(messages)
}
