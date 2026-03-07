use crate::api::statics;
use crate::api::structures;
use crate::api::structures::LimitMessageTokenUser;
use crate::db;
use crate::security;
use crate::utils::logging;

use ::function_name::named;

#[named]
#[actix_web::post("/servers/{sid}/{channel_name}/get_messages_migration")]
pub async fn get_channel_messages_migration(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<LimitMessageTokenUser>,
    http: actix_web::HttpRequest,
) -> impl actix_web::Responder {
    let sid = param!(http, "sid");
    let channel_name = param!(http, "channel_name");

    let limit: usize = match req.limit.parse::<usize>() {
        Ok(value) => value,
        Err(_) => {
            return actix_web::HttpResponse::BadRequest()
                .body("invalid `limit` (must be positive integer)");
        }
    };
    let offset: usize = match req.offset.parse::<usize>() {
        Ok(value) => value,
        Err(_) => {
            return actix_web::HttpResponse::BadRequest()
                .body("invalid `offset` (must be positive integer)");
        }
    };

    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);

    if db::prelude::check_user_is_in_server(
        &scylla_session,
        &cache,
        sid.clone(),
        req.token.clone(),
        req.username.clone(),
    )
    .await
    .is_none()
    {
        logging::log("SERVERS FAIL: invalid token in fetch_server_channel_messages", Some(function_name!()));
        return actix_web::HttpResponse::Unauthorized().body("Invalid token or user not in server");
    }
    
    if let Some(messages) = db::messages::fetch_server_channel_messages(
        &scylla_session,
        sid.clone(),
        channel_name,
        Some(limit),
        Some(offset),
    )
    .await
    {
        return actix_web::HttpResponse::Ok().json(&structures::Messages { m_list: messages });
    }
    
    logging::log("SERVERS FAIL: fetch_server_channel_messages", Some(function_name!()));
    actix_web::HttpResponse::InternalServerError().body("Failed to fetch messages")
}

#[named]
#[actix_web::post("/servers/{sid}/{channel_name}/send_message")]
pub async fn send_message(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::SendMessage>,
    http: actix_web::HttpRequest,
) -> impl actix_web::Responder {
	if req.m_content.len() > statics::MAX_MESSAGE_LENGTH {
		return actix_web::HttpResponse::LengthRequired()
			.body(format!("Failed to send message: Message longer than {}", statics::MAX_MESSAGE_LENGTH));
	}
	
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    let sid = param!(http, "sid", &scylla_session);
    let channel_name = param!(http, "channel_name", &scylla_session, sid);

    if db::prelude::check_permission(
        &scylla_session,
        &cache,
        sid.clone(),
        req.token.clone(),
        req.username.clone(),
        db::structures::Permissions::SEND_MESSAGES.bits(),
    )
    .await
    .is_none()
    {
        logging::log("FAILED PERMISSION CHECK", Some(function_name!()));
        return actix_web::HttpResponse::Forbidden().body("You do not have permission to send messages");
    }
    
    let ttl = db::users::get_ttl(&scylla_session, req.username.clone())
        .await;

    let (enc_message, enc_salt) =
        security::messages::encrypt(&req.m_content, &security::salt());
    
    if db::server::send_message(
        &scylla_session,
        sid.clone(),
        channel_name.clone(),
        enc_message,
        req.username.clone(),
        enc_salt,
        ttl,
    )
    .await
    .is_err()
    {
        logging::log("FAILED AT SEND MESSAGE", Some(function_name!()));
        return actix_web::HttpResponse::InternalServerError().body("Failed to send message");
    }
    actix_web::HttpResponse::Ok().body("Message sent.")
}

// TODO: what happens if invalid datetime tag is given?
//     - currently a e2e test fails here... this will need to be investigated at some point
//     - it seems that when the datetime is invalid the API doesn't fail with a proper message but instead says
//       "Message deleted succesfully" without anything actually happening...
#[named]
#[actix_web::post("/servers/{sid}/{channel_name}/delete_message")]
pub async fn delete_message(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::DeleteMessage>,
    http: actix_web::HttpRequest,
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

    let sid = param!(http, "sid");
    let channel_name = param!(http, "channel_name");

    let Ok(dt) = chrono::NaiveDateTime::parse_from_str(&req.datetime, "%Y-%m-%d %H:%M:%S%.f") else {
        return actix_web::HttpResponse::BadRequest()
            .body("invalid `datetime` format, expected `%Y-%m-%d %H:%M:%S%.f`");
    };

    let millis = dt.and_utc().timestamp_millis();
    let cql_datetime = scylla::value::CqlTimestamp(millis);

    if db::messages::verify_message_ownership(
        &scylla_session,
        sid.clone(),
        channel_name.clone(),
        cql_datetime,
        req.username.clone(),
    )
    .await
        == Some(true)
        || db::server::check_user_is_owner(&scylla_session, sid.clone(), req.username.clone()).await
            == Some(true)
    {
        if db::messages::delete_message(
            &scylla_session,
            sid.clone(),
            cql_datetime,
            channel_name.clone(),
        )
        .await
        .is_some()
        {
            return actix_web::HttpResponse::Ok().body("Message deleted successfully");
        }
        return actix_web::HttpResponse::InternalServerError().body("Failed to delete message");
    }
    logging::log("Unauthorized: not message owner or server owner", Some(function_name!()));
    actix_web::HttpResponse::Unauthorized()
        .body("You are not authorized to delete this message")
}
