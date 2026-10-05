#![allow(unused_imports)]
use scylla::client::session::Session;

use crate::security;
use crate::utils::logging;
use crate::metrics;

use ::function_name::named;

// NOTE: Specifically v1 db...
use crate::v1::api::structures;
use crate::v1::db;
use crate::db as db0;
use crate::api::structures as structures0;
use crate::api::statics as statics0;

/// Creates a new server owned by the caller, along with a default "info" channel and default
/// "admin"/"member" roles. The caller is automatically added as a member with the "admin" role,
/// and a new session token is issued (the old one is invalidated).
///
/// ### Why v1 and not v0?
/// `v1` adds more complete permissions to the owner of the server.
///
/// ### Request JSON (`CreateServer`)
/// ```json
/// {
///   "token": "abc123",
///   "desc": "My cool server",
///   "img_url": "https://example.com/img.png",
///   "name": "My Server",
///   "username": "alice"
/// }
/// ```
///
/// ### Example (reqwest)
/// ```rust
/// let client = reqwest::Client::new();
/// let res = client
///     .post("http://localhost:1313/create_server")
///     .json(&serde_json::json!({
///         "token": "abc123",
///         "desc": "My cool server",
///         "img_url": "https://example.com/img.png",
///         "name": "My Server",
///         "username": "alice"
///     }))
///     .send()
///     .await?;
/// ```
#[named]
#[actix_web::post("/v1/create_server")]
pub async fn create_server(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures0::CreateServer>,
    shared_collector: actix_web::web::Data<structures0::AppState>,
) -> impl actix_web::Responder {
    if req.name.len() > statics0::MAX_SERVER_LENGTH {
        return actix_web::HttpResponse::LengthRequired().body(format!(
            "Failed to create server: Server name longer than {}",
            statics0::MAX_SERVER_LENGTH
        ));
    }
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    let collector = cache_metrics!(shared_collector);
    if db0::prelude::check_token(
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
    if db0::server::create_server(
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
        db0::server::create_channel(&scylla_session, sid.clone(), "info".to_string()).await;
    let mut server_created = structures0::ServerCreatedResponse {
        token: security::token(),
        sid: sid.clone(),
    };
    let armored_new_token = security::armor_token_logged(&server_created.token);
    if armored_new_token.is_none() {
        server_created.token = req.token.clone();
    }
    if let Some(armored_new_token) = armored_new_token {
        if let Err(insert_err) = db0::prelude::insert_user_token(
            &scylla_session,
            &cache,
            db0::structures::KeyUser {
                key: Some(armored_new_token),
                username: Some(req.username.clone()),
            },
        ).await
        {
            logging::log(&format!("Failed to insert token due to error:\n {insert_err}"), Some(function_name!()));
            server_created.token = req.token.clone();
        } else if let Some(armored_old_token) = security::armor_token_logged(&req.token) {
            let _ = db0::users::delete_token(
                &scylla_session,
                req.username.clone(),
                armored_old_token,
            )
            .await;
        }
    }

    if db0::server::add_user_to_server(&scylla_session, sid.clone(), req.username.clone())
        .await
        .is_some()
    {
        let admin_role = db0::structures::ServerRole {
            server_id: sid.clone(),
            name: "admin".to_string(),
            color: String::new(),
            permissions: 
                db::structures::Permissions::SEND_MESSAGES.bits() 
                | db::structures::Permissions::ADD_ROLE.bits()
                | db::structures::Permissions::CHANGE_FE.bits(),
        };
        let member_role = db0::structures::ServerRole {
            server_id: sid.clone(),
            name: "member".to_string(),
            color: String::new(),
            permissions: db::structures::Permissions::SEND_MESSAGES.bits(),
        };
        
        let _ = db0::roles::insert_server_role(&scylla_session, sid.clone(), admin_role).await;
        let _ = db0::roles::insert_server_role(&scylla_session,sid.clone(), member_role).await;

        let _ = scylla_session.query_unpaged(
                db0::statics::ASSIGN_ROLE_TO_USER,
                (sid.clone(), req.username.clone(), "admin".to_string()),
            )
            .await;
        return actix_web::HttpResponse::Ok().json(&server_created);
    }
    logging::log("SERVERS FAIL: add_user_to_server", Some(function_name!()));
    actix_web::HttpResponse::InternalServerError().body("Failed to add user to server")
}

/// Fetches public information about a server (e.g. name, description, image). This endpoint
/// takes no request body — the server id (`sid`) is supplied as a URL path parameter.
///
/// ### Request JSON
/// _None. This is a `GET` request with no body._
///
/// ### Example (reqwest)
/// ```rust
/// let client = reqwest::Client::new();
/// let res = client
///     .get("http://localhost:1313/v1/servers/SID123/info")
///     .send()
///     .await?;
/// ```
#[named]
#[actix_web::get("/v1/servers/{sid}/info")]
pub async fn get_info(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    http: actix_web::HttpRequest,
) -> impl actix_web::Responder {
    let sid: String = param!(http, "sid");
    let scylla_session = scylla_session!(session);
    if let Some(server_info) = db::server::fetch_info(&scylla_session, sid.clone()).await {
        return actix_web::HttpResponse::Ok().json(&server_info);
    }
    logging::log(
        &format!("Could not find server({}) information...", sid.clone()),
        Some(function_name!())
    );
    actix_web::HttpResponse::NotFound().json(&structures::Status {
        status: "Could not find server information.".to_string(),
    })
}

/// Fetches a server's frontend configuration. This endpoint takes no request
/// body — the server id (`sid`) is supplied as a URL path parameter.
///
/// ### Request JSON
/// _None. This is a `GET` request with no body._
///
/// ### Example (reqwest)
/// ```rust
/// let client = reqwest::Client::new();
/// let res = client
///     .get("http://localhost:1313/v1/servers/SID123/frontend")
///     .send()
///     .await?;
/// ```
#[named]
#[actix_web::get("/v1/servers/{sid}/frontend")]
pub async fn get_frontend(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    http: actix_web::HttpRequest,
) -> impl actix_web::Responder {
    let sid: String = param!(http, "sid");
    let scylla_session = scylla_session!(session);
    if let Some(server_info) = db::server::fetch_fe_config(&scylla_session, sid.clone()).await {
        return actix_web::HttpResponse::Ok().json(&server_info);
    }
    logging::log(
        &format!("Could not find server({}) frontend configuration...", sid.clone()),
        Some(function_name!())
    );
    actix_web::HttpResponse::NotFound().json(&structures::Status {
        status: "Could not find server frontend configuration.".to_string(),
    })
}

/// Patches a server's frontend configuration. This endpoint takes no request
/// body — the server id (`sid`) is supplied as a URL path parameter.
///
/// ### Request JSON
/// _None. This is a `GET` request with no body._
///
/// ### Example (reqwest)
/// ```rust
/// let client = reqwest::Client::new();
/// let res = client
///     .patch("http://localhost:1313/v1/servers/SID123/frontend")
///     .send()
///     .await?;
/// ```
#[named]
#[actix_web::patch("/v1/servers/{sid}/frontend")]
pub async fn patch_frontend(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::UpdateServerFrontend>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    shared_collector: actix_web::web::Data<structures0::AppState>,
    http: actix_web::HttpRequest,
) -> impl actix_web::Responder {

    let sid: String = param!(http, "sid");
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    let collector = cache_metrics!(shared_collector);

    if db0::prelude::check_permission(
        &scylla_session,
        &cache,
        sid.clone(),
        req.token.clone(),
        req.username.clone(),
        db::structures::Permissions::CHANGE_FE.bits(),
        &collector,
    )
    .await
    .is_none()
    {
        return actix_web::HttpResponse::Forbidden().body("You do not have permission to manage frontend configuration.");
    }
    
    if let Err(e) = db::server::update_fe_config(
        &scylla_session,
        sid.clone(),
        req.config.clone()
    ).await {
        logging::log(
            &format!("Could not update server({}) frontend configuration... Got error: {:?}", sid.clone(), e),
            Some(function_name!()),
        );
        return actix_web::HttpResponse::NotFound().json(&structures::Status {
            status: "Could not update frontend configuration for server.".to_string(),
        });
    }

    actix_web::HttpResponse::Ok()
        .body("Frontend configuration updated!")
}
