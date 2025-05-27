#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use crate::api::structures;
use crate::security;
use crate::db;

// !TODO: create_server API
#[actix_web::post("/api/create_server")] 
pub async fn create_server(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::SendMessage>
) ->impl actix_web::Responder {
    actix_web::HttpResponse::Ok().json(
        &structures::Messages {
            m_list: Vec::new()
        }
    )
}

// !TODO: get_user_servers API
#[actix_web::post("/api/get_user_servers")] 
pub async fn get_user_servers(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::SendMessage>
) ->impl actix_web::Responder {
    actix_web::HttpResponse::Ok().json(
        &structures::Messages {
            m_list: Vec::new()
        }
    )
}

