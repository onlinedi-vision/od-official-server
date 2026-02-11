use crate::api::{structures,statics,prelude};
use crate::db;
use crate::security;
use crate::utils::logging;

use ::function_name::named;

#[actix_web::post("/api/new_user")]
pub async fn new_user_login(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::NewUser>,
) -> impl actix_web::Responder {
	if req.username.len() > statics::MAX_USERNAME_LENGTH {
		return actix_web::HttpResponse::LengthRequired()
			.body(format!("Failed to create user: Username longer than {}", statics::MAX_SERVER_LENGTH));
	}
    let user_salt = security::salt();
    let password_salt = security::salt();
    let password_hash = security::argon(
        security::aes::encrypt(
            &security::aes::encrypt_with_key(
                &format!("{}{}", user_salt.clone(), req.password.clone()),
                &password_salt,
            )
        )
    );
    let token_holder = structures::TokenHolder {
        token: security::token(),
    };
    let user_instance = db::structures::User::new(
        req.username.clone(),
        req.email.clone(),
        password_hash.clone().expect(
            "Argon2 failed to create a proper hash. Check src/security/mod.rs:argon()"
        ),
        security::armor_token(token_holder.token.clone()),
        security::aes::encrypt(&user_salt),
        security::aes::encrypt(&password_salt),
    );

    let scylla_session = scylla_session!(session);
    match db::users::insert_new_user(&scylla_session, user_instance).await {
        None => actix_web::HttpResponse::Conflict().body("User already exists or insert failed"),
        Some(_) => actix_web::HttpResponse::Ok().json(&token_holder),
    }
}

#[named]
#[actix_web::post("/api/try_login")]
pub async fn try_login(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::LoginUser>,
) -> impl actix_web::Responder {
    let new_token_holder = structures::TokenHolder {
        token: security::token(),
    };
    let username = db::structures::UserUsername {
        username: Some(req.username.clone()),
    };
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    match db::users::get_user_password_hash(&scylla_session, username).await {
        Some(secrets) => {
            let password_hash = secrets[0].password_hash.clone().unwrap();
            let user_salt = secrets[0].user_salt.clone().unwrap();
            let password_salt = secrets[0].password_salt.clone().unwrap();
            let decrypted_user_salt = security::aes::decrypt(&user_salt);
            let decrypted_password_salt = security::aes::decrypt(&password_salt);
            let user_password_plain =
                security::aes::encrypt(
                    &security::aes::encrypt_with_key(
                        &format!("{}{}", decrypted_user_salt.clone(), req.password.clone()),
                        &decrypted_password_salt,
                )
            );
            
            if security::argon_check(user_password_plain, password_hash) {
                let _ = db::prelude::insert_user_token(
                    &scylla_session,
                    &cache,
                    db::structures::KeyUser {
                        key: Some(security::armor_token(new_token_holder.token.clone())),
                        username: Some(req.username.clone()),
                    },
                )
                .await;

                actix_web::HttpResponse::Ok().json(&new_token_holder)
            } else {
                println!("not matchy");
                actix_web::HttpResponse::Unauthorized().body("Invalid username or password")
            }
        }
        _ => {
            logging::log("Failed because user password hash cannot be retrieved from scylla.", Some(function_name!()));
            actix_web::HttpResponse::Unauthorized().body("Invalid username or password")
        }
    }
}

#[named]
#[actix_web::post("/api/token_login")]
pub async fn token_login(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::TokenLoginUser>,
) -> impl actix_web::Responder {
    let new_token_holder = structures::TokenHolder {
        token: security::token(),
    };

    let username = db::structures::UserUsername {
        username: Some(req.username.clone()),
    };

    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);

    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await
    .is_some()
    {
        match db::users::get_user_password_hash(&scylla_session, username).await {
            Some(secrets) => {
                let password_hash = secrets[0].password_hash.clone().unwrap();
                let user_salt = secrets[0].user_salt.clone().unwrap();
                let password_salt = secrets[0].password_salt.clone().unwrap();
                let decrypted_user_salt = security::aes::decrypt(&user_salt);
                let decrypted_password_salt = security::aes::decrypt(&password_salt);
                let user_password_plain = security::aes::encrypt(
                    &security::aes::encrypt_with_key(
                        &format!("{}{}", decrypted_user_salt.clone(), req.password.clone()),
                        &decrypted_password_salt,
                    )
                );
                if security::argon_check(user_password_plain, password_hash) {
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
                    println!("not matchy");
                    actix_web::HttpResponse::Unauthorized().body("Invalid password")
                }
            }
            _ => {
                logging::log("Failed because user password hash cannot be retrieved from scylla.", Some(function_name!()));
                actix_web::HttpResponse::Unauthorized().body("Invalid password")
            }
        }
    } else {
        logging::log("Failed because user supplied token is incorrect.", Some(function_name!()));
        actix_web::HttpResponse::Unauthorized().body("Invalid or expired token")
    }
}

#[named]
#[actix_web::post("/api/get_user_servers")]
pub async fn get_user_servers(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::TokenUser>,
) -> impl actix_web::Responder {
    let new_token_holder = structures::TokenHolder {
        token: security::token(),
    };
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await
    .is_some()
    {
        match db::server::fetch_user_servers(&scylla_session, req.username.clone()).await {
            Some(sids) => {
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

                actix_web::HttpResponse::Ok().json(&structures::ServersList {
                    token: new_token_holder.token.clone(),
                    s_list: sids,
                })
            }
            None => {
                logging::log("no hash", Some(function_name!()));
                actix_web::HttpResponse::NotFound().body("No servers found for user")
            }
        }
    } else {
        logging::log("no token", Some(function_name!()));
        actix_web::HttpResponse::Unauthorized().body("Invalid or expired token")
    }
}

#[named]
#[actix_web::post("/api/get_user_pfp")]
pub async fn get_user_pfp(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::TokenUser>,
) -> impl actix_web::Responder {
    let new_token_holder = structures::TokenHolder {
        token: security::token(),
    };
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await
    .is_some()
    {
        match db::users::fetch_user_pfp(&scylla_session, &req.username).await {
            Some(pfp_row) => {
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

                actix_web::HttpResponse::Ok().json(&structures::GetUserPfpResp {
                    token: new_token_holder.token.clone(),
                    img_url: pfp_row.pfp,
                })
            }
            None => actix_web::HttpResponse::NotFound().body("User not found."),
        }
    } else {
        logging::log("no token", Some(function_name!()));
        actix_web::HttpResponse::Unauthorized().body("Invalid or expired token")
    }
}

#[named]
#[actix_web::post("/api/set_user_pfp")]
pub async fn set_user_pfp(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::SetUserPfpReq>,
) -> impl actix_web::Responder {
    let new_token_holder = structures::TokenHolder {
        token: security::token(),
    };
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await
    .is_some()
    {
        let img_opt = match req.img_url.as_deref() {
            // If "" -> None
            Some(s) if s.trim().is_empty() => None,
            other => other,
        };
        match db::users::set_user_pfp(&scylla_session, &req.username, img_opt).await {
            Some(Ok(())) => {
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

                actix_web::HttpResponse::Ok().json(&structures::GetUserPfpResp {
                    token: new_token_holder.token.clone(),
                    img_url: req.img_url.clone(),
                })
            }
            Some(Err(_e)) => actix_web::HttpResponse::InternalServerError()
                .body("Failed to update profile picture."),
            None => actix_web::HttpResponse::InternalServerError()
                .body("Failed to update profile picture."),
        }
    } else {
        logging::log("no token", Some(function_name!()));
        actix_web::HttpResponse::Unauthorized().body("Invalid or expired token")
    }
}
