use crate::db;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub async fn spell(
    session: &scylla::client::session::Session,
    key: String,
    spell: String,
    username: String
) -> Option<Result<()>> {

    let res = session
        .query_unpaged(db::statics::INSERT_FRIEND, (key, spell, username))
        .await
        .map(|_| ())
        .map_err(From::from);

    Some(res)
    
}

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

    for row in query_rows.rows::<(Option<&str>,)>().ok()? {
        return match row.ok()? {
            (Some(spell),) => Some(spell.to_string()),
            _ => None
        }
    }
    None    
}

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
