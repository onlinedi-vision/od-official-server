#![allow(unused_imports)]
use crate::api::structures;
use crate::api::structures::{LimitMessageTokenUser, TokenHolder, TokenLoginUser, TokenUser};
use crate::db;
use crate::db::statics::SELECT_USERS_BY_ROLE;
use crate::security;
use std::clone;
use std::io::Write;

#[actix_web::post("/servers/{sid}/api/{channel_name}/get_messages")]
pub async fn get_channel_messages(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<TokenUser>,
    http: actix_web::HttpRequest,
) -> impl actix_web::Responder {
    let sid: String = http.match_info().get("sid").unwrap().to_string();
    let channel_name: String = http.match_info().get("channel_name").unwrap().to_string();
    let scylla_session = session.lock.lock().unwrap();
    let cache = shared_cache.lock.lock().unwrap();
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
            match db::messages::fetch_server_channel_messages(
                &scylla_session,
                sid.clone(),
                channel_name,
                None,
                None,
            )
            .await
            {
                Some(messages) => {
                    actix_web::HttpResponse::Ok()
                        .json(&structures::Messages { m_list: messages })
                }
                None => {
                    println!("SERVERS FAIL: fetch_server_channel_messages");
                    actix_web::HttpResponse::InternalServerError()
                        .body("Failed to fetch messages")
                }
            }
        }
        None => {
            println!("SERVERS FAIL: invalid token in fetch_server_channel_messages");
            actix_web::HttpResponse::Unauthorized().body("Invalid token or user not in server")
        }
    }
}

#[actix_web::post("/servers/{sid}/api/{channel_name}/get_messages_migration")]
pub async fn get_channel_messages_migration(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<LimitMessageTokenUser>,
    http: actix_web::HttpRequest,
) -> impl actix_web::Responder {
    let sid: String = http.match_info().get("sid").unwrap().to_string();
    let channel_name: String = http.match_info().get("channel_name").unwrap().to_string();

    let limit: usize = req.limit.clone().parse::<usize>().unwrap();
    let offset: usize = req.offset.clone().parse::<usize>().unwrap();

    let scylla_session = session.lock.lock().unwrap();
    let cache = shared_cache.lock.lock().unwrap();

    if db::prelude::check_user_is_in_server(
        &scylla_session,
        &cache,
        sid.clone(),
        req.token.clone(),
        req.username.clone(),
    )
    .await.is_some()
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

#[actix_web::post("/servers/{sid}/api/{channel_name}/send_message")]
pub async fn send_message(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::SendMessage>,
    http: actix_web::HttpRequest,
) -> impl actix_web::Responder {
    let sid: String = http.match_info().get("sid").unwrap().to_string();
    let channel_name: String = http.match_info().get("channel_name").unwrap().to_string();

    let scylla_session = session.lock.lock().unwrap();
    let cache = shared_cache.lock.lock().unwrap();
	
	if req.m_content.len() > db::statics::MAX_MESSAGE_LENGTH {
		return actix_web::HttpResponse::LengthRequired()
			.body(format!("Failed to send message: Message longer than {}", db::statics::MAX_MESSAGE_LENGTH));
		}
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
                    actix_web::HttpResponse::Ok()
                        .json(&structures::Messages { m_list: Vec::new() })
                }
                None => {
                    println!("FAILED AT SEND MESSAGE");
                    actix_web::HttpResponse::InternalServerError()
                        .body("Failed to send message")
                }
            }
        }
        None => {
            println!("FAILED AT USER IN SERVER");
            actix_web::HttpResponse::Unauthorized()
                .body("Invalid token or user not in server")
        }
    }
}

#[actix_web::post("/servers/{sid}/api/{channel_name}/delete_message")]
pub async fn delete_message(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::DeleteMessage>,
    http: actix_web::HttpRequest,
) -> impl actix_web::Responder {
    let scylla_session = session.lock.lock().unwrap();
    let cache = shared_cache.lock.lock().unwrap();

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

    let sid: String = http.match_info().get("sid").unwrap().to_string();
    let channel_name: String = http.match_info().get("channel_name").unwrap().to_string();

    let dt = chrono::NaiveDateTime::parse_from_str(&req.datetime, "%Y-%m-%d %H:%M:%S%.f").unwrap();
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
        .await.is_some()
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
