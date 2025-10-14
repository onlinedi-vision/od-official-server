#![allow(unused_imports)]
use crate::api::structures;
use crate::api::structures::{
    TokenHolder,
    TokenLoginUser,
    TokenUser,
    LimitMessageTokenUser
};
use std::io::Write;
use crate::security;
use crate::db;

#[actix_web::post("/servers/{sid}/api/{channel_name}/get_messages")] 
pub async fn get_channel_messages(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<TokenUser>,
    http: actix_web::HttpRequest
) -> impl actix_web::Responder {
    let sid: String = http.match_info().get("sid").unwrap().to_string();
    let channel_name: String = http.match_info().get("channel_name").unwrap().to_string();
    let scylla_session = session.lock.lock().unwrap();
    match db::prelude::check_user_is_in_server(&scylla_session, sid.clone(), req.token.clone(), req.username.clone()).await {
        Some(_) => {
            match db::messages::fetch_server_channel_messages(&scylla_session, sid.clone(), channel_name, None, None).await {
                Some(messages) => {
                        return actix_web::HttpResponse::Ok().json(
                            &structures::Messages {
                                m_list: messages
                            }
                        );
                },
                None => {
                    println!("SERVERS FAIL: fetch_server_channel_messages");
                    return actix_web::HttpResponse::InternalServerError().body("Failed to fetch messages");
                }
            }

        },
        None => {
            println!("SERVERS FAIL: invalid token in fetch_server_channel_messages");
            actix_web::HttpResponse::Unauthorized().body("Invalid token or user not in server")
        }
    }
}

#[actix_web::post("/servers/{sid}/api/{channel_name}/get_messages_migration")] 
pub async fn get_channel_messages_migration(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<LimitMessageTokenUser>,
    http: actix_web::HttpRequest
) -> impl actix_web::Responder {
    let sid: String = http.match_info().get("sid").unwrap().to_string();
    let channel_name: String = http.match_info().get("channel_name").unwrap().to_string();
    let limit: usize = req.limit.clone().parse::<usize>().unwrap();
    let offset: usize = req.offset.clone().parse::<usize>().unwrap();
    let scylla_session = session.lock.lock().unwrap();
    if let Some(_) = db::prelude::check_user_is_in_server(&scylla_session, sid.clone(), req.token.clone(), req.username.clone()).await {
        if let Some(messages) = db::messages::fetch_server_channel_messages(&scylla_session, sid.clone(), channel_name, Some(limit), Some(offset)).await {
            return actix_web::HttpResponse::Ok().json(
                &structures::Messages {
                    m_list: messages
                }
            );
        } else {
            println!("SERVERS FAIL: fetch_server_channel_messages");
            return actix_web::HttpResponse::InternalServerError().body("Failed to fetch messages");
        }
    } else {
        println!("SERVERS FAIL: invalid token in fetch_server_channel_messages");
        actix_web::HttpResponse::Unauthorized().body("Invalid token or user not in server")
    }
}

#[actix_web::post("/servers/{sid}/api/{channel_name}/send_message")] 
pub async fn send_message(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::SendMessage>,
    http: actix_web::HttpRequest
) -> impl actix_web::Responder {
    let sid: String = http.match_info().get("sid").unwrap().to_string();
    let channel_name: String = http.match_info().get("channel_name").unwrap().to_string();
    let scylla_session = session.lock.lock().unwrap();
    match db::prelude::check_user_is_in_server(&scylla_session, sid.clone(), req.token.clone(), req.username.clone()).await {
        Some(_) => {
            match db::server::send_message(
                &scylla_session,
                sid.clone(),
                channel_name.clone(),
                req.m_content.clone(),
                req.username.clone()
            ).await {
                Some(_) => {
                   return actix_web::HttpResponse::Ok().json(
                        &structures::Messages {
                            m_list: Vec::new()
                        }
                    );       
                },
                None => {
                    println!("FAILED AT SEND MESSAGE");
                    return actix_web::HttpResponse::InternalServerError().body("Failed to send message");
                }
            }
        },
        None => {
            println!("FAILED AT USER IN SERVER");
            return actix_web::HttpResponse::Unauthorized().body("Invalid token or user not in server");
        }
    }
}

#[actix_web::post("/servers/{sid}/api/{channel_name}/delete_message")]
pub async fn delete_message(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::DeleteMessage>,
    http: actix_web::HttpRequest
) -> impl actix_web::Responder {

    let scylla_session = session.lock.lock().unwrap();
    if db::prelude::check_token(
            &scylla_session,
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
    
    if req.owner == req.username 
    || db::server::check_user_is_owner(&scylla_session, sid.clone(), req.username.clone()).await == Some(true) 
    || db::roles::fetch_user_roles(&scylla_session, sid.clone(), req.username.clone()).await == Some(vec!["Admin".to_string()]) 
    {
        if let Some(_) = db::messages::delete_message(
            &scylla_session,
            sid.clone(),
            req.datetime.clone(),
            channel_name.clone()
        ).await {
            return actix_web::HttpResponse::Ok().body("Message deleted successfully");
        } 
        else {
            return actix_web::HttpResponse::InternalServerError().body("Failed to delete message");
        }
    } 
    else {
        println!("Unauthorized: not message owner or server owner or admin");
        return actix_web::HttpResponse::Unauthorized().body("You are not authorized to delete this message");
    }
}
