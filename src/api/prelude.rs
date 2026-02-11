use crate::security;
use crate::utils::logging;
use crate::db;
use crate::api::structures;
use moka::future::Cache;
use ::function_name::named;

#[named]
pub async fn check_user_password(
    secrets:Vec<db::structures::UserSecrets>,
    username: &str,
    password: &str,
    scylla_session: std::sync::MutexGuard<'_, scylla::client::session::Session>,
    cache: std::sync::MutexGuard<'_, Cache<std::string::String, std::string::String>>,
    new_token_holder: structures::TokenHolder
) -> actix_web::HttpResponse {
    
    let password_hash = secrets[0].password_hash.clone().unwrap();
    let user_salt = secrets[0].user_salt.clone().unwrap();
    let password_salt = secrets[0].password_salt.clone().unwrap();
    let decrypted_user_salt = security::aes::decrypt(&user_salt);
    let decrypted_password_salt = security::aes::decrypt(&password_salt);
    let user_password_hash =
        security::sha512(security::aes::encrypt(&security::aes::encrypt_with_key(
            &format!("{}{}", decrypted_user_salt.clone(), password),
            &decrypted_password_salt,
        )));
    
    if user_password_hash == password_hash {
        let _ = db::prelude::insert_user_token(
            &scylla_session,
            &cache,
            db::structures::KeyUser {
                key: Some(security::armor_token(new_token_holder.token.clone())),
                username: Some(username.to_string()),
            },
        )
        .await;

        actix_web::HttpResponse::Ok().json(&new_token_holder)
    } else {
        logging::log("Failed because user supplied password is incorrect.", Some(function_name!()));
        actix_web::HttpResponse::Unauthorized().body("Invalid username or password")
    }
}

macro_rules! scylla_session {
    ($session:ident) => {
        match $session.lock.lock() {
            Ok(guard) => guard,
            Err(_) => {
                return actix_web::HttpResponse::InternalServerError()
                    .body("Internal error: scylla session lock poisoned.");
            }
        }
    };
}

macro_rules! cache {
    ($shared_cache:ident) => {
        match $shared_cache.lock.lock() {
            Ok(guard) => guard,
            Err(_) => {
                return actix_web::HttpResponse::InternalServerError()
                    .body("Internal error: cache lock poisoned.");
            }
        }
    };
}

macro_rules! param {
    ($http:expr, $name:expr) => {
        match $http.match_info().get($name) {
            Some(param) => param.to_string(),
            None => {
                return actix_web::HttpResponse::BadRequest()
                    .body(format!("missing `{}` parameter", $name));
            }
        }
    };
}
