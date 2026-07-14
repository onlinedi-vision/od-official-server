use crate::db;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Stores a spell value under `key` for a user.
///
/// # Example
/// ```rs
/// spell(&session, "key1".into(), "spell1".into(), "alice".into()).await;
/// ```
pub async fn spell(
    session: &scylla::client::session::Session,
    key: String,
    spell: String,
    username: String
) -> Option<Result<()>> {

    let res = session
        .query_unpaged(db::statics::INSERT_SPELL, (key, spell, username))
        .await
        .map(|_| ())
        .map_err(From::from);

    Some(res)
    
}

/// Fetches the stored spell value for `key` and `username`.
///
/// # Example
/// ```rs
/// spell_check(&session, "key1".into(), "alice".into()).await;
/// ```
pub async fn spell_check(
    session: &scylla::client::session::Session,
    key: String,
    username: String
) -> Option<String> {
    
    let query_rows = session
        .query_unpaged(db::statics::SELECT_SPELL, (key,username))
        .await
        .ok()?
        .into_rows_result()
        .ok()?;

    if let Some(row) = (query_rows.rows::<(Option<&str>,)>().ok()?).next() {
        return match row.ok()? {
            (Some(spell),) => Some(spell.to_string()),
            _ => None
        }
    }
    None    
}

/// Deletes the stored spell value under `key`.
///
/// # Example
/// ```rs
/// spell_repel(&session, "key1".into()).await;
/// ```
pub async fn spell_repel(
    session: &scylla::client::session::Session,
    key: String
) -> Option<Result<()>> {
    Some(
        session
            .query_unpaged(db::statics::DELETE_SPELL, (key,))
            .await
            .map(|_| ())
            .map_err(From::from),
    )
}
