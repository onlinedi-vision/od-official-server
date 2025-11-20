use moka;

use crate::db;
use crate::db::{statics, structures};
use crate::env::get_env_var;
use crate::security;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub async fn insert_user_token(
    session: &scylla::client::session::Session,
    cache: &moka::future::Cache<String,String>,
    user: structures::KeyUser,
) -> Option<Result<()>> {

    if let Some(username) = user.username.clone()
        && let Some(key) = user.key.clone() {
            let _ = cache.insert(username.clone(), key.clone()).await;
        }
       
    Some(
        session
            .query_unpaged(statics::INSERT_NEW_TOKEN, (user.username, user.key))
            .await
            .map(|_| ())
            .map_err(From::from),
    )
}

pub async fn new_scylla_session(uri: &str) -> Result<scylla::client::session::Session> {
    scylla::client::session_builder::SessionBuilder::new()
        .known_node(uri)
        .user("cassandra", get_env_var("SCYLLA_CASSANDRA_PASSWORD"))
        .build()
        .await
        .map_err(From::from)
}

pub async fn new_moka_cache(cache_size: u64) -> Result<moka::future::Cache<String, String>> {
    Ok(moka::future::Cache::<String,String>::new(cache_size))
}

pub async fn check_token(
    session: &scylla::client::session::Session,
    cache: &moka::future::Cache<String, String>,
    token: String,
    un: Option<String>,
) -> Option<()> {
    let query_rows: scylla::response::query_result::QueryRowsResult;
    let plain_token = token.clone();
    let crypted_token = security::armor_token(plain_token);

    

    if let Some(username) = un.clone() {
        println!("{}", crypted_token.clone());

        if let Some(cache_token) = cache.get(&username.clone()).await
            && cache_token == crypted_token.clone() {
                return Some(());
            }

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
    println!(" db/check_token {:?} {:?}", token, un);
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

pub async fn check_user_is_in_server(
    session: &scylla::client::session::Session,
    cache: &moka::future::Cache<String,String>,
    sid: String,
    token: String,
    un: String,
) -> Option<Vec<structures::UserUsername>> {
    if (db::prelude::check_token(session, cache, token.clone(), Some(un.clone())).await).is_some() {
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
                    println!("SERVER");
                    ret_vec.push(structures::UserUsername {
                        username: Some(user.to_string()),
                    });
                }
                _ => {
                    println!("NOT SERVER");
                    return None;
                }
            }
        }
        Some(ret_vec)
    } else {
        println!("????????? TOKEN");
        None
    }
}
