use crate::db::{statics, structures};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub async fn insert_server_role(
    session: &scylla::client::session::Session,
    server_id: String,
    role: structures::ServerRole,
) -> Option<Result<()>> {
    Some(
        session
            .query_unpaged(
                statics::INSERT_SERVER_ROLE,
                (server_id, role.role_name, role.color, role.permissions),
            )
            .await
            .map(|_| ())
            .map_err(From::from),
    )
}

pub async fn delete_server_role(
    session: &scylla::client::session::Session,
    server_id: String,
    role_name: String,
) -> Option<Result<()>> {
    Some(
        session
            .query_unpaged(statics::DELETE_SERVER_ROLE, (server_id, role_name))
            .await
            .map(|_| ())
            .map_err(From::from),
    )
}

pub async fn remove_role_from_all_users(
    session: &scylla::client::session::Session,
    server_id: String,
    role_name: String,
) -> Option<Result<()>> {
    let query_rows = session
        .query_unpaged(
            statics::SELECT_USERS_BY_ROLE,
            (server_id.clone(), role_name.clone()),
        )
        .await
        .ok()?
        .into_rows_result()
        .ok()?;

    let mut usernames = Vec::new();
    for row in query_rows.rows::<(Option<String>,)>().ok()? {
        if let Ok((Some(username),)) = row {
            usernames.push(username);
        }
    }

    println!(
        "Found {} users with role '{}' to remove.",
        usernames.len(),
        role_name
    );

    for username in usernames {
        let user_role = structures::UserServerRole {
            server_id: server_id.clone(),
            username: username.clone(),
            role_name: role_name.clone(),
        };

        if let Some(result) = remove_role_from_user(session, user_role).await {
            match result {
                Ok(_) => {
                    println!(
                        "Successfully removed role '{}' from user '{}'",
                        role_name, username
                    );
                }
                Err(err) => {
                    println!(
                        "Error removig role '{}' from user '{}': {:?}",
                        role_name, username, err
                    );
                }
            }
        } else {
            println!(
                "Failed to remove role '{}' from user '{}'",
                role_name, username
            );
        }
    }

    Some(Ok(()))
}

pub async fn assign_role_to_user(
    session: &scylla::client::session::Session,
    user_role: structures::UserServerRole,
) -> Option<Result<()>> {
    Some(
        session
            .query_unpaged(
                statics::ASSIGN_ROLE_TO_USER,
                (user_role.server_id, user_role.username, user_role.role_name),
            )
            .await
            .map(|_| ())
            .map_err(From::from),
    )
}

pub async fn remove_role_from_user(
    session: &scylla::client::session::Session,
    user_role: structures::UserServerRole,
) -> Option<Result<()>> {
    Some(
        session
            .query_unpaged(
                statics::REMOVE_ROLE_FROM_USER,
                (user_role.server_id, user_role.username, user_role.role_name),
            )
            .await
            .map(|_| ())
            .map_err(From::from),
    )
}

pub async fn fetch_server_roles(
    session: &scylla::client::session::Session,
    server_id: &str,
) -> Option<Vec<structures::ServerRole>> {
    let query_rows = session
        .query_unpaged(statics::SELECT_SERVER_ROLES, (server_id,))
        .await
        .ok()?
        .into_rows_result()
        .ok()?;

    let mut roles = Vec::new();
    for row in query_rows
        .rows::<(
            Option<String>,
            Option<String>,
            Option<std::collections::HashSet<String>>,
        )>()
        .ok()?
    {
        if let Ok((Some(role_name), color, permissions)) = row {
            roles.push(structures::ServerRole {
                role_name,
                server_id: server_id.to_string(),
                color,
                permissions: permissions.unwrap_or_default(),
            });
        }
    }
    if roles.len() > 0 {
        Some(roles)
    } else {
        None
    }
}

pub async fn fetch_user_roles(
    session: &scylla::client::session::Session,
    server_id: String,
    username: String,
) -> Option<Vec<String>> {
    let query_rows = session
        .query_unpaged(statics::SELECT_USER_ROLES, (server_id, username))
        .await
        .ok()?
        .into_rows_result()
        .ok()?;

    let mut role_names = Vec::new();
    for row in query_rows.rows::<(Option<String>,)>().ok()? {
        if let Ok((Some(role_name),)) = row {
            role_names.push(role_name);
        }
    }
    if role_names.len() > 0 {
        Some(role_names)
    } else {
        None
    }
}

pub async fn check_role_exists(
    session: &scylla::client::session::Session,
    server_id: &str,
    role_name: &str,
) -> Option<bool> {
    let query_rows = session
        .query_unpaged(statics::SELECT_SERVER_ROLE_BY_NAME, (server_id, role_name))
        .await
        .ok()?
        .into_rows_result()
        .ok()?;

    Some(
        !query_rows
            .rows::<(Option<String>,)>()
            .ok()?
            .next()
            .is_none(),
    )
}
