use crate::db;

pub async fn fetch_server_channel_messages_unlimited(
    session: &scylla::client::session::Session,
    sid: String,
    channel_name: String,   
) -> Option<Vec<db::structures::Message>> {
    
    let query_rows = session
        .query_unpaged(db::statics::SELECT_SERVER_CHANNEL_MESSAGES, (sid, channel_name))
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

    if messages.len() > 0 {
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
    offset: usize
) -> Option<Vec<db::structures::Message>> {
    let query_rows = session
        .query_unpaged(db::statics::SELECT_SERVER_CHANNEL_MESSAGES_MIGRATION, (sid, channel_name, limit as i32))
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
        )>()
        .ok()?
        .enumerate()
    {
        if idx >= offset {
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
    }

    if messages.len() > 0 {
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
    offset_option: Option<usize>
) -> Option<Vec<db::structures::Message>> {

    if let Some(limit) = limit_option {
        if let Some(offset) = offset_option {
            return fetch_server_channel_messages_limited(session, sid.clone(), channel_name.clone(), limit, offset).await;
        }
    }
    return fetch_server_channel_messages_unlimited(session, sid.clone(), channel_name.clone()).await;

}
