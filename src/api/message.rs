#![allow(unused_imports)]
use actix_web::guard;

use crate::api::structures;
use crate::api::structures::{LimitMessageTokenUser, TokenHolder, TokenLoginUser, TokenUser};
use crate::api::statics;
use crate::db;
use crate::db::statics::SELECT_USERS_BY_ROLE;
use crate::security;
use std::clone;
use std::io::Write;

#[actix_web::post("/servers/{sid}/api/{channel_name}/get_messages_migration")]
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
    .is_some()
    {
        if let Some(messages) = db::messages::fetch_server_channel_messages(
            &scylla_session,
            sid.clone(),
            channel_name,
            Some(limit),
            Some(offset),
        )
        .await
        {
            actix_web::HttpResponse::Ok().json(&structures::Messages { m_list: messages })
        } else {
            println!("SERVERS FAIL: fetch_server_channel_messages");
            actix_web::HttpResponse::InternalServerError().body("Failed to fetch messages")
        }
    } else {
        println!("SERVERS FAIL: invalid token in fetch_server_channel_messages");
        actix_web::HttpResponse::Unauthorized().body("Invalid token or user not in server")
    }
}

// TODO: what happens if channel/server doesn't exist:
//     - it seems you can send messages to *things* that don't exist
#[actix_web::post("/servers/{sid}/api/{channel_name}/send_message")]
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
    let sid = param!(http, "sid");
    let channel_name = param!(http, "channel_name");

    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);

    match db::prelude::check_user_is_in_server(
    
        &scylla_session,
        &cache,
        sid.clone(),
        req.token.clone(),
        req.username.clone(),
    )
    .await
    {
        Some(_) => {
            let (enc_message, enc_salt) =
                security::messages::encrypt(&req.m_content, &security::salt());
            match db::server::send_message(
                &scylla_session,
                sid.clone(),
                channel_name.clone(),
                enc_message,
                req.username.clone(),
                enc_salt,
            )
            .await
            {
                Some(_) => {
                    actix_web::HttpResponse::Ok().json(&structures::Messages { m_list: Vec::new() })
                }
                None => {
                    println!("FAILED AT SEND MESSAGE");
                    actix_web::HttpResponse::InternalServerError().body("Failed to send message")
                }
            }
        }
        None => {
            println!("FAILED AT USER IN SERVER");
            actix_web::HttpResponse::Unauthorized().body("Invalid token or user not in server")
        }
    }
}

// TODO: what happens if invalid datetime tag is given?
//     - currently a e2e test fails here... this will need to be investigated at some point
//     - it seems that when the datetime is invalid the API doesn't fail with a proper message but instead says
//       "Message deleted succesfully" without anything actually happening...
#[actix_web::post("/servers/{sid}/api/{channel_name}/delete_message")]
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

    let dt = match chrono::NaiveDateTime::parse_from_str(&req.datetime, "%Y-%m-%d %H:%M:%S%.f") {
        Ok(dt) => dt,
        Err(_) => {
            return actix_web::HttpResponse::BadRequest()
                .body("invalid `datetime` format, expected `%Y-%m-%d %H:%M:%S%.f`");
        }
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
            actix_web::HttpResponse::Ok().body("Message deleted successfully")
        } else {
            actix_web::HttpResponse::InternalServerError().body("Failed to delete message")
        }
    } else {
        println!("Unauthorized: not message owner or server owner");
        actix_web::HttpResponse::Unauthorized()
            .body("You are not authorized to delete this message")
    }
}
