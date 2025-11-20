#![allow(unused_imports)]
use crate::api::structures;
use crate::api::structures::{CreateChannel, TokenHolder, TokenUser};
use crate::db;
use crate::security;

#[actix_web::post("/servers/{sid}/api/get_channels")]
pub async fn get_channels(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>, 
    req: actix_web::web::Json<TokenUser>,
    http: actix_web::HttpRequest,
) -> impl actix_web::Responder {

    let sid: String = http.match_info().get("sid").unwrap().to_string();
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
        Some(_) => match db::server::fetch_server_channels(&scylla_session, sid).await {
            Some(channels) => {
                return actix_web::HttpResponse::Ok()
                    .json(&structures::Channels { c_list: channels });
            }
            None => {
                println!("SERVERS FAIL: fetch_server_channels");
                return actix_web::HttpResponse::InternalServerError()
                    .body("Failed to fetch server channels");
            }
        },
        None => {
            println!("SERVERS FAIL: invalid token in fetch_server_channels");
            actix_web::HttpResponse::Unauthorized()
                .body("Invalid token or user not in server")
        }
    }
}

#[actix_web::post("/servers/{sid}/api/create_channel")]
pub async fn create_channel(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>, 
    req: actix_web::web::Json<CreateChannel>,
    http: actix_web::HttpRequest,
) -> impl actix_web::Responder {
    let sid: String = http.match_info().get("sid").unwrap().to_string();
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
            match db::server::create_channel(&scylla_session, sid, req.channel_name.clone()).await {
                Some(_) => {
                    let new_token_holder = structures::TokenHolder {
                        token: security::token(),
                    };

                    let _ = db::prelude::insert_user_token(
                        &scylla_session,
                        &cache,
                        db::structures::KeyUser {
                            key: Some(security::armor_token(new_token_holder.token.clone())),
                            username: Some(req.username.clone()),
                        },
                    )
                    .await;

                    let _ = db::users::delete_token(
                        &scylla_session,
                        req.username.clone(),
                        security::armor_token(req.token.clone()),
                    )
                    .await;

                    actix_web::HttpResponse::Ok().json(&new_token_holder)
                }
                None => {
                    println!("SERVERS FAIL: create_channel");
                    actix_web::HttpResponse::InternalServerError()
                        .body("Could not create channel")
                }
            }
        }
        None => {
            println!("SERVERS FAIL: invalid token in create_channel");
            actix_web::HttpResponse::Unauthorized()
                .body("Invalid token or user not in server")
        }
    }
}


#[actix_web::post("/servers/{sid}/api/{channel_name}/delete_channel")]
pub async fn delete_channel(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>, 
    req: actix_web::web::Json<structures::TokenUser>,
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

    

    if db::server::check_user_is_owner(&scylla_session, sid.clone(), req.username.clone()).await == Some(true) 
    {
        if let Some(_) = db::server::delete_channel(&scylla_session, sid, channel_name).await {
            actix_web::HttpResponse::Ok().body("Channel deleted successfully")
        } 
        else {
            actix_web::HttpResponse::InternalServerError().body("Failed to delete channel")
        }
    } 
    else {
        println!("Unauthorized: not server owner");
        actix_web::HttpResponse::Unauthorized().body("You don't have permission to delete this channel")
    }
}
