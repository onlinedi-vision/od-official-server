use crate::api::structures;
use crate::api::structures::TokenHolder;
use crate::security;
use crate::db;

#[actix_web::post("/servers/{sid}/api/get_channels")]
pub async fn get_channels (
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<TokenHolder>,
    http: actix_web::HttpRequest
) -> impl actix_web::Responder {
    let sid: String = http.match_info().get("sid").unwrap().to_string();
    let scylla_session = session.lock.lock().unwrap();
    match db::check_token(&scylla_session, req.token.clone(), None).await {
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


