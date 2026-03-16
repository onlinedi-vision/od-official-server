use crate::api::statics;
use crate::api::structures;
use crate::db;
use crate::security;
use crate::utils::logging;

use ::function_name::named;

#[named]
#[actix_web::post("/add_server_role")]
pub async fn add_server_role(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::ServerRoleRequest>,
    shared_collector: actix_web::web::Data<structures::AppState>,
) -> impl actix_web::Responder {
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    let collector = cache_metrics!(shared_collector);

    if req.name.len() > statics::MAX_ROLE_NAME_LENGTH {
        return actix_web::HttpResponse::BadRequest()
            .body(format!("Role name exceeds maximum length of {}", statics::MAX_ROLE_NAME_LENGTH));
    }

    if db::prelude::check_permission(
        &scylla_session,
        &cache,
        req.server_id.clone(),
        req.token.clone(),
        req.username.clone(),
        db::structures::Permissions::ADD_ROLE.bits(),
        &collector,
    )
    .await
    .is_none()
    {
        return actix_web::HttpResponse::Forbidden().body("You do not have permission to manage roles");
    }
    
    //gotta watch out for injection attacks
    if (req.permissions & !db::structures::Permissions::all().bits()) != 0{
        return actix_web::HttpResponse::Forbidden().body("Invalid permission request!")
    }


    let role = db::structures::ServerRole {
        name: req.name.clone(),
        server_id: req.server_id.clone(),
        color: req.color.clone().unwrap_or_default(),
        permissions: req.permissions.clone(),
    };

    match db::roles::insert_server_role(&scylla_session, req.server_id.clone(), role).await {
        Ok(()) => actix_web::HttpResponse::Ok().body("Role added successfully"),
        Err(e) => {
            logging::log(&format!("Error inserting role '{}': {:?}", req.name, e), Some(function_name!()));
            actix_web::HttpResponse::InternalServerError().body("Failed to insert role")
        }
    }
}

#[named]
#[actix_web::post("/api/assign_role")]
pub async fn assign_role(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::UserServerRoleRequest>,
    shared_collector: actix_web::web::Data<structures::AppState>,
) -> impl actix_web::Responder {
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    let collector = cache_metrics!(shared_collector);

    if req.role_name.len() > statics::MAX_ROLE_NAME_LENGTH {
        return actix_web::HttpResponse::BadRequest()
            .body(format!("Role name exceeds maximum length of {}", statics::MAX_ROLE_NAME_LENGTH));
    }

    if db::prelude::check_permission(
        &scylla_session,
        &cache,
        req.server_id.clone(),
        req.token.clone(),
        req.username.clone(),
        db::structures::Permissions::ADD_ROLE.bits(),
        &collector,
    )
    .await
    .is_none()
    {
        return actix_web::HttpResponse::Forbidden().body("You do not have permission to manage roles");
    }

    if !db::prelude::is_member_of_server(
        &scylla_session,
        req.server_id.clone(),
        req.target_user.clone(),
    )
    .await
    {
        return actix_web::HttpResponse::BadRequest().body("Target user is not in the server");
    }

    match db::roles::assign_role(
        &scylla_session,
        req.server_id.clone(),
        req.target_user.clone(),
        req.role_name.clone(),
    ).await
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
    shared_collector: actix_web::web::Data<structures::AppState>,
) -> impl actix_web::Responder {
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    let collector = cache_metrics!(shared_collector);

    if req.role_name.len() > statics::MAX_ROLE_NAME_LENGTH {
        return actix_web::HttpResponse::BadRequest()
            .body(format!("Role name exceeds maximum length of {}", statics::MAX_ROLE_NAME_LENGTH));
    }

    if db::prelude::check_permission(
        &scylla_session,
        &cache,
        req.server_id.clone(),
        req.token.clone(),
        req.username.clone(),
        db::structures::Permissions::ADD_ROLE.bits(),
        &collector,
    )
    .await
    .is_none()
    {
        return actix_web::HttpResponse::Forbidden().body("You do not have permission to manage roles");
    }

    if !db::prelude::is_member_of_server(
        &scylla_session,
        req.server_id.clone(),
        req.target_user.clone(),
    )
    .await
    {
        return actix_web::HttpResponse::BadRequest().body("Target user is not in the server");
    }

    match db::roles::remove_role(
        &scylla_session,
        req.server_id.clone(), 
        req.target_user.clone(),
        req.role_name.clone(),
        ).await
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
    shared_collector: actix_web::web::Data<structures::AppState>,
) -> impl actix_web::Responder {
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    let collector = cache_metrics!(shared_collector);

    if req.role_name.len() > statics::MAX_ROLE_NAME_LENGTH {
        return actix_web::HttpResponse::BadRequest()
            .body(format!("Role name exceeds maximum length of {}", statics::MAX_ROLE_NAME_LENGTH));
    }

    if db::prelude::check_permission(
        &scylla_session,
        &cache,
        req.server_id.clone(),
        req.token.clone(),
        req.username.clone(),
        db::structures::Permissions::ADD_ROLE.bits(),
        &collector,
    )
    .await
    .is_none()
    {
        return actix_web::HttpResponse::Forbidden().body("You do not have permission to manage roles");
    }

    if let Err(e) = db::roles::remove_role_from_all_users(
        &scylla_session,
        req.server_id.clone(),
        req.role_name.clone(),
    ).await {
        logging::log(&format!("Error cleaning up role assignments: {:?}", e), Some(function_name!()));
    }

    match db::roles::delete_role(&scylla_session, 
        req.server_id.clone(),
        req.role_name.clone())
        .await
    {
        Ok(_) => actix_web::HttpResponse::Ok().body("Role deleted successfully"),
        Err(e) => {
            logging::log(&format!("Error deleting role: {:?}", e), Some(function_name!()));
            actix_web::HttpResponse::InternalServerError().body("Failed to delete role")
        }
    }
}