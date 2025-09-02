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
    Some(roles)
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
    Some(role_names)
}
