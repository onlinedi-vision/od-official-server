use crate::api::structures;
use crate::api::structures::{
    TokenHolder,
    TokenLoginUser,
    TokenUser
};
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
            match db::fetch_server_channel_messages(&scylla_session, sid.clone(), channel_name).await {
                Some(messages) => {
                        return actix_web::HttpResponse::Ok().json(
                            &structures::Messages {
                                m_list: messages
                            }
                        );
                },
                None => {
                    println!("SERVERS FAIL: fetch_server_channel_messages");
                    return actix_web::HttpResponse::Ok().json(
                        &structures::Messages {
                            m_list: Vec::new()
                        }
                    );
                }
            }

        },
        None => {
            println!("SERVERS FAIL: invalid token in fetch_server_channel_messages");
            actix_web::HttpResponse::Ok().json(
                &structures::Messages {
                    m_list: Vec::new()
                }
            )
        }
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
                sid,
                channel_name,
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
                _ => {
                    return actix_web::HttpResponse::Ok().json(
                        &structures::Messages {
                            m_list: Vec::new()
                        }
                    );
                }
            }
        },
        _ => {
            return actix_web::HttpResponse::Ok().json(
                &structures::Messages {
                    m_list: Vec::new()
                }
            );
        }
    }
}


