use crate::db;
use crate::security;
use crate::utils::logging;

use ::function_name::named;

/// Fetches all messages in a server channel without a limit.
///
/// # Example
/// ```rs
/// fetch_server_channel_messages_unlimited(&session, "sid1".into(), "general".into()).await;
/// ```
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

    if messages.is_empty() {
        None
    } else {
        Some(messages)
    }
}

/// Fetches a limited, offset page of messages in a server channel.
///
/// # Example
/// ```rs
/// fetch_server_channel_messages_limited(&session, "sid1".into(), "general".into(), 20, 0).await;
/// ```
#[named]
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
            (sid, channel_name, i32::try_from(limit).unwrap_or(db::statics::DEFAULT_MESSAGE_LIMIT)),
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
                    match security::messages::decrypt(mc, salt) {
                        Ok(content) => {
                            messages.push(db::structures::Message {
                                username: Some(un.to_string()),
                                datetime: Some(format!("{:?}", dt.0)),
                                m_content: Some(content),
                            });
                        }
                        Err(err) => {
                            logging::log(
                                &format!("Skipping message with undecryptable content: {err}"),
                                Some(function_name!()),
                            );
                        }
                    }
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

    if messages.is_empty() {
        None
    } else {
        Some(messages)
    }
}

/// Fetches messages in a server channel, using a limited or unlimited query
/// depending on whether `limit_option`/`offset_option` are provided.
///
/// # Example
/// ```rs
/// fetch_server_channel_messages(&session, "sid1".into(), "general".into(), Some(20), Some(0)).await;
/// ```
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

/// Deletes a single message from a server channel by its timestamp.
///
/// # Example
/// ```rs
/// delete_message(&session, "sid1".into(), timestamp, "general".into()).await;
/// ```
pub async fn delete_message(
    session: &scylla::client::session::Session,
    sid: String,
    datetime: scylla::value::CqlTimestamp,
    channel_name: String,
) -> Option<Result<(), Box<dyn std::error::Error>>> {
    if session.query_unpaged(
            db::statics::DELETE_SERVER_MESSAGES_MIGRATION,
            (sid, channel_name, datetime))
        .await
        .is_ok()
    {
        Some(Ok(()))
    } else {
        None
    }
}

/// Checks whether `username` is the author of the message at `datetime`.
///
/// # Example
/// ```rs
/// verify_message_ownership(&session, "sid1".into(), "general".into(), timestamp, "alice".into()).await;
/// ```
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
