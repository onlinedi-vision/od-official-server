use crate::api::structures;
use crate::api::structures::{
    TokenHolder,
    CreateChannel,
    TokenUser
};
use crate::security;
use crate::db;

#[actix_web::post("/servers/{sid}/api/get_channels")]
pub async fn get_channels (
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<TokenUser>,
    http: actix_web::HttpRequest
) -> impl actix_web::Responder {
    let sid: String = http.match_info().get("sid").unwrap().to_string();
    let scylla_session = session.lock.lock().unwrap();
    match db::prelude::check_user_is_in_server(&scylla_session, sid.clone(), req.token.clone(), req.username.clone()).await {
        Some(_) => {
            match db::fetch_server_channels(&scylla_session, sid).await {
                Some(channels) => {
                        return actix_web::HttpResponse::Ok().json(
                            &structures::Channels {
                                c_list: channels
                            }
                        );
                },
                None => {
                    println!("SERVERS FAIL: fetch_server_channels");
                    return actix_web::HttpResponse::Ok().json(
                        &structures::Channels {
                            c_list: Vec::new()
                        }
                    );
                }
            }
        },
        _ => {
            println!("SERVERS FAIL: invalid token in fetch_server_channels");
            return actix_web::HttpResponse::Ok().json(
                &structures::Channels {
                    c_list: Vec::new()
                }
            );
        }
    };
}

#[actix_web::post("/servers/{sid}/api/create_channel")]
pub async fn create_channel (
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<CreateChannel>,
    http: actix_web::HttpRequest
) -> impl actix_web::Responder {

    let sid: String = http.match_info().get("sid").unwrap().to_string();
    let scylla_session = session.lock.lock().unwrap();
    match db::prelude::check_user_is_in_server(&scylla_session, sid.clone(), req.token.clone(), req.username.clone()).await {
        Some(_) => {
            
            match db::server::create_channel(&scylla_session, sid, req.channel_name.clone()).await {
                Some(_) => {
                    let new_token_holder = structures::TokenHolder {
                        token: security::token()
                    };

                    let _ = db::prelude::update_user_key(
                        &scylla_session, 
                        db::structures::KeyUser{
                            key: Some(new_token_holder.token.clone()), 
                            username: Some(req.username.clone())
                        }
                    ).await;
                    return actix_web::HttpResponse::Ok().json(
                        &new_token_holder
                    );
                },
                None => {
                    println!("SERVERS FAIL: create_channel");
                    return actix_web::HttpResponse::Ok().json(
                        &structures::TokenHolder {
                            token: "".to_string()
                        }
                    );
                }
            }
        },
        _ => {
            println!("SERVERS FAIL: invalid token in create_channel");
            return actix_web::HttpResponse::Ok().json(
                &structures::TokenHolder {
                    token: "".to_string()
                }
            );
        }

    }
}
