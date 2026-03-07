use crate::api::statics;
use crate::api::structures;
use crate::api::structures::{CreateChannel, TokenUser};
use crate::db;
use crate::security;
use crate::utils::logging;

use ::function_name::named;

#[named]
#[actix_web::post("/servers/{sid}/get_channels")]
pub async fn get_channels(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<TokenUser>,
    http: actix_web::HttpRequest,
    shared_collector: actix_web::web::Data<structures::AppState>,
) -> impl actix_web::Responder {
    
    let sid = param!(http, "sid");
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
        logging::log("SERVERS FAIL: invalid token in fetch_server_channels", Some(function_name!()));
        return actix_web::HttpResponse::Unauthorized().body("Invalid token or user not in server");
    }


    if let Some(channels) = db::server::fetch_server_channels(&scylla_session, sid).await {
        return actix_web::HttpResponse::Ok().json(&structures::Channels { c_list: channels });
    }
    
    logging::log("SERVERS FAIL: fetch_server_channels", Some(function_name!()));
    actix_web::HttpResponse::InternalServerError()
        .body("Failed to fetch server channels")
    
}

#[named]
#[actix_web::post("/servers/{sid}/create_channel")]
pub async fn create_channel(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<CreateChannel>,
    http: actix_web::HttpRequest,
    shared_collector: actix_web::web::Data<structures::AppState>,
) -> impl actix_web::Responder {
    if req.channel_name.len() > statics::MAX_CHANNEL_LENGTH {
        return actix_web::HttpResponse::LengthRequired().body(format!(
            "Failed to create channel: Channel name longer than {}",
            statics::MAX_CHANNEL_LENGTH
        ));
    }
    let scylla_session = scylla_session!(session);
    let sid = param!(http, "sid", &scylla_session);
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
        logging::log("SERVERS FAIL: invalid token in create_channel", Some(function_name!()));
        return actix_web::HttpResponse::Unauthorized().body("Invalid token or user not in server");
    }
    
    if db::server::create_channel(&scylla_session, sid, req.channel_name.clone()).await.is_none() {
        logging::log("SERVERS FAIL: create_channel", Some(function_name!()));
        return actix_web::HttpResponse::InternalServerError().body("Could not create channel");
    }
    
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

#[named]
#[actix_web::post("/servers/{sid}/{channel_name}/delete_channel")]
pub async fn delete_channel(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::TokenUser>,
    http: actix_web::HttpRequest,
    shared_collector: actix_web::web::Data<structures::AppState>,
) -> impl actix_web::Responder {

    let scylla_session = scylla_session!(session);
    let sid = param!(http, "sid", &scylla_session);
    let channel_name = param!(http, "channel_name", &scylla_session, sid);
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

    if db::server::check_user_is_owner(&scylla_session, sid.clone(), req.username.clone()).await != Some(true) {
        logging::log("Unauthorized: not server owner", Some(function_name!()));
        return actix_web::HttpResponse::Unauthorized()
            .body("You don't have permission to delete this channel");
    }
    
    if (db::server::delete_channel(&scylla_session, sid, channel_name).await).is_some() {
        return actix_web::HttpResponse::Ok().body("Channel deleted successfully");
    }
    actix_web::HttpResponse::InternalServerError().body("Failed to delete channel")
}
