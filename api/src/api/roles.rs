#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use crate::api::structures;
use crate::db;
use crate::security;

#[actix_web::post("/api/add_server_role")]
pub async fn add_server_role(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::ServerRoleRequest>,
) -> impl actix_web::Responder {
    let scylla_session = session.lock.lock().unwrap();

    if let Some(_) = db::prelude::check_token(
        &scylla_session,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await
    {
        //trigger
        let role = db::structures::ServerRole {
            role_name: req.role_name.clone(),
            server_id: req.server_id.clone(),
            color: req.color.clone(),
            permissions: req
                .permissions
                .clone()
                .unwrap_or_default()
                .into_iter()
                .collect::<std::collections::HashSet<String>>(),
        };

        if let Some(result) =
            db::roles::insert_server_role(&scylla_session, req.server_id.clone(), role).await
        {
            return match result {
                Ok(_) => actix_web::HttpResponse::Ok().body("Role added successfully"),
                Err(err) => {
                    println!("Error inserting role: {:?}", err);
                    actix_web::HttpResponse::InternalServerError().body("Failed to insert role")
                }
            };
        }
    }

    actix_web::HttpResponse::Unauthorized().body("Invalid token")
}


#[actix_web::post("/api/assign_role_to_user")]
pub async fn assign_role_to_user(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::UserServerRoleRequest>,
    
) -> impl actix_web::Responder {
    let scylla_session = session.lock.lock().unwrap();

    if let Some(_) = db::prelude::check_token(
        &scylla_session,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await
    {
        let user_role = db::structures::UserServerRole{
            server_id: req.server_id.clone(),
            username: req.username.clone(),
            role_name: req.role_name.clone(),
        };
        
        if let Some(result) = db::roles::assign_role_to_user(&scylla_session, user_role).await {
            return match result {
                Ok(_) => actix_web::HttpResponse::Ok().body("Role assigned successfully"),
                Err(err) => {
                    println!("Error assigning role: {:?}", err);
                    actix_web::HttpResponse::InternalServerError().body("Failed to assign role")
                }
            };
        }
    }

    actix_web::HttpResponse::Unauthorized().body("Invalid token")
}


#[actix_web::post("/api/remove_role_from_user")]
pub async fn remove_role_from_user(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::UserServerRoleRequest>,
) -> impl actix_web::Responder {
    let scylla_session = session.lock.lock().unwrap();

    if let Some(_) = db::prelude::check_token(
        &scylla_session,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await
    {
        
        let user_role = db::structures::UserServerRole{
            server_id: req.server_id.clone(),
            username: req.username.clone(),
            role_name: req.role_name.clone(),
        };
        
        if let Some(result) = db::roles::remove_role_from_user(&scylla_session, user_role).await
        {
            return match result {
                Ok(_) => actix_web::HttpResponse::Ok().body("Role removed successfully"),
                Err(err) => {
                    println!("Error removing role: {:?}", err);
                    actix_web::HttpResponse::InternalServerError().body("Failed to remove role")
                }
            };
        }
    }
    actix_web::HttpResponse::Unauthorized().body("Invalid token")
}


#[actix_web::get("/api/fetch_server_roles")]
pub async fn fetch_server_roles(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    query: actix_web::web::Query<structures::ServerRoleQuery>,
) -> impl actix_web::Responder {
    let scylla_session = session.lock.lock().unwrap();

    if let Some(_) = db::prelude::check_token(
        &scylla_session,
        query.token.clone(),
        Some(query.username.clone()),
    )
    .await
    {
        if let Some(roles) =
            db::roles::fetch_server_roles(&scylla_session, &query.server_id).await
        {
            return actix_web::HttpResponse::Ok().json(roles);
        }
        return actix_web::HttpResponse::InternalServerError().body("Failed to fetch server roles");
    }

    actix_web::HttpResponse::Unauthorized().body("Invalid Token")
}


#[actix_web::get("/api/fetch_user_roles")]
pub async fn fetch_user_roles(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    query: actix_web::web::Query<structures::UserRoleQuery>,
) -> impl actix_web::Responder {
    let scylla_session = session.lock.lock().unwrap();

    if let Some(_) = db::prelude::check_token(
        &scylla_session,
        query.token.clone(),
        Some(query.username.clone()),
    )
    .await
    {
        if let Some(roles) = db::roles::fetch_user_roles(
            &scylla_session,
            query.server_id.clone(),
            query.username.clone(),
        )
        .await
        {
            return actix_web::HttpResponse::Ok().json(roles);
        }
        return actix_web::HttpResponse::InternalServerError().body("Failed to fetch user roles");
    }

    actix_web::HttpResponse::Unauthorized().body("Invalid Token")
}
