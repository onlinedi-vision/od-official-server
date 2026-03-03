use crate::api::structures;
use crate::db;
use crate::security;
use crate::utils::logging;

use ::function_name::named;

#[named]
#[actix_web::post("/api/add_server_role")]
pub async fn add_server_role(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::ServerRoleRequest>,
) -> impl actix_web::Responder {
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);

    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await
    .is_none()
    {
        return actix_web::HttpResponse::Unauthorized().body("Invalid token");
    }
    
    let role = db::structures::ServerRole {
        name: req.name.clone(),
        server_id: req.server_id.clone(),
        color: req.color.clone().unwrap_or_default(),
        permissions: req.permissions.clone(),
    };

    if let Some(result) =
        db::roles::insert_server_role(&scylla_session, req.server_id.clone(), role).await
        && result.is_ok() {
            return actix_web::HttpResponse::Ok().body("Role added successfully");
        }
    logging::log(&format!("Error inserting role: {:?}", req.name.clone()), Some(function_name!()));
    actix_web::HttpResponse::InternalServerError().body("Failed to insert role")
}

#[named]
#[actix_web::post("/api/assign_role")]
pub async fn assign_role(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::UserServerRoleRequest>,
) -> impl actix_web::Responder {
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);

    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await
    .is_none()
    {
        return actix_web::HttpResponse::Unauthorized().body("Invalid token");
    }

    match session
        .lock
        .lock()
        .unwrap()
        .query_unpaged(
            db::statics::ASSIGN_ROLE_TO_USER,
            (req.server_id.clone(), req.target_user.clone(), req.role_name.clone()),
        )
        .await
    {
        Ok(_) => actix_web::HttpResponse::Ok().body("Role assigned"),
        Err(e) => {
            logging::log(&format!("Error assigning role: {:?}", e), Some(function_name!()));
            actix_web::HttpResponse::InternalServerError().body("Failed to assign role")
        }
    }
}

#[named]
#[actix_web::post("/api/remove_role")]
pub async fn remove_role(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::UserServerRoleRequest>,
) -> impl actix_web::Responder {
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);

    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await
    .is_none()
    {
        return actix_web::HttpResponse::Unauthorized().body("Invalid token");
    }

    match session
        .lock
        .lock()
        .unwrap()
        .query_unpaged(
            db::statics::REMOVE_ROLE_FROM_USER,
            (req.server_id.clone(), req.target_user.clone(), req.role_name.clone()),
        )
        .await
    {
        Ok(_) => actix_web::HttpResponse::Ok().body("Role removed successfully"),
        Err(e) => {
            logging::log(&format!("Error removing role: {:?}", e), Some(function_name!()));
            actix_web::HttpResponse::InternalServerError().body("Failed to remove role")
        }
    }
}

#[named]
#[actix_web::post("/api/delete_server_role")]
pub async fn delete_server_role(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::DeleteServerRoleRequest>,
) -> impl actix_web::Responder {
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);

    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await
    .is_none()
    {
        return actix_web::HttpResponse::Unauthorized().body("Invalid token");
    }

    match session
        .lock
        .lock()
        .unwrap()
        .query_unpaged(
            db::statics::DELETE_SERVER_ROLE,
            (req.server_id.clone(), req.role_name.clone()),
        )
        .await
    {
        Ok(_) => actix_web::HttpResponse::Ok().body("Role deleted successfully"),
        Err(e) => {
            logging::log(&format!("Error deleting role: {:?}", e), Some(function_name!()));
            actix_web::HttpResponse::InternalServerError().body("Failed to delete role")
        }
    }
}
