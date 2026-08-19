use crate::api;
use crate::db::{roles, statics, structures, users};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Creates a new server record.
///
/// # Example
/// ```rs
/// create_server(&session, "sid1".into(), &desc, &img_url, &name, "alice".into()).await;
/// ```
pub async fn create_server(
    session: &scylla::client::session::Session,
    sid: String,
    desc: &String,
    img_url: &String,
    name: &String,
    owner: String,
) -> Option<Result<()>> {
    Some(
        session
            .query_unpaged(
                statics::INSERT_NEW_SERVER,
                (sid, desc, img_url, name, owner),
            )
            .await
            .map(|_| ())
            .map_err(From::from),
    )
}

/// Adds a user to a server.
///
/// # Example
/// ```rs
/// add_user_to_server(&session, "sid1".into(), "alice".into()).await;
/// ```
pub async fn add_user_to_server(
    session: &scylla::client::session::Session,
    sid: String,
    owner: String,
) -> Option<Result<()>> {
    Some(
        session
            .query_unpaged(statics::INSERT_USER_INTO_SERVER, (sid, owner))
            .await
            .map(|_| ())
            .map_err(From::from),
    )
}

/// Fetches public info for all users in a server.
///
/// # Example
/// ```rs
/// fetch_server_users(&session, "sid1".into()).await;
/// ```
pub async fn fetch_server_users(
    session: &scylla::client::session::Session,
    sid: String,
) -> Option<Vec<api::structures::PublicInfoUser>> {
    let query_rows = session
        .query_unpaged(statics::SELECT_SERVER_USERS, (sid.clone(),))
        .await
        .ok()?
        .into_rows_result()
        .ok()?;
    let mut users = Vec::<api::structures::PublicInfoUser>::new();
    for row in query_rows.rows::<(Option<&str>,)>().ok()? {
        match row.ok()? {
            (Some(username),) => {
                if let Some(user_info) = users::fetch_user_info(session, username.to_string()).await
                {
                    if let Some(roles) =
                        roles::fetch_user_role_names(session, sid.clone(), username.to_string()).await
                    {
                        users.push(api::structures::PublicInfoUser {
                            username: username.to_string(),
                            bio: user_info[0].bio.clone()?.clone(),
                            img_url: user_info[0].pfp.clone()?.clone(),
                            roles,
                        });
                    } else {
                        users.push(api::structures::PublicInfoUser {
                            username: username.to_string(),
                            bio: user_info[0].bio.clone()?.clone(),
                            img_url: user_info[0].pfp.clone()?.clone(),
                            roles: Vec::new(),
                        });
                    }
                }
            }
            _ => {
                return None;
            }
        }
    }

    if users.is_empty() { None } else { Some(users) }
}

/// Fetches basic info (name, description, image) for a server.
///
/// # Example
/// ```rs
/// fetch_server_info(&session, "sid1".into()).await;
/// ```
pub async fn fetch_server_info(
    session: &scylla::client::session::Session,
    sid: String,
) -> Option<structures::ServerInfo> {
    let query_rows = session
        .query_unpaged(statics::SELECT_SERVER_INFO, ((sid),))
        .await
        .ok()?
        .into_rows_result()
        .ok()?;
    if let Some(row) = (query_rows
        .rows::<(Option<&str>, Option<&str>, Option<&str>)>()
        .ok()?).next()
    {
        match row.ok()? {
            (Some(name), Some(desc), Some(img_url)) => {
                return Some(structures::ServerInfo {
                    name: name.to_string(),
                    desc: desc.to_string(),
                    img_url: img_url.to_string(),
                });
            }
            (Some(name), Some(desc), None) => {
                return Some(structures::ServerInfo {
                    name: name.to_string(),
                    desc: desc.to_string(),
                    img_url: String::new(),
                });
            }
            (Some(name), None, None) => {
                return Some(structures::ServerInfo {
                    name: name.to_string(),
                    desc: String::new(),
                    img_url: String::new(),
                });
            }
            (Some(name), None, Some(img_url)) => {
                return Some(structures::ServerInfo {
                    name: name.to_string(),
                    desc: String::new(),
                    img_url: img_url.to_string(),
                });
            }
            _ => {
                return None;
            }
        }
    }
    None
}

/// Sends a message to a server channel, optionally with a TTL.
///
/// # Example
/// ```rs
/// send_message(&session, "sid1".into(), "general".into(), "hi".into(), "alice".into(), salt, 0).await;
/// ```
pub async fn send_message(
    session: &scylla::client::session::Session,
    sid: String,
    channel_name: String,
    m_content: String,
    username: String,
    salt: String,
    ttl: i32
) -> Result<()> {
    let mid = uuid::Uuid::new_v4().to_string();
    if ttl == 0 {
        return session
                .query_unpaged(
                    statics::INSERT_SERVER_CHANNEL_MESSAGE,
                    (mid, channel_name, m_content, sid, username, true, salt),
                )
                .await
                .map(|_| ())
                .map_err(From::from);
    }
    
    session
        .query_unpaged(
            statics::INSERT_SERVER_CHANNEL_MESSAGE_TTL,
            (mid, channel_name, m_content, sid, username, true, salt, ttl),
        )
        .await
        .map(|_| ())
        .map_err(From::from)
}

/// Creates a new channel in a server.
///
/// # Example
/// ```rs
/// create_channel(&session, "sid1".into(), "general".into()).await;
/// ```
pub async fn create_channel(
    session: &scylla::client::session::Session,
    sid: String,
    channel_name: String,
) -> Option<Result<()>> {
    Some(
        session
            .query_unpaged(statics::INSERT_SERVER_CHANNEL, (sid, channel_name))
            .await
            .map(|_| ())
            .map_err(From::from),
    )
}

/// Fetches all server ids a user belongs to.
///
/// # Example
/// ```rs
/// fetch_user_servers(&session, "alice".into()).await;
/// ```
pub async fn fetch_user_servers(
    session: &scylla::client::session::Session,
    username: String,
) -> Option<Vec<String>> {
    let query_rows = session
        .query_unpaged(statics::SELECT_USER_SID_LIST, (username,))
        .await
        .ok()?
        .into_rows_result()
        .ok()?;
    let mut sids = Vec::<String>::new();
    for row in query_rows.rows::<(Option<&str>,)>().ok()? {
        match row.ok()? {
            (Some(sid),) => {
                sids.push(sid.to_string());
            }
            _ => {
                return None;
            }
        }
    }

    if sids.is_empty() { None } else { Some(sids) }
}

/// Fetches all channels in a server.
///
/// # Example
/// ```rs
/// fetch_server_channels(&session, "sid1".into()).await;
/// ```
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

    if channels.is_empty() {
        None
    } else {
        Some(channels)
    }
}

/// Deletes a server and all of its related data (channels, users, messages, roles).
///
/// # Example
/// ```rs
/// delete_server(&session, "sid1".into()).await;
/// ```
pub async fn delete_server(
    session: &scylla::client::session::Session,
    sid: String,
) -> Option<Result<()>> {
    session
        .query_unpaged(statics::DELETE_SERVER_BY_SID, (sid.clone(),))
        .await
        .ok()?;
    session
        .query_unpaged(statics::DELETE_SERVER_CHANNELS_BY_SID, (sid.clone(),))
        .await
        .ok()?;
    session
        .query_unpaged(statics::DELETE_SERVER_USERS_BY_SID, (sid.clone(),))
        .await
        .ok()?;
    session
        .query_unpaged(
            statics::DELETE_SERVER_MESSAGES_MIGRATION_BY_SID,
            (sid.clone(),),
        )
        .await
        .ok()?;
    session
        .query_unpaged(statics::DELETE_SERVER_ROLES_BY_SID, (sid.clone(),))
        .await
        .ok()?;
    session
        .query_unpaged(statics::DELETE_USER_ROLES_BY_SID, (sid.clone(),))
        .await
        .ok()?;

    Some(Ok(()))
}

/// Checks whether `username` is the owner of server `sid`.
///
/// # Example
/// ```rs
/// check_user_is_owner(&session, "sid1".into(), "alice".into()).await;
/// ```
pub async fn check_user_is_owner(
    session: &scylla::client::session::Session,
    sid: String,
    username: String,
) -> Option<bool> {
    let query_rows = session
        .query_unpaged(statics::SELECT_SERVER_OWNER, (sid.clone(),))
        .await
        .ok()?
        .into_rows_result()
        .ok()?;

    if let Some(row) = (query_rows.rows::<(Option<&str>,)>().ok()?).next() {
        match row.ok()? {
            (Some(owner),) => {
                if owner == username {
                    return Some(true);
                }
                return Some(false);
            }
            _ => {
                return None;
            }
        }
    }
    None
}

/// Deletes a channel and its messages from a server.
///
/// # Example
/// ```rs
/// delete_channel(&session, "sid1".into(), "general".into()).await;
/// ```
pub async fn delete_channel(
    session: &scylla::client::session::Session,
    sid: String,
    channel_name: String,
) -> Option<Result<()>> {
    session
        .query_unpaged(statics::DELETE_CHANNEL, (sid.clone(), channel_name.clone()))
        .await
        .ok()?;
    session
        .query_unpaged(
            statics::DELETE_SERVER_MESSAGES_MIGRATIONS_BY_SID_AND_CHANNEL,
            (sid.clone(), channel_name.clone()),
        )
        .await
        .ok()?;

    Some(Ok(()))
}
