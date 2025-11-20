use crate::db::{statics, structures};

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
                println!("[get_user_password_hash] wasn't able to retrieve user info"); // TODO: FIX DEBUG LOGS FUCK ME
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

pub async fn delete_token(
    session: &scylla::client::session::Session,
    username: String,
    token: String,
) -> Option<Result<()>> {
    println!("DELETE: {} {}", username, token);
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
