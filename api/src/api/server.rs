#![allow(dead_code)]
#![allow(unused_imports)]
use crate::api::structures;
use crate::db;

#[actix_web::post("/api/create_server")] 
pub async fn create_server() -> impl actix_web::Responder {
    actix_web::HttpResponse::Ok().json(
        &structures::Messages {
            m_list: Vec::new()
        }
    )
}
