use crate::v1::db::{statics, structures};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Fetches basic info (name, description, image and now the frontend configuration) for a server.
///
/// # Example
/// ```rs
/// fetch_info(&session, "sid1".into()).await;
/// ```
pub async fn fetch_info(
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
        .rows::<(Option<&str>, Option<&str>, Option<&str>, Option<&str>)>()
        .ok()?).next()
    {
        let (name, desc, img_url, fe_config) = row.ok()?;
        return Some(structures::ServerInfo {
            name: name?.to_string(),
            desc: desc.unwrap_or_default().to_string(),
            img_url: img_url.unwrap_or_default().to_string(),
            fe_config: fe_config.unwrap_or_default().to_string(),
        });
    }
    None
}

/// Fetches the frontend configuration of a server.
///
/// # Example
/// ```rs
/// fetch_fe_config(&session, "sid1".into()).await;
/// ```
pub async fn fetch_fe_config(
    session: &scylla::client::session::Session,
    sid: String,
) -> Option<structures::ServerFeConfig> {
    let query_rows = session
        .query_unpaged(statics::SELECT_SERVER_FE_CONFIG, ((sid),))
        .await
        .ok()?
        .into_rows_result()
        .ok()?;
    if let Some(row) = (query_rows
        .rows::<(Option<&str>,)>()
        .ok()?).next()
    {
        let (fe_config,) = row.ok()?;
        return Some(structures::ServerFeConfig {
            fe_config: fe_config.unwrap_or_default().to_string(),
        });
    }
    None
}

/// Updates the frontend configuration of a server.
///
/// # Example
/// ```rs
/// update_ttl(&session, "alice".into(), "d".into()).await;
/// ```
pub async fn update_fe_config(
    session: &scylla::client::session::Session,
    sid: String,
    config: String,
) -> Result<()> {
    
    session
        .query_unpaged(statics::UPDATE_FE_CONFIG, (config, sid))
        .await
        .map(|_| ())
        .map_err(From::from)
    
}
