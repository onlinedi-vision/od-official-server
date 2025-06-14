use crate::security;
use crate::api::structures;
use base64::prelude::*;

#[actix_web::post("/api/cdn")]
pub async fn save_file (
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::File>,
    http: actix_web::HttpRequest
) -> impl actix_web::Responder {

    if let Some(homedir) = std::env::home_dir() {
        let new_dir_token = security::token();
        let new_file_token = format!("{}/{}", new_dir_token.clone(), security::token());
        let new_dir = format!("{}/cdn/{}", homedir.display(), new_dir_token.clone());
        if let Ok(_) = std::fs::create_dir_all(new_dir.clone().as_str()) {
            if let Ok(v) = BASE64_STANDARD.decode(req.cont.clone()) {
            let pload = String::from_utf8(v).expect("Invalid");
            if let Ok(_) = std::fs::write(format!("{}/cdn/{}", homedir.display(), new_file_token.clone()), pload) {
                return actix_web::HttpResponse::Ok().json(
                    &structures::FileURL {
                        url: format!("https://onlinedi.vision/cdn/{}", new_file_token)
                    }
                );
            }
            }
            return actix_web::HttpResponse::Ok().json(
                &structures::FileURL {
                    url: "write".to_string()
                }
            );
        }
        
        return actix_web::HttpResponse::Ok().json(
            &structures::FileURL {
                url: "create dir".to_string()
            }
        );
    }
    return actix_web::HttpResponse::Ok().json(
        &structures::FileURL {
            url: "home dir".to_string()
        }
    );

}
