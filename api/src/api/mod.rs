#![allow(unused_variables)]

/* !TODO: 
 *  -- check out passing secrets with GET requests (to replace weird POST request implementation)
 * */

mod structures;
pub mod message;
pub mod channel;
pub mod user;
pub mod server;

#[actix_web::get("/api/version")] 
pub async fn get_api_version() ->impl actix_web::Responder {
  return actix_web::HttpResponse::Ok().body(
    "v0.0.1".to_string()
  );
  
}

