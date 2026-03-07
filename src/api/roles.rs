use crate::api::structures;
use crate::db;
use crate::security;
use crate::utils::logging;
use crate::metrics;

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

    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.username.clone()),
        &collector,
    )
    .await
    .is_none()
    {
        return actix_web::HttpResponse::Unauthorized().body("Invalid token");
    }
    
    let role = db::structures::ServerRole {
        role_name: req.role_name.clone(),
        server_id: req.server_id.clone(),
        color: req.color.clone(),
        permissions: req
            .permissions
            .clone()
            .unwrap_or_default()
            .into_iter()
            .collect::<std::collections::HashSet<String>>(),
    };

    if let Some(result) =
        db::roles::insert_server_role(&scylla_session, req.server_id.clone(), role).await
        && result.is_ok() {
            return actix_web::HttpResponse::Ok().body("Role added successfully");
        }
    logging::log(&format!("Error inserting role: {:?}", req.role_name.clone()), Some(function_name!()));
    actix_web::HttpResponse::InternalServerError().body("Failed to insert role")
}

#[named]
#[actix_web::post("/delete_server_role")]
pub async fn delete_server_role(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::DeleteServerRoleRequest>,
    shared_collector: actix_web::web::Data<structures::AppState>,
) -> impl actix_web::Responder {
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    let collector = cache_metrics!(shared_collector);

    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.username.clone()),
        &collector,
    )
    .await
    .is_none()
    {
        return actix_web::HttpResponse::Unauthorized().body("Invalid user token.");
    }
    
    if let Some(role_exists) =
        db::roles::check_role_exists(&scylla_session, &req.server_id, &req.role_name).await
        && !role_exists
    {
        return actix_web::HttpResponse::BadRequest()
            .body("Role does not exist on this server.");
    }

    if let Some(result) = db::roles::remove_role_from_all_users(
        &scylla_session,
        req.server_id.clone(),
        req.role_name.clone(),
    )
    .await
    {
        if result.is_err() {
            logging::log(&format!("Error removing role from users: {:?}/{:?}", req.role_name.clone(), req.username.clone()), Some(function_name!()));
            return actix_web::HttpResponse::InternalServerError()
                .body("Failed to remove role from users.");
        }
        
        if let Some(result) = db::roles::delete_server_role(
            &scylla_session,
            req.server_id.clone(),
            req.role_name.clone(),
        )
        .await
        {
            if result.is_err() {
                logging::log(&format!("Error deleting role: {:?}",req.role_name.clone()), Some(function_name!()));
                return actix_web::HttpResponse::InternalServerError()
                    .body("Failed to delete role from server.");
            }
            return actix_web::HttpResponse::Ok().body("Role deleted successfully");
        }
    }
    actix_web::HttpResponse::InternalServerError().body("Not able to delete role.")
}

#[named]
#[actix_web::post("/assign_role_to_user")]
pub async fn assign_role_to_user(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::UserServerRoleRequest>,
    shared_collector: actix_web::web::Data<structures::AppState>,
) -> impl actix_web::Responder {
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    let collector = cache_metrics!(shared_collector);

    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.username.clone()),
        &collector,
    )
    .await
    .is_none()
    {
        return actix_web::HttpResponse::Unauthorized().body("Invalid token");
    }
    
    if let Some(role_exists) =
        db::roles::check_role_exists(&scylla_session, &req.server_id, &req.role_name).await
    {
        if !role_exists {
            return actix_web::HttpResponse::BadRequest()
                .body("Role does not exist on this server");
        }
    } else {
        return actix_web::HttpResponse::InternalServerError()
            .body("Failed to check role existence on server.");
    }

    let user_role = db::structures::UserServerRole {
        server_id: req.server_id.clone(),
        username: req.username.clone(),
        role_name: req.role_name.clone(),
    };
    if let Some(result) = db::roles::assign_role_to_user(&scylla_session, user_role).await && result.is_ok() {
        return actix_web::HttpResponse::BadRequest().body("Role assigned successfully");
    }

    logging::log(&format!("Error assigning role: {:?}", req.role_name.clone()), Some(function_name!()));
    actix_web::HttpResponse::InternalServerError().body("Failed to assign role")
}

#[named]
#[actix_web::post("/remove_role_from_user")]
pub async fn remove_role_from_user(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::UserServerRoleRequest>,
    shared_collector: actix_web::web::Data<structures::AppState>,
) -> impl actix_web::Responder {
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    let collector = cache_metrics!(shared_collector);

    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.username.clone()),
        &collector,
    )
    .await
    .is_none()
    {
        return actix_web::HttpResponse::Unauthorized().body("Invalid token");
    }
    
    let user_role = db::structures::UserServerRole {
        server_id: req.server_id.clone(),
        username: req.username.clone(),
        role_name: req.role_name.clone(),
    };

    if let Some(result) = db::roles::remove_role_from_user(&scylla_session, user_role).await && result.is_ok() {
        return actix_web::HttpResponse::Ok().body("Role removed successfully");
    }
    
    logging::log(&format!("Error removing role: {:?}", req.role_name.clone()), Some(function_name!()));
    actix_web::HttpResponse::InternalServerError().body("Failed to remove role")

}

#[actix_web::get("/fetch_server_roles")]
pub async fn fetch_server_roles(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    query: actix_web::web::Query<structures::ServerRoleQuery>,
    shared_collector: actix_web::web::Data<structures::AppState>,
) -> impl actix_web::Responder {
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    let collector = cache_metrics!(shared_collector);

    if db::prelude::check_token(
        &scylla_session,
        &cache,
        query.token.clone(),
        Some(query.username.clone()),
        &collector,
    )
    .await
    .is_none()
    {
        return actix_web::HttpResponse::Unauthorized().body("Invalid Token");
    }
    
    if let Some(roles) = db::roles::fetch_server_roles(&scylla_session, &query.server_id).await
    {
        return actix_web::HttpResponse::Ok().json(roles);
    }
    actix_web::HttpResponse::InternalServerError().body("Failed to fetch server roles")

}

#[actix_web::get("/fetch_user_roles")]
pub async fn fetch_user_roles(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    query: actix_web::web::Query<structures::UserRoleQuery>,
    shared_collector: actix_web::web::Data<structures::AppState>,
) -> impl actix_web::Responder {
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    let collector = cache_metrics!(shared_collector);

    if db::prelude::check_token(
        &scylla_session,
        &cache,
        query.token.clone(),
        Some(query.username.clone()),
        &collector
    )
    .await
    .is_none()
    {
        return actix_web::HttpResponse::Unauthorized().body("Invalid Token");
    }
    
    if let Some(roles) = db::roles::fetch_user_roles(
        &scylla_session,
        query.server_id.clone(),
        query.username.clone(),
    )
    .await
    {
        return actix_web::HttpResponse::Ok().json(roles);
    }
    actix_web::HttpResponse::InternalServerError().body("Failed to fetch user roles")

}
