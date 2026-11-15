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

/// Consumes the stored spell for `key` and `username` at most once.
/// Returns the value only after its conditional deletion is confirmed.
///
/// # Example
/// ```rs
/// spell_check(&session, "key1".into(), "alice".into()).await;
/// ```
pub async fn spell_check(
    session: &scylla::client::session::Session,
    key: String,
    username: String,
) -> Option<String> {
    let query_rows = session
        .query_unpaged(db::statics::SELECT_SPELL, (&key, &username))
        .await
        .ok()?
        .into_rows_result()
        .ok()?;

    let (spell,) = query_rows.first_row::<(Option<&str>,)>().ok()?;
    let spell = spell?;

    let claim_rows = session
        .query_unpaged(db::statics::DELETE_SPELL, (key, spell, username))
        .await
        .ok()?
        .into_rows_result()
        .ok()?;

    // This query returns [applied] and the previous username.
    let (applied, _previous_username) = claim_rows.single_row::<(bool, Option<&str>)>().ok()?;

    if applied {
        Some(spell.to_string())
    } else {
        None
    }
}
