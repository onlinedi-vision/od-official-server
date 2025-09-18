#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use scylla::client::session::Session;

use crate::api::structures;
use crate::db;
use crate::security;

// !TODO: create_server API
#[actix_web::post("/api/create_server")]
pub async fn create_server(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::CreateServer>,
) -> impl actix_web::Responder {
    let scylla_session = session.lock.lock().unwrap();
    if let Some(_) = db::prelude::check_token(
        &scylla_session,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await
    {
        let sid = security::sid();
        if let Some(_) = db::server::create_server(
            &scylla_session,
            sid.clone(),
            &req.desc,
            &req.img_url,
            &req.name,
            req.username.clone(),
        )
        .await
        {
            let _ =
                db::server::create_channel(&scylla_session, sid.clone(), "info".to_string()).await;
            let new_token_holder = structures::TokenHolder {
                token: security::token(),
            };
            let _ = db::prelude::update_user_key(
                &scylla_session,
                db::structures::KeyUser {
                    key: Some(new_token_holder.token.clone()),
                    username: Some(req.username.clone()),
                },
            )
            .await;
            if let Some(_) =
                db::server::add_user_to_server(&scylla_session, sid, req.username.clone()).await
            {
                return actix_web::HttpResponse::Ok().json(&new_token_holder);
            } else {
                println!("SERVERS FAIL: add_user_to_server");
                return actix_web::HttpResponse::InternalServerError()
                    .body("Failed to add user to server");
            }
        } else {
            println!("SERVERS FAIL: create_server");
            return actix_web::HttpResponse::InternalServerError().body("Failed to create server");
        }
    } else {
        println!("SERVERS FAIL: invalid token in create_server");
        return actix_web::HttpResponse::Unauthorized().body("Invalid token");
    }
}

#[actix_web::post("/servers/{sid}/api/join")]
pub async fn join_server(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::TokenUser>,
    http: actix_web::HttpRequest,
) -> impl actix_web::Responder {
    let sid: String = http.match_info().get("sid").unwrap().to_string();
    let scylla_session = session.lock.lock().unwrap();
    if let Some(_) = db::prelude::check_token(
        &scylla_session,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await
    {
        if let Some(_) =
            db::server::add_user_to_server(&scylla_session, sid, req.username.clone()).await
        {
            let new_token_holder = structures::TokenHolder {
                token: security::token(),
            };
            let _ = db::prelude::update_user_key(
                &scylla_session,
                db::structures::KeyUser {
                    key: Some(new_token_holder.token.clone()),
                    username: Some(req.username.clone()),
                },
            )
            .await;

            return actix_web::HttpResponse::Ok().json(&new_token_holder);
        } else {
            println!("SERVERS FAIL: add_user_to_server");
            return actix_web::HttpResponse::InternalServerError()
                .body("Failed to add user to server");
        }
    } else {
        println!("SERVERS FAIL: invalid token in create_server");
        return actix_web::HttpResponse::Unauthorized().body("Invalid token");
    }
}

// !TODO: get_user_servers API
#[actix_web::post("/servers/{sid}/api/get_server_users")]
pub async fn get_server_users(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::TokenUser>,
    http: actix_web::HttpRequest,
) -> impl actix_web::Responder {
    let sid: String = http.match_info().get("sid").unwrap().to_string();
    let scylla_session = session.lock.lock().unwrap();
    if let Some(_) = db::prelude::check_user_is_in_server(
        &scylla_session,
        sid.clone(),
        req.token.clone(),
        req.username.clone(),
    )
    .await
    {
        if let Some(users) = db::server::fetch_server_users(&scylla_session, sid.clone()).await {
            return actix_web::HttpResponse::Ok().json(&structures::UsersList { u_list: users });
        } else {
            return actix_web::HttpResponse::Ok()
                .json(&structures::UsersList { u_list: Vec::new() });
        }
    } else {
        return actix_web::HttpResponse::Unauthorized().body("Invalid token or user not in server");
    }
}

#[actix_web::get("/servers/{sid}/api/get_server_info")]
pub async fn get_server_info(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    http: actix_web::HttpRequest,
) -> impl actix_web::Responder {
    let sid: String = http.match_info().get("sid").unwrap().to_string();
    let scylla_session = session.lock.lock().unwrap();
    if let Some(server_info) = db::server::fetch_server_info(&scylla_session, sid.clone()).await {
        return actix_web::HttpResponse::Ok().json(&server_info);
    } else {
        return actix_web::HttpResponse::NotFound().json(&structures::Status {
            status: "nok".to_string(),
        });
    }
}
