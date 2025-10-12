use crate::db::{statics, structures, users, roles};
use crate::api;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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
                if let Some(user_info) = users::fetch_user_info(session, username.to_string()).await {
                    if let Some(roles) = roles::fetch_user_roles(session, sid.clone(), username.to_string()).await { 
                        users.push(
                            api::structures::PublicInfoUser {
                                username: username.to_string(),
                                bio: user_info[0].bio.clone()?.to_string(),
                                img_url: user_info[0].pfp.clone()?.to_string(),
                                roles: roles,
                            }
                        );
                    }
                }
            }
            _ => {
                return None;
            }
        }
    }

    if users.len() > 0 {
        Some(users)
    } else {
        None
    }
}

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
    for row in query_rows
        .rows::<(Option<&str>, Option<&str>, Option<&str>)>()
        .ok()?
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
                    img_url: "".to_string(),
                });
            }
            (Some(name), None, None) => {
                return Some(structures::ServerInfo {
                    name: name.to_string(),
                    desc: "".to_string(),
                    img_url: "".to_string(),
                });
            }
            (Some(name), None, Some(img_url)) => {
                return Some(structures::ServerInfo {
                    name: name.to_string(),
                    desc: "".to_string(),
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

pub async fn send_message(
    session: &scylla::client::session::Session,
    sid: String,
    channel_name: String,
    m_content: String,
    username: String,
) -> Option<Result<()>> {
    let mid = uuid::Uuid::new_v4().to_string();
    Some(
        session
            .query_unpaged(
                statics::INSERT_SERVER_CHANNEL_MESSAGE,
                (mid, channel_name, m_content, sid, username),
            )
            .await
            .map(|_| ())
            .map_err(From::from),
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
            .map(|_| ())
            .map_err(From::from),
    )
}

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

    if sids.len() > 0 {
        Some(sids)
    } else {
        None
    }
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

    if channels.len() > 0 {
        Some(channels)
    } else {
        None
    }
}
