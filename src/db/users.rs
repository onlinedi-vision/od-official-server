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
                return None;
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
                    return Some(insert_user_result);
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
                return Some(insert_user_result);
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
    // ) -> (Option<String>, Option<String>, Option<String>) {
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
                // return (Some(password_hash.to_string()), Some(user_salt.to_string()), Some(password_salt.to_string()));
            }
            _ => {
                println!("[get_user_password_hash] wasn't able to retrieve user info"); // TODO: FIX DEBUG LOGS FUCK ME
                return None;
            }
        };
    }
    Some(secrets)
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
