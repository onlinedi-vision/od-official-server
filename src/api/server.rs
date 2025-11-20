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
    shared_cache: actix_web::web::Data<security::structures::MokaCache>, 
    req: actix_web::web::Json<structures::CreateServer>,
) -> impl actix_web::Responder {
    
    let scylla_session = session.lock.lock().unwrap();
    let cache = shared_cache.lock.lock().unwrap();

    if let Some(_) = db::prelude::check_token(
        &scylla_session,
        &cache,
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
            let _ = db::prelude::insert_user_token(
                &scylla_session,
                &cache,
                db::structures::KeyUser {
                    key: Some(security::armor_token(new_token_holder.token.clone())),
                    username: Some(req.username.clone()),
                },
            )
            .await;

            let role = db::structures::ServerRole {
                role_name: "member".to_string(),
                server_id: sid.clone(),
                color: Some("#807675".to_string()),
                permissions: std::collections::HashSet::<String>::new(),
            };

            let _  = db::roles::insert_server_role(&scylla_session, sid.clone(), role).await;
            
            let user_role = db::structures::UserServerRole {
                server_id: sid.clone(),
                username: req.username.clone(),
                role_name: "member".to_string(),
            };
            let _ = db::roles::assign_role_to_user(&scylla_session, user_role).await;            

            if let Some(_) =
                db::server::add_user_to_server(&scylla_session, sid, req.username.clone()).await
            {
                actix_web::HttpResponse::Ok().json(&new_token_holder)
            } else {
                println!("SERVERS FAIL: add_user_to_server");
                actix_web::HttpResponse::InternalServerError()
                    .body("Failed to add user to server")
            }
        } else {
            println!("SERVERS FAIL: create_server");
            actix_web::HttpResponse::InternalServerError().body("Failed to create server")
        }
    } else {
        println!("SERVERS FAIL: invalid token in create_server");
        actix_web::HttpResponse::Unauthorized().body("Invalid token")
    }
}

#[actix_web::post("/servers/{sid}/api/join")]
pub async fn join_server(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>, 
    req: actix_web::web::Json<structures::TokenUser>,
    http: actix_web::HttpRequest,
) -> impl actix_web::Responder {

    let sid: String = http.match_info().get("sid").unwrap().to_string();
    let scylla_session = session.lock.lock().unwrap();
    let cache = shared_cache.lock.lock().unwrap();

    if let Some(_) = db::prelude::check_token(
        &scylla_session,
        &cache,
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
        } else {
            println!("SERVERS FAIL: add_user_to_server");
            actix_web::HttpResponse::InternalServerError()
                .body("Failed to add user to server")
        }
    } else {
        println!("SERVERS FAIL: invalid token in create_server");
        actix_web::HttpResponse::Unauthorized().body("Invalid token")
    }
}

// !TODO: get_user_servers API
#[actix_web::post("/servers/{sid}/api/get_server_users")]
pub async fn get_server_users(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>, 
    req: actix_web::web::Json<structures::TokenUser>,
    http: actix_web::HttpRequest,
) -> impl actix_web::Responder {

    let sid: String = http.match_info().get("sid").unwrap().to_string();
    let scylla_session = session.lock.lock().unwrap();
    let cache = shared_cache.lock.lock().unwrap();
    
    if let Some(_) = db::prelude::check_user_is_in_server(
        &scylla_session,
        &cache,
        sid.clone(),
        req.token.clone(),
        req.username.clone(),
    )
    .await
    {
        if let Some(users) = db::server::fetch_server_users(&scylla_session, sid.clone()).await {
            actix_web::HttpResponse::Ok().json(&structures::UsersList { u_list: users })
        } else {
            actix_web::HttpResponse::Ok()
                .json(&structures::UsersList { u_list: Vec::new() })
        }
    } else {
        actix_web::HttpResponse::Unauthorized().body("Invalid token or user not in server")
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
        actix_web::HttpResponse::Ok().json(&server_info)
    } else {
        actix_web::HttpResponse::NotFound().json(&structures::Status {
            status: "nok".to_string(),
        })
    }
}

#[actix_web::post("/servers/{sid}/api/delete_server")]
pub async fn delete_server(
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

    if db::server::check_user_is_owner(&scylla_session, sid.clone(), req.username.clone()).await == Some(true) {
        if let Some(_) = db::server::delete_server(&scylla_session, sid).await {
            actix_web::HttpResponse::Ok().body("Server deleted successfully")
        } 
        else {
            actix_web::HttpResponse::InternalServerError().body("Failed to delete server")
        }
    } 
    else {
        println!("Unauthorized: not server owner");
        actix_web::HttpResponse::Unauthorized().body("You don't have permission to delete this server")
    }
}
