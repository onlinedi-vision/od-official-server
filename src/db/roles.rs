use crate::db::{statics, structures};
use crate::utils::logging;

use ::function_name::named;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub async fn insert_server_role(
    session: &scylla::client::session::Session,
    server_id: String,
    role: structures::ServerRole,
) -> Option<Result<()>> {
    let res: std::result::Result<scylla::response::query_result::QueryResult, _> = session
        .query_unpaged(
            statics::INSERT_SERVER_ROLE,
            (server_id,role.id,role.name,role.color,role.permissions),
        ).await;
        Some(res.map(|_| ()).map_err(From::from))
       
}


pub async fn fetch_user_roles(
    session: &scylla::client::session::Session,
    server_id: String,
    username: String,
) -> Option<Result<Vec<String>>>{

    let res = session
    .query_unpaged(statics::SELECT_USER_ROLES, (server_id,username))
    .await;

    let rows = res.expect("REASON").into_rows_result();

    let mut roles = Vec::new();

    for row in res.rows {
        let (role,): (String,) = row.get("role_name");
        roles.push(role);
    }

    Some(Ok(roles))

}
