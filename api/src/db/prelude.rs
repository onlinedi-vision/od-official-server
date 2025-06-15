use crate::db;
use crate::db:: {   
    statics,
    structures
};
use crate::env::get_env_var;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub async fn new_scylla_session(
    uri: &str
) -> Result<scylla::client::session::Session> {
    scylla::client::session_builder::SessionBuilder::new()
        .known_node(uri)
        .user("cassandra", &get_env_var("SCYLLA_CASSANDRA_PASSWORD"))
        .build()
        .await
        .map_err(From::from)
}

pub async fn update_user_key(
    session: &scylla::client::session::Session,
    keyuser: structures::KeyUser
) -> Result<()> {
    session
        .query_unpaged(statics::UPDATE_USER_KEY, (keyuser.key, keyuser.username))
        .await
        .map(|_|())
        .map_err(From::from)
}

pub async fn check_token(
    session: &scylla::client::session::Session,
    token: String,
    un: Option<String>
) -> Option<()> {
    let query_rows: scylla::response::query_result::QueryRowsResult;
    if let Some(username) = un {
        query_rows = session
            .query_unpaged(statics::CHECK_TOKEN_USER, (token.clone(),username))
            .await.ok()?
            .into_rows_result().ok()?;
    } else {
        query_rows = session
            .query_unpaged(statics::CHECK_TOKEN, (token.clone(),))
            .await.ok()?
            .into_rows_result().ok()?;
    }
    println!(" db/check_token {:?}", token);
    match query_rows.rows::<(Option<&str>,)>() {
        Ok(row) => {
            if row.rows_remaining() > 0 {
                return Some(());
            } else {
                return None;
            }
        },
        _ => None
    }
}

pub async fn check_user_is_in_server(
    session: &scylla::client::session::Session,
    sid: String,
    token: String,
    un: String
) -> Option<Vec<structures::UserUsername>> {
    
    if let Some(_) = db::prelude::check_token(&session, token.clone(), Some(un.clone())).await {
        let query_rows = session
            .query_unpaged(statics::SELECT_SERVER_USER, (sid,un.clone()))
            .await.ok()?
            .into_rows_result().ok()?;
        let mut ret_vec = Vec::new();
        for row in query_rows.rows::<(Option<&str>,)>().ok()? {
            match row.ok()? {
                (Some(user),) => {
                    println!("SERVER");
                    ret_vec.push(
                        structures::UserUsername {
                            username: Some(user.to_string())
                        }
                    );
                },
                _ => {
                    println!("NOT SERVER");
                    return None;
                }
            }
        }
        Some(ret_vec)
    } else {
        None
    }
}
