use crate::db;
use crate::security;

pub async fn fetch_server_channel_messages_unlimited(
    session: &scylla::client::session::Session,
    sid: String,
    channel_name: String,
) -> Option<Vec<db::structures::Message>> {
    let query_rows = session
        .query_unpaged(
            db::statics::SELECT_SERVER_CHANNEL_MESSAGES,
            (sid, channel_name),
        )
        .await
        .ok()?
        .into_rows_result()
        .ok()?;
    let mut messages = Vec::<db::structures::Message>::new();
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
                messages.push(db::structures::Message {
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

    if !messages.is_empty() {
        Some(messages)
    } else {
        None
    }
}

pub async fn fetch_server_channel_messages_limited(
    session: &scylla::client::session::Session,
    sid: String,
    channel_name: String,
    limit: usize,
    offset: usize,
) -> Option<Vec<db::structures::Message>> {
    let query_rows = session
        .query_unpaged(
            db::statics::SELECT_SERVER_CHANNEL_MESSAGES_MIGRATION,
            (sid, channel_name, limit as i32),
        )
        .await
        .ok()?
        .into_rows_result()
        .ok()?;
    let mut messages = Vec::<db::structures::Message>::new();
    for (idx, row) in query_rows
        .rows::<(
            Option<&str>,
            Option<scylla::value::CqlTimestamp>,
            Option<&str>,
            Option<bool>,
            Option<&str>,
        )>()
        .ok()?
        .enumerate()
    {
        if idx >= offset {
            match row.ok()? {
                (Some(un), Some(dt), Some(mc), Some(_), Some(salt)) => {
                    messages.push(db::structures::Message {
                        username: Some(un.to_string()),
                        datetime: Some(format!("{:?}", dt.0)),
                        m_content: Some(security::messages::decrypt(mc, salt)),
                    });
                }
                (Some(un), Some(dt), Some(mc), None, _) => {
                    messages.push(db::structures::Message {
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
    }

    if !messages.is_empty() {
        Some(messages)
    } else {
        None
    }
}

pub async fn fetch_server_channel_messages(
    session: &scylla::client::session::Session,
    sid: String,
    channel_name: String,
    limit_option: Option<usize>,
    offset_option: Option<usize>,
) -> Option<Vec<db::structures::Message>> {
    if let Some(limit) = limit_option
        && let Some(offset) = offset_option {
            return fetch_server_channel_messages_limited(
                session,
                sid.clone(),
                channel_name.clone(),
                limit,
                offset,
            )
            .await;
        }
    return fetch_server_channel_messages_unlimited(session, sid.clone(), channel_name.clone())
        .await;
}

pub async fn delete_message(
    session: &scylla::client::session::Session,
    sid: String,
    datetime: scylla::value::CqlTimestamp,
    channel_name: String,
) -> Option<Result<(), Box<dyn std::error::Error>>> {
    session
        .query_unpaged(
            db::statics::DELETE_SERVER_MESSAGES_MIGRATION,
            (sid, channel_name, datetime),
        )
        .await
        .ok()?;

    Some(Ok(()))
}

pub async fn verify_message_ownership(
    session: &scylla::client::session::Session,
    sid: String,
    channel_name: String,
    datetime: scylla::value::CqlTimestamp,
    username: String,
) -> Option<bool> {
    let query_rows = session
        .query_unpaged(
            db::statics::SELECT_SERVER_MESSAGE_MIGRATIONS_OWNER,
            (sid, channel_name, datetime),
        )
        .await
        .ok()?
        .into_rows_result()
        .ok()?;

    if let Some(row) = (query_rows.rows::<(Option<&str>,)>().ok()?).next() {
        match row.ok()? {
            (Some(un),) => {
                if un == username {
                    return Some(true);
                } else {
                    return Some(false);
                }
            }
            _ => {
                return None;
            }
        }
    }

    None
}
