use crate::db::statics;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub async fn send_dm_invite(
    session: &scylla::client::session::Session,
    u1: String,
    u2: String,
    invite_id: String,
    sender: String,
) -> Option<Result<()>> {
    Some(
        session
            .query_unpaged(statics::INSERT_DM_INVITE, (u1, u2, invite_id, sender))
            .await
            .map(|_| ())
            .map_err(From::from),
    )
}

pub async fn fetch_dm_invite(
    session: &scylla::client::session::Session,
    u1: String,
    u2: String,
) -> Option<(String, String)> {
    let query_rows = session
        .query_unpaged(statics::SELECT_DM_INVITE, (u1, u2))
        .await
        .ok()?
        .into_rows_result()
        .ok()?;
    for row in (query_rows.rows::<(Option<&str>, Option<&str>)>()).ok()? {
        if let Ok((Some(invite_id), Some(sender))) = row {
            return Some((invite_id.to_string(), sender.to_string()));
        }
    }
    None
}

pub async fn delete_dm_invite(
    session: &scylla::client::session::Session,
    u1: String,
    u2: String,
) -> Option<Result<()>> {
    Some(
        session
            .query_unpaged(statics::DELETE_DM_INVITE, (u1, u2))
            .await
            .map(|_| ())
            .map_err(From::from),
    )
}

pub async fn fetch_pending_dm_invites(
    session: &scylla::client::session::Session,
    username: String,
) -> Option<Vec<(String, String)>> {
    let mut invites = Vec::<(String, String)>::new();

    for query in [
        statics::SELECT_PENDING_INVITES_BY_U1,
        statics::SELECT_PENDING_INVITES_BY_U2,
    ] {
        let query_rows = session
            .query_unpaged(query, (username.clone(),))
            .await
            .ok()?
            .into_rows_result()
            .ok()?;

        for row in (query_rows.rows::<(Option<&str>, Option<&str>)>()).ok()? {
            if let Ok((Some(invite_id), Some(sender))) = row {
                if sender != username {
                    invites.push((invite_id.to_string(), sender.to_string()));
                }
            }
        }
    }
    Some(invites)
}
