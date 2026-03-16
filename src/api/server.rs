#![allow(unused_imports)]
use scylla::client::session::Session;

use crate::api::statics;
use crate::api::structures;
use crate::db;
use crate::security;
use crate::utils::logging;
use crate::metrics;

use ::function_name::named;

#[named]
#[actix_web::post("/create_server")]
pub async fn create_server(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::CreateServer>,
    shared_collector: actix_web::web::Data<structures::AppState>,
) -> impl actix_web::Responder {
    if req.name.len() > statics::MAX_SERVER_LENGTH {
        return actix_web::HttpResponse::LengthRequired().body(format!(
            "Failed to create server: Server name longer than {}",
            statics::MAX_SERVER_LENGTH
        ));
    }
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
        logging::log("SERVERS FAIL: invalid token in create_server", Some(function_name!()));
        return actix_web::HttpResponse::Unauthorized().body("Invalid token");
    }
    
    let sid = security::sid();
    if db::server::create_server(
        &scylla_session,
        sid.clone(),
        &req.desc,
        &req.img_url,
        &req.name,
        req.username.clone(),
    )
    .await
    .is_none()
    {
        logging::log("SERVERS FAIL: create_server", Some(function_name!()));
        return actix_web::HttpResponse::InternalServerError().body("Failed to create server");
    }
    
    let _ =
        db::server::create_channel(&scylla_session, sid.clone(), "info".to_string()).await;
    let server_created = structures::ServerCreatedResponse {
        token: security::token(),
        sid: sid.clone(),
    };
    let _ = db::prelude::insert_user_token(
        &scylla_session,
        &cache,
        db::structures::KeyUser {
            key: Some(security::armor_token(&server_created.token)),
            username: Some(req.username.clone()),
        },
    )
    .await;

   


    if db::server::add_user_to_server(&scylla_session, sid.clone(), req.username.clone())
        .await
        .is_some()
    {
        let admin_role = db::structures::ServerRole {
            server_id: sid.clone(),
            name: "admin".to_string(),
            color: String::new(),
            permissions: db::structures::Permissions::SEND_MESSAGES.bits() | db::structures::Permissions::ADD_ROLE.bits(),
        };
        let member_role = db::structures::ServerRole {
            server_id: sid.clone(),
            name: "member".to_string(),
            color: String::new(),
            permissions: db::structures::Permissions::SEND_MESSAGES.bits(),
        };

        
        let _ = db::roles::insert_server_role(&scylla_session, sid.clone(), admin_role).await;
        let _ = db::roles::insert_server_role(&scylla_session,sid.clone(), member_role).await;

        let _ = scylla_session
            .query_unpaged(
                db::statics::ASSIGN_ROLE_TO_USER,
                (sid.clone(), req.username.clone(), "admin".to_string()),
            )
            .await;
        return actix_web::HttpResponse::Ok().json(&server_created);
    }
    logging::log("SERVERS FAIL: add_user_to_server", Some(function_name!()));
    actix_web::HttpResponse::InternalServerError().body("Failed to add user to server")
}

#[named]
#[actix_web::post("/servers/{sid}/join")]
pub async fn join_server(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::TokenUser>,
    http: actix_web::HttpRequest,
    shared_collector: actix_web::web::Data<structures::AppState>,
) -> impl actix_web::Responder {
    let sid: String = param!(http, "sid");
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
        logging::log("SERVERS FAIL: invalid token in create_server", Some(function_name!()));
        return actix_web::HttpResponse::Unauthorized().body("Invalid token");
    }
    
    if db::server::add_user_to_server(&scylla_session, sid.clone(), req.username.clone())
        .await
        .is_none()
    {
        logging::log("SERVERS FAIL: add_user_to_server", Some(function_name!()));
        return actix_web::HttpResponse::InternalServerError().body("Failed to add user to server");
    }

    

    // give the member role to anyone that joins the server
    let _ = scylla_session
        .query_unpaged(
            db::statics::ASSIGN_ROLE_TO_USER,
            (sid, req.username.clone(), "member".to_string()),
        )
        .await;

    let new_token_holder = structures::TokenHolder {
        token: security::token(),
    };
    let _ = db::prelude::insert_user_token(
        &scylla_session,
        &cache,
        db::structures::KeyUser {
            key: Some(security::armor_token(&new_token_holder.token)),
            username: Some(req.username.clone()),
        },
    )
    .await;

    let _ = db::users::delete_token(
        &scylla_session,
        req.username.clone(),
        security::armor_token(&req.token),
    )
    .await;

    actix_web::HttpResponse::Ok().json(&new_token_holder)
}

#[actix_web::post("/servers/{sid}/get_server_users")]
pub async fn get_server_users(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::TokenUser>,
    http: actix_web::HttpRequest,
    shared_collector: actix_web::web::Data<structures::AppState>,
) -> impl actix_web::Responder {
    let sid: String = param!(http, "sid");
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    let collector = cache_metrics!(shared_collector);

    if db::prelude::check_user_is_in_server(
        &scylla_session,
        &cache,
        sid.clone(),
        req.token.clone(),
        req.username.clone(),
        &collector,
    )
    .await
    .is_none()
    {
        return actix_web::HttpResponse::Unauthorized().body("Invalid token or user not in server");
    }
    
    if let Some(users) = db::server::fetch_server_users(&scylla_session, sid.clone()).await {
        return actix_web::HttpResponse::Ok().json(&structures::UsersList { u_list: users });
    }
    
    actix_web::HttpResponse::Ok().json(&structures::UsersList { u_list: Vec::new() })
}

#[actix_web::get("/servers/{sid}/get_server_info")]
pub async fn get_server_info(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    http: actix_web::HttpRequest,
) -> impl actix_web::Responder {
    let sid: String = param!(http, "sid");
    let scylla_session = scylla_session!(session);
    if let Some(server_info) = db::server::fetch_server_info(&scylla_session, sid.clone()).await {
        return actix_web::HttpResponse::Ok().json(&server_info);
    }
    actix_web::HttpResponse::NotFound().json(&structures::Status {
        status: "Could not find server information.".to_string(),
    })
}

#[named]
#[actix_web::post("/servers/{sid}/delete_server")]
pub async fn delete_server(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::TokenUser>,
    http: actix_web::HttpRequest,
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

    let sid: String = param!(http, "sid");

    if db::server::check_user_is_owner(&scylla_session, sid.clone(), req.username.clone()).await
        != Some(true)
    {
        logging::log("Unauthorized: not server owner", Some(function_name!()));
        return actix_web::HttpResponse::Unauthorized()
            .body("You don't have permission to delete this server");
    }

    if db::server::delete_server(&scylla_session, sid)
        .await
        .is_some()
    {
        return actix_web::HttpResponse::Ok().body("Server deleted successfully");
    }
    
    actix_web::HttpResponse::InternalServerError().body("Failed to delete server")

}

#[actix_web::post("/am_i_in_server")]
pub async fn am_i_in_server(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::TokenUserServer>,
    shared_collector: actix_web::web::Data<structures::AppState>,
) -> impl actix_web::Responder {
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    let collector = cache_metrics!(shared_collector);

    if db::prelude::check_user_is_in_server(
        &scylla_session,
        &cache,
        req.sid.clone(),
        req.token.clone(),
        req.username.clone(),
        &collector,
    )
    .await
    .is_some()
    {
        return actix_web::HttpResponse::Ok().body("Yes you are part of the server.");
    }

    actix_web::HttpResponse::NotFound().body("You are not part of this server.")
}
