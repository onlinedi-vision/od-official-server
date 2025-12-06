#![allow(unused_variables)]

/* !TODO:
 *  -- check out passing secrets with GET requests (to replace weird POST request implementation)
 * */
#[macro_use]
pub mod prelude;

pub mod channel;
pub mod friends;
pub mod invites;
pub mod message;
pub mod roles;
pub mod server;
pub mod spell_caster;
pub mod structures;
pub mod user;
use chrono::prelude::*;

#[actix_web::get("/api/version")]
pub async fn get_api_version() -> impl actix_web::Responder {
    actix_web::HttpResponse::Ok().body("v0.0.1".to_string())
}

#[actix_web::get("/api/time")]
pub async fn get_api_time() -> impl actix_web::Responder {
    actix_web::HttpResponse::Ok().body(format!("{:?}", Local::now()))
}
