#![allow(unused_imports)]
use scylla::client::session::Session;

use crate::api::statics;
use crate::api::structures;
use crate::db;
use crate::security;
use crate::utils::logging;
use crate::metrics;

use ::function_name::named;

/// Creates a new server owned by the caller, along with a default "info" channel and default
/// "admin"/"member" roles. The caller is automatically added as a member with the "admin" role,
/// and a new session token is issued (the old one is invalidated).
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
    let mut server_created = structures::ServerCreatedResponse {
        token: security::token(),
        sid: sid.clone(),
    };
    let armored_new_token = security::armor_token_logged(&server_created.token);
    if armored_new_token.is_none() {
        server_created.token = req.token.clone();
    }
    if let Some(armored_new_token) = armored_new_token {
        if let Err(insert_err) = db::prelude::insert_user_token(
            &scylla_session,
            &cache,
            db::structures::KeyUser {
                key: Some(armored_new_token),
                username: Some(req.username.clone()),
            },
        )
        .await
        {
            logging::log(&format!("Failed to insert token due to error:\n {insert_err}"), Some(function_name!()));
            server_created.token = req.token.clone();
        } else if let Some(armored_old_token) = security::armor_token_logged(&req.token) {
            let _ = db::users::delete_token(
                &scylla_session,
                req.username.clone(),
                armored_old_token,
            )
            .await;
        }
    }

   


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

/// Adds the caller to a server and assigns them the "member" role. Issues a new session token
/// on success (the old one is invalidated).
///
/// ### Request JSON (`TokenUser`)
/// ```json
/// {
///   "username": "alice",
///   "token": "abc123"
/// }
/// ```
///
/// ### Example (reqwest)
/// ```rust
/// let client = reqwest::Client::new();
/// let res = client
///     .post("http://localhost:1313/servers/SID123/join")
///     .json(&serde_json::json!({
///         "username": "alice",
///         "token": "abc123"
///     }))
///     .send()
///     .await?;
/// ```
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
    if let Err(insert_err) = db::prelude::insert_user_token(
        &scylla_session,
        &cache,
        db::structures::KeyUser {
            key: Some(armor_token_or!(
                &new_token_holder.token,
                actix_web::HttpResponse::InternalServerError().body("Failed to insert new token")
            )),
            username: Some(req.username.clone()),
        },
    )
    .await {
        logging::log(&format!("Failed to insert token due to error:\n {insert_err}"), Some(function_name!()));
        return actix_web::HttpResponse::InternalServerError().body("Failed to insert new token");
    }

    let _ = db::users::delete_token(
        &scylla_session,
        req.username.clone(),
        armor_token_or!(
            &req.token,
            actix_web::HttpResponse::InternalServerError().body("Failed to rotate token")
        ),
    )
    .await;

    actix_web::HttpResponse::Ok().json(&new_token_holder)
}

/// Fetches the public profile info (username, bio, image, roles) of every user in a server.
/// The caller must be a member of that server.
///
/// ### Request JSON (`TokenUser`)
/// ```json
/// {
///   "username": "alice",
///   "token": "abc123"
/// }
/// ```
///
/// ### Example (reqwest)
/// ```rust
/// let client = reqwest::Client::new();
/// let res = client
///     .post("http://localhost:1313/servers/SID123/get_server_users")
///     .json(&serde_json::json!({
///         "username": "alice",
///         "token": "abc123"
///     }))
///     .send()
///     .await?;
/// ```
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
///     .get("http://localhost:1313/servers/SID123/get_server_info")
///     .send()
///     .await?;
/// ```
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
/// Deletes a server entirely. Only the server owner is allowed to perform this action.
///
/// ### Request JSON (`TokenUser`)
/// ```json
/// {
///   "username": "alice",
///   "token": "abc123"
/// }
/// ```
///
/// ### Example (reqwest)
/// ```rust
/// let client = reqwest::Client::new();
/// let res = client
///     .post("http://localhost:1313/servers/SID123/delete_server")
///     .json(&serde_json::json!({
///         "username": "alice",
///         "token": "abc123"
///     }))
///     .send()
///     .await?;
/// ```
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

/// Checks whether the authenticated user is currently a member of the specified server.
///
/// ### Request JSON (`TokenUserServer`)
/// ```json
/// {
///   "username": "alice",
///   "token": "abc123",
///   "sid": "SID123"
/// }
/// ```
///
/// ### Example (reqwest)
/// ```rust
/// let client = reqwest::Client::new();
/// let res = client
///     .post("http://localhost:1313/am_i_in_server")
///     .json(&serde_json::json!({
///         "username": "alice",
///         "token": "abc123",
///         "sid": "SID123"
///     }))
///     .send()
///     .await?;
/// ```
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
