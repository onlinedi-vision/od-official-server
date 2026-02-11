use crate::db::{statics, structures};
use crate::utils::logging;

use ::function_name::named;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub async fn insert_new_user(
    session: &scylla::client::session::Session,
    user: structures::User,
) -> Option<Result<()>> {
    let query_rows = session
        .query_unpaged(statics::SELECT_USER_USERNAME, (user.username.clone(),))
        .await
        .ok()?
        .into_rows_result()
        .ok()?;
    match query_rows.rows::<(Option<&str>,)>() {
        Ok(row) => {
            if row.rows_remaining() > 0 {
                None
            } else {
                let insert_user_result = session
                    .query_unpaged(
                        statics::INSERT_NEW_USER,
                        (
                            user.username.clone(),
                            user.password_hash,
                            user.email,
                            user.key.clone(),
                            user.bio,
                            user.user_salt,
                            user.password_salt,
                        ),
                    )
                    .await
                    .map(|_| ())
                    .map_err(From::from);
                if insert_user_result.is_err() {
                    Some(insert_user_result)
                } else {
                    return Some(
                        session
                            .query_unpaged(statics::INSERT_NEW_TOKEN, (user.username, user.key))
                            .await
                            .map(|_| ())
                            .map_err(From::from),
                    );
                }
            }
        }
        _ => {
            let insert_user_result = session
                .query_unpaged(
                    statics::INSERT_NEW_USER,
                    (
                        user.username.clone(),
                        user.password_hash,
                        user.email,
                        user.key.clone(),
                        user.bio,
                        user.user_salt,
                        user.password_salt,
                    ),
                )
                .await
                .map(|_| ())
                .map_err(From::from);
            if insert_user_result.is_err() {
                Some(insert_user_result)
            } else {
                return Some(
                    session
                        .query_unpaged(statics::INSERT_NEW_TOKEN, (user.username, user.key))
                        .await
                        .map(|_| ())
                        .map_err(From::from),
                );
            }
        }
    }
}

#[named]
pub async fn get_user_password_hash(
    session: &scylla::client::session::Session,
    user: structures::UserUsername,
) -> Option<Vec<structures::UserSecrets>> {
    let query_rows = session
        .query_unpaged(statics::SELECT_USER_PASSWORD_HASH, ((user.username),))
        .await
        .ok()?
        .into_rows_result()
        .ok()?;
    let mut secrets = Vec::<structures::UserSecrets>::new();
    for row in query_rows
        .rows::<(Option<&str>, Option<&str>, Option<&str>)>()
        .ok()?
    {
        match row.ok()? {
            (Some(password_hash), Some(user_salt), Some(password_salt)) => {
                secrets.push(structures::UserSecrets {
                    password_hash: Some(password_hash.to_string()),
                    user_salt: Some(user_salt.to_string()),
                    password_salt: Some(password_salt.to_string()),
                });
            }
            _ => {
                logging::log("wasn't able to retrieve user info", Some(function_name!()));
                return None;
            }
        };
    }
    if !secrets.is_empty() {
        Some(secrets)
    } else {
        None
    }
}

#[named]
pub async fn delete_token(
    session: &scylla::client::session::Session,
    username: String,
    token: String,
) -> Option<Result<()>> {
    logging::log(&format!("DELETE: {} {}", username, token), Some(function_name!()));
    Some(
        session
            .query_unpaged(statics::DELETE_TOKEN, (username, token))
            .await
            .map(|_| ())
            .map_err(From::from),
    )
}

pub async fn fetch_user_info(
    session: &scylla::client::session::Session,
    username: String,
) -> Option<Vec<structures::UserInfo>> {
    let query_rows = session
        .query_unpaged(statics::SELECT_USER_INFO, ((username),))
        .await
        .ok()?
        .into_rows_result()
        .ok()?;
    let mut user_info = Vec::<structures::UserInfo>::new();
    for row in query_rows.rows::<(Option<&str>, Option<&str>)>().ok()? {
        match row.ok()? {
            (Some(pfp), Some(bio)) => {
                user_info.push(structures::UserInfo {
                    pfp: Some(pfp.to_string()),
                    bio: Some(bio.to_string()),
                });
            }
            _ => {
                return None;
            }
        };
    }
    if !user_info.is_empty() {
        Some(user_info)
    } else {
        None
    }
}

pub async fn fetch_user_pfp(
    session: &scylla::client::session::Session,
    username: &str,
) -> Option<structures::UserPfp> {
    let query_rows = session
        .query_unpaged(statics::SELECT_USER_PFP, (username,))
        .await
        .ok()?
        .into_rows_result()
        .ok()?;
    if let Some(row) = (query_rows.rows::<(Option<&str>,)>().ok()?).next() {
        match row.ok()? {
            (Some(pfp),) => {
                return Some(structures::UserPfp {
                    pfp: Some(pfp.to_string()),
                });
            }
            (None,) => {
                return Some(structures::UserPfp { pfp: None });
            }
        }
    }
    None
}

pub async fn set_user_pfp(
    session: &scylla::client::session::Session,
    username: &str,
    img_url: Option<&str>,
) -> Option<Result<()>> {
    Some(
        session
            .query_unpaged(statics::UPDATE_USER_PFP, (img_url, username))
            .await
            .map(|_| ())
            .map_err(From::from),
    )
}

pub async fn update_ttl(
    session: &scylla::client::session::Session,
    username: String,
    ttl: String,
) -> Result<()> {
    
    session
        .query_unpaged(statics::UPDATE_USER_TTL, (ttl, username))
        .await
        .map(|_| ())
        .map_err(From::from)
    
}

pub async fn get_ttl(
    session: &scylla::client::session::Session,
    username: String
) -> i32 {
    match get_ttl_symbol(&session, username).await.unwrap_or("N".to_string()).as_str() {
        "s"     =>          3 as i32, // every 3 _s_econd   (only for test cases)
        "S"     =>         10 as i32, // every 10 _S_econds (only for test cases)
        "h"     =>      60*60 as i32, // every _h_our
        "H"     =>   12*60*60 as i32, // every 12 _H_ours
        "d"     =>   24*60*60 as i32, // every _d_ay
        "w"     => 7*24*60*60 as i32, // every _w_eek
        "N" | _ =>          0 as i32, // _N_ever
    }
}

pub async fn get_ttl_symbol(
    session: &scylla::client::session::Session,
    username: String
) -> Option<String> {
    if let Ok(query_result) = session
        .query_unpaged(statics::SELECT_USER_TTL, (username.clone(),))
        .await
    {
        if let Ok(query_rows) = query_result.into_rows_result()
        {
            if let Ok(rows) = query_rows.rows::<(Option<&str>,)>() {
                for row in rows {
                    if let Ok(row_unwrapped) = row {
                        match row_unwrapped {
                            (Some(str_ttl),) => {
                                return Some(str_ttl.to_string());
                            },
                            _ => {
                                return None;
                            }
                        }
                    }
                }
            }
        }
    }
    return None;
}
