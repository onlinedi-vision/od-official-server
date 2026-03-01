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
) -> Option<Result<()>>{
    let res: std::result::Result<scylla::response::query_result::QueryResult, _ > = session
    .query_unpaged(
     statics::GET_USER_ROLES,
     (server_id,username),   
    ).await;
}
