use moka;

use crate::db;
use crate::db::{statics, structures};
use crate::env::get_env_var;
use crate::security;
use crate::utils::logging;
use crate::metrics;

use ::function_name::named;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub async fn insert_user_token(
    session: &scylla::client::session::Session,
    cache: &moka::future::Cache<String,String>,
    user: structures::KeyUser,
) -> Option<Result<()>> {

    if let Some(username) = user.username.clone()
        && let Some(key) = user.key.clone() {
            let () = cache.insert(username.clone(), key.clone()).await;
        }
       
    Some(
        session
            .query_unpaged(statics::INSERT_NEW_TOKEN, (user.username, user.key, *statics::TOKEN_TTL))
            .await
            .map(|_| ())
            .map_err(From::from),
    )
}

pub async fn new_scylla_session(uri: &str) -> Result<scylla::client::session::Session> {
    scylla::client::session_builder::SessionBuilder::new()
        .known_node(uri)
        .user(get_env_var("SCYLLA_DB_USER"), get_env_var("SCYLLA_CASSANDRA_PASSWORD"))
        .build()
        .await
        .map_err(From::from)
}

pub fn new_moka_cache(cache_size: u64) -> moka::future::Cache<String, String> {
    moka::future::Cache
        ::<String,String>
        ::builder()
        .max_capacity(cache_size)
        .time_to_live(
            std::time::Duration::from_secs(
                u64::try_from(*db::statics::TOKEN_TTL)
                    .unwrap_or(db::statics::DEFAULT_TOKEN_TTL)
            )
        )
        .build()
}

#[named]
pub async fn check_token(
    session: &scylla::client::session::Session,
    cache: &moka::future::Cache<String, String>,
    token: String,
    un: Option<String>,
    collector: &metrics::prelude::MetricsCollector
) -> Option<()> {
    if token.len() < 16 {
        return None;
    }

    let query_rows: scylla::response::query_result::QueryRowsResult;
    let crypted_token = security::armor_token(&token);

    if let Some(username) = un.clone() {
        if let Some(cache_token) = cache.get(&username.clone()).await
        && cache_token == crypted_token.clone() {
            
            collector.total_cache_hit_count
                .inc();

            logging::log("Cache hit...", Some(function_name!()));
            return Some(());
        }
        
        collector.total_cache_miss_count
            .inc();

        logging::log("Cache miss...", Some(function_name!()));

        query_rows = session
            .query_unpaged(statics::CHECK_TOKEN_USER, (crypted_token.clone(), username))
            .await
            .ok()?
            .into_rows_result()
            .ok()?;
    } else {
        query_rows = session
            .query_unpaged(statics::CHECK_TOKEN, (crypted_token.clone(),))
            .await
            .ok()?
            .into_rows_result()
            .ok()?;
    }

    match query_rows.rows::<(Option<&str>,)>() {
        Ok(row) => {
            if row.rows_remaining() > 0 {
                Some(())
            } else {
                None
            }
        }
        _ => None,
    }
}

pub async fn check_sid(
    session: &scylla::client::session::Session,
    sid: String
) -> bool {
    if let Ok(query_result) = session
        .query_unpaged(statics::SELECT_SERVER_SID, (sid.clone(),))
        .await
    && let Ok(query_rows) = query_result.into_rows_result()
    && let Ok(rows) = query_rows.rows::<(Option<&str>,)>() 
    && let Some(row_ok) = rows.flatten().next() {

        match row_ok {
            (Some(db_sid),) => {
                return db_sid == sid;
            }
            _ => {
                return false;
            }
        }

    }
    false
}

pub async fn check_channel_name(
    session: &scylla::client::session::Session,
    sid: String,
    channel_name: String
) -> bool {
    if let Ok(query_result) = session
        .query_unpaged(statics::SELECT_SERVER_CHANNEL, (sid.clone(), channel_name.clone()))
        .await
    && let Ok(query_rows) = query_result.into_rows_result()
    && let Ok(rows) = query_rows.rows::<(Option<&str>,)>() 
    && let Some(row_ok) = rows.flatten().next() {

        match row_ok {
            (Some(db_channel_name),) => {
                return db_channel_name == channel_name;
            }
            _ => {
                return false;
            }
        }

    }
    false
}

pub async fn check_user_is_in_server(
    session: &scylla::client::session::Session,
    cache: &moka::future::Cache<String,String>,
    sid: String,
    token: String,
    un: String,
    collector: &metrics::prelude::MetricsCollector
) -> Option<Vec<structures::UserUsername>> {
    if db::prelude::check_token(session, cache, token.clone(), Some(un.clone()), collector).await.is_some() {
        let query_rows = session
            .query_unpaged(statics::SELECT_SERVER_USER, (sid, un.clone()))
            .await
            .ok()?
            .into_rows_result()
            .ok()?;
        let mut ret_vec = Vec::new();
        for row in query_rows.rows::<(Option<&str>,)>().ok()? {
            match row.ok()? {
                (Some(user),) => {
                    ret_vec.push(structures::UserUsername {
                        username: Some(user.to_string()),
                    });
                }
                _ => {
                    return None;
                }
            }
        }
        if !ret_vec.is_empty() {
            return Some(ret_vec);
        }
    } 
    None
}
