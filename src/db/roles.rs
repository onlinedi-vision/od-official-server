use crate::db::{statics, structures};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;



pub async fn insert_server_role(
    session: &scylla::client::session::Session,
    server_id: String,
    role: structures::ServerRole,
) -> Result<()> {
    let res: std::result::Result<scylla::response::query_result::QueryResult, _> = session
        .query_unpaged(
            statics::INSERT_SERVER_ROLE,
            (server_id, role.name, role.color, role.permissions),
        )
        .await;
    res.map(|_| ()).map_err(From::from)
}

pub async fn assign_role(
session: &scylla::client::session::Session,
server_id:String,
target_user:String,
role_name:String,
) -> Result<()>{

    let res: std::result::Result<scylla::response::query_result::QueryResult, _> = session
    .query_unpaged(
        statics::ASSIGN_ROLE_TO_USER,
        (server_id,target_user,role_name),

    ).await;
    res.map(|_| ()).map_err(From::from)
}

pub async fn delete_role(
    session: &scylla::client::session::Session,
    server_id:String,
    role_name:String,
) -> Result<()>{
    let res: std::result::Result<scylla::response::query_result::QueryResult, _ > = session.query_unpaged(
        statics::DELETE_SERVER_ROLE,
        (server_id,role_name),
    ).await;
    res.map(|_| ()).map_err(From::from)
}

pub async fn remove_role(
    session: &scylla::client::session::Session,
    server_id:String,
    target_user:String,
    role_name:String,
) -> Result<()>{
    let res: std::result::Result<scylla::response::query_result::QueryResult, _>= session.query_unpaged(
        statics::REMOVE_ROLE_FROM_USER,
        (server_id,target_user,role_name),
    ).await;
    res.map(|_| ()).map_err(From::from)
}

/// Returns the list of role names a user has in a server (for display in `PublicInfoUser`).
pub async fn fetch_user_role_names(
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

    let mut names = Vec::new();
    for row in query_rows.rows::<(Option<&str>,)>().ok()? {
        if let (Some(name),) = row.ok()? {
            names.push(name.to_string());
        }
    }
    if names.is_empty() { None } else { Some(names) }
}

/// Returns the combined permission bits (OR of all role permissions) for a user in a server.
pub async fn fetch_user_permissions(
    session: &scylla::client::session::Session,
    server_id: String,
    username: String,
) -> i64 {

    let Some(role_names) = fetch_user_role_names(session, server_id.clone(), username).await else {
        return 0;
    };

    let mut combined: i64 = 0;
    for role_name in role_names {
        if let Ok(result) = session
            .query_unpaged(statics::SELECT_ROLE_PERMISSIONS, (server_id.clone(), role_name))
            .await
        && let Ok(rows) = result.into_rows_result() {
            for row in rows.rows::<(Option<i64>,)>().into_iter().flatten() {
                if let Ok((Some(perms),)) = row {
                    combined |= perms;
                }
            }
        }
    }
    combined
}

/// Remove all user-role assignments for a deleted role in a server.
pub async fn remove_role_from_all_users(
    session: &scylla::client::session::Session,
    server_id: String,
    role_name: String,
) -> Result<()> {
    let query_rows = session
        .query_unpaged(statics::SELECT_USER_ROLES_BY_ROLE, (server_id.clone(), role_name.clone()))
        .await
        .map_err(|e| -> Box<dyn std::error::Error> { From::from(e) })?
        .into_rows_result()
        .map_err(|e| -> Box<dyn std::error::Error> { From::from(e) })?;

    for row in query_rows.rows::<(Option<&str>,)>().into_iter().flatten() {
        if let Ok((Some(username),)) = row {
            let _ = session
                .query_unpaged(
                    statics::REMOVE_ROLE_FROM_USER,
                    (server_id.clone(), username.to_string(), role_name.clone()),
                )
                .await;
        }
    }
    Ok(())
}
