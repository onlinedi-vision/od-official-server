#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use crate::api::structures;
use crate::db;
use crate::security;

#[actix_web::post("/api/add_server_role")]
pub async fn add_server_role(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::ServerRoleRequest>,
) -> impl actix_web::Responder {
    let scylla_session = match session.lock.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return actix_web::HttpResponse::InternalServerError()
                .body("Internal error: scylla session lock poisoned.");
        }
    };
    let cache = match shared_cache.lock.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return actix_web::HttpResponse::InternalServerError()
                .body("Internal error: cache lock poisoned.");
        }
    };

    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await
    .is_some()
    {
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
        {
            return match result {
                Ok(_) => actix_web::HttpResponse::Ok().body("Role added successfully"),
                Err(err) => {
                    println!("Error inserting role: {:?}", err);
                    actix_web::HttpResponse::InternalServerError().body("Failed to insert role")
                }
            };
        }
    }

    actix_web::HttpResponse::Unauthorized().body("Invalid token")
}

#[actix_web::post("/api/delete_server_role")]
pub async fn delete_server_role(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::DeleteServerRoleRequest>,
) -> impl actix_web::Responder {
    let scylla_session = match session.lock.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return actix_web::HttpResponse::InternalServerError()
                .body("Internal error: scylla session lock poisoned.");
        }
    };
    let cache = match shared_cache.lock.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return actix_web::HttpResponse::InternalServerError()
                .body("Internal error: cache lock poisoned.");
        }
    };

    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await
    .is_some()
    {
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
            match result {
                Ok(_) => {
                    if let Some(result) = db::roles::delete_server_role(
                        &scylla_session,
                        req.server_id.clone(),
                        req.role_name.clone(),
                    )
                    .await
                    {
                        return match result {
                            Ok(_) => {
                                actix_web::HttpResponse::Ok().body("Role deleted successfully")
                            }
                            Err(err) => {
                                println!("Error deleting role: {:?}", err);
                                actix_web::HttpResponse::InternalServerError()
                                    .body("Failed to delete role from server.")
                            }
                        };
                    }
                }
                Err(err) => {
                    println!("Error removing role from users: {:?}", err);
                    return actix_web::HttpResponse::InternalServerError()
                        .body("Failed to remove role from users.");
                }
            }
        }
    }

    actix_web::HttpResponse::Unauthorized().body("Invalid user token.")
}

#[actix_web::post("/api/assign_role_to_user")]
pub async fn assign_role_to_user(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::UserServerRoleRequest>,
) -> impl actix_web::Responder {
    let scylla_session = match session.lock.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return actix_web::HttpResponse::InternalServerError()
                .body("Internal error: scylla session lock poisoned.");
        }
    };
    let cache = match shared_cache.lock.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return actix_web::HttpResponse::InternalServerError()
                .body("Internal error: cache lock poisoned.");
        }
    };

    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await
    .is_some()
    {
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
        if let Some(result) = db::roles::assign_role_to_user(&scylla_session, user_role).await {
            return match result {
                Ok(_) => actix_web::HttpResponse::BadRequest().body("Role assigned successfully"),
                Err(err) => {
                    println!("Error assigning role: {:?}", err);
                    actix_web::HttpResponse::InternalServerError().body("Failed to assign role")
                }
            };
        }
    }

    actix_web::HttpResponse::Unauthorized().body("Invalid token")
}

#[actix_web::post("/api/remove_role_from_user")]
pub async fn remove_role_from_user(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::UserServerRoleRequest>,
) -> impl actix_web::Responder {
    let scylla_session = match session.lock.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return actix_web::HttpResponse::InternalServerError()
                .body("Internal error: scylla session lock poisoned.");
        }
    };
    let cache = match shared_cache.lock.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return actix_web::HttpResponse::InternalServerError()
                .body("Internal error: cache lock poisoned.");
        }
    };

    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await
    .is_some()
    {
        let user_role = db::structures::UserServerRole {
            server_id: req.server_id.clone(),
            username: req.username.clone(),
            role_name: req.role_name.clone(),
        };

        if let Some(result) = db::roles::remove_role_from_user(&scylla_session, user_role).await {
            return match result {
                Ok(_) => actix_web::HttpResponse::Ok().body("Role removed successfully"),
                Err(err) => {
                    println!("Error removing role: {:?}", err);
                    actix_web::HttpResponse::InternalServerError().body("Failed to remove role")
                }
            };
        }
    }
    actix_web::HttpResponse::Unauthorized().body("Invalid token")
}

#[actix_web::get("/api/fetch_server_roles")]
pub async fn fetch_server_roles(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    query: actix_web::web::Query<structures::ServerRoleQuery>,
) -> impl actix_web::Responder {
    let scylla_session = match session.lock.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return actix_web::HttpResponse::InternalServerError()
                .body("Internal error: scylla session lock poisoned.");
        }
    };
    let cache = match shared_cache.lock.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return actix_web::HttpResponse::InternalServerError()
                .body("Internal error: cache lock poisoned.");
        }
    };

    if db::prelude::check_token(
        &scylla_session,
        &cache,
        query.token.clone(),
        Some(query.username.clone()),
    )
    .await
    .is_some()
    {
        if let Some(roles) = db::roles::fetch_server_roles(&scylla_session, &query.server_id).await
        {
            return actix_web::HttpResponse::Ok().json(roles);
        }
        return actix_web::HttpResponse::InternalServerError().body("Failed to fetch server roles");
    }

    actix_web::HttpResponse::Unauthorized().body("Invalid Token")
}

#[actix_web::get("/api/fetch_user_roles")]
pub async fn fetch_user_roles(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    query: actix_web::web::Query<structures::UserRoleQuery>,
) -> impl actix_web::Responder {
    let scylla_session = match session.lock.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return actix_web::HttpResponse::InternalServerError()
                .body("Internal error: scylla session lock poisoned.");
        }
    };
    let cache = match shared_cache.lock.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return actix_web::HttpResponse::InternalServerError()
                .body("Internal error: cache lock poisoned.");
        }
    };

    if db::prelude::check_token(
        &scylla_session,
        &cache,
        query.token.clone(),
        Some(query.username.clone()),
    )
    .await
    .is_some()
    {
        if let Some(roles) = db::roles::fetch_user_roles(
            &scylla_session,
            query.server_id.clone(),
            query.username.clone(),
        )
        .await
        {
            return actix_web::HttpResponse::Ok().json(roles);
        }
        return actix_web::HttpResponse::InternalServerError().body("Failed to fetch user roles");
    }

    actix_web::HttpResponse::Unauthorized().body("Invalid Token")
}
