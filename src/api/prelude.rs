use crate::security;
use crate::utils::logging;
use crate::db;
use crate::api::structures;
use moka::future::Cache;
use ::function_name::named;

macro_rules! armor_token_or {
    ($token:expr, $response:expr) => {
        match crate::security::armor_token_logged($token) {
            Some(armored) => armored,
            None => return $response,
        }
    };
}

macro_rules! cache_metrics {
    ($col:ident) => {
        $col.metrics_collector.clone()
    };
}

macro_rules! scylla_session {
    ($session:ident) => {
        $session.lock.lock().await
    };
}

macro_rules! cache {
    ($shared_cache:ident) => {
        $shared_cache.lock.lock().await
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

    ($http:expr, $name:expr, $scylla_session:expr) => {
        match $http.match_info().get($name) {
            Some(param) => {
                if ! db::prelude::check_sid($scylla_session, param.to_string().clone()).await {
                    return actix_web::HttpResponse::NotFound().body(format!("Couldn't find that server. ({}) :(", param.to_string().clone()));
                }
                param.to_string()
            },
            None => {
                return actix_web::HttpResponse::BadRequest()
                    .body(format!("missing `{}` parameter", $name));
            }
        }
    };

    ($http:expr, $name:expr, $scylla_session:expr, $sid:expr) => {
        match $http.match_info().get($name) {
            Some(param) => {
                if ! db::prelude::check_channel_name($scylla_session, $sid.clone(), param.to_string().clone()).await {
                    return actix_web::HttpResponse::NotFound().body(format!("Couldn't find that channel. ({}) :(", param.to_string().clone()));
                }
                param.to_string()
            },
            None => {
                return actix_web::HttpResponse::BadRequest()
                    .body(format!("missing `{}` parameter", $name));
            }
        }
    };
}

#[named]
pub async fn check_user_password(
    secrets:Vec<db::structures::UserSecrets>,
    username: &str,
    password: &str,
    scylla_session: tokio::sync::MutexGuard<'_, scylla::client::session::Session>,
    cache: tokio::sync::MutexGuard<'_, Cache<std::string::String, std::string::String>>,
    new_token_holder: structures::TokenHolder
) -> actix_web::HttpResponse {

    if let Some(password_hash) = secrets[0].password_hash.clone()
    && let Some(user_salt) = secrets[0].user_salt.clone()
    && let Some(password_salt) = secrets[0].password_salt.clone() {
        let decrypted_user_salt = match security::aes::try_decrypt(&user_salt) {
            Ok(salt) => salt,
            Err(err) => {
                logging::log(
                    &format!("Failed to decrypt user salt: {err}"),
                    Some(function_name!()),
                );
                return actix_web::HttpResponse::InternalServerError()
                    .body("Authentication failed");
            }
        };
        let decrypted_password_salt = match security::aes::try_decrypt(&password_salt) {
            Ok(salt) => salt,
            Err(err) => {
                logging::log(
                    &format!("Failed to decrypt password salt: {err}"),
                    Some(function_name!()),
                );
                return actix_web::HttpResponse::InternalServerError()
                    .body("Authentication failed");
            }
        };
        let user_password_plain = match security::aes::try_encrypt_with_key(
            &format!("{}{}", decrypted_user_salt.clone(), password),
            &decrypted_password_salt,
        ) {
            Ok(inner) => match security::aes::try_encrypt(&inner) {
                Ok(encrypted) => encrypted,
                Err(err) => {
                    logging::log(
                        &format!("Failed to encrypt password for verification: {err}"),
                        Some(function_name!()),
                    );
                    return actix_web::HttpResponse::InternalServerError()
                        .body("Authentication failed");
                }
            },
            Err(err) => {
                logging::log(
                    &format!("Failed to encrypt password for verification: {err}"),
                    Some(function_name!()),
                );
                return actix_web::HttpResponse::InternalServerError()
                    .body("Authentication failed");
            }
        };

        if security::argon_check(&user_password_plain, &password_hash) {
            if let Err(insert_err) = db::prelude::insert_user_token(
                &scylla_session,
                &cache,
                db::structures::KeyUser {
                    key: Some(armor_token_or!(
                        &new_token_holder.token,
                        actix_web::HttpResponse::InternalServerError()
                            .body("Failed to insert new token")
                    )),
                    username: Some(username.to_string()),
                },
            )
            .await {
                logging::log(&format!("Failed to insert token due to error:\n {insert_err}"), Some(function_name!()));
                return actix_web::HttpResponse::InternalServerError().body("Failed to insert new token");
            }

            return actix_web::HttpResponse::Ok().json(&new_token_holder);

        }
        
        logging::log("Failed because user supplied password is incorrect.", Some(function_name!()));
        return actix_web::HttpResponse::Unauthorized().body("Invalid username or password");

    }
    
    logging::log("Failed because user supplied data is incorrect.", Some(function_name!()));
    actix_web::HttpResponse::Unauthorized().body("Invalid username or password")
}
