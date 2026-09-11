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

    if db0::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.username.clone()),
        &collector,
    )
    .await.is_none()
    {
        logging::log(
            &format!("Invalid token when attempting to patch server({})'s frotend configuration...", sid.clone()),
            Some(function_name!()),
        );
        return actix_web::HttpResponse::Unauthorized().body("Invalid token!");
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
