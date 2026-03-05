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
    let password_hash = security::sha512(security::aes::encrypt(&security::aes::encrypt_with_key(
        &format!("{}{}", user_salt.clone(), req.password.clone()),
        &password_salt,
    )));
    let token_holder = structures::TokenHolder {
        token: security::token(),
    };
    let user_instance = db::structures::User::new(
        req.username.clone(),
        req.email.clone(),
        password_hash.clone(),
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

#[actix_web::patch("/api/user/ttl")]
pub async fn patch_user_ttl(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::UpdateUserTTL>,
) -> impl actix_web::Responder {

    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    
    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await.is_none()
    {
        return actix_web::HttpResponse::Unauthorized().body("Invalid token!");
    }

    if db::users::update_ttl(
        &scylla_session,
        req.username.clone(),
        req.ttl.clone(),
    )
    .await.is_err()
    {
        return actix_web::HttpResponse::InternalServerError()
            .body("Internal error: scylla session lock poisoned.");
    }

    actix_web::HttpResponse::Ok()
        .body("TTL Updated.")

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
    
    if let Some(secrets) = db::users::get_user_password_hash(&scylla_session, username).await  {
        // TODO: wow this returns a HTTP responder... why?
        return prelude::check_user_password(secrets, &req.username, &req.password, scylla_session, cache, new_token_holder).await;
    }
    
    logging::log("Failed because user password hash cannot be retrieved from scylla.", Some(function_name!()));
    actix_web::HttpResponse::Unauthorized().body("Invalid username or password")
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
    .is_none()
    {
        logging::log("Failed because user supplied token is incorrect.", Some(function_name!()));
        return actix_web::HttpResponse::Unauthorized().body("Invalid or expired token");
    }
    if let Some(secrets) = db::users::get_user_password_hash(&scylla_session, username).await {
        return prelude::check_user_password(secrets, &req.username, &req.password, scylla_session, cache, new_token_holder).await;
    }
    
    logging::log("Failed because user password hash cannot be retrieved from scylla.", Some(function_name!()));
    actix_web::HttpResponse::Unauthorized().body("Invalid password")

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
    .is_none()
    {
        logging::log("no token", Some(function_name!()));
        return actix_web::HttpResponse::Unauthorized().body("Invalid or expired token");
    }

    if let Some(sids) = db::server::fetch_user_servers(&scylla_session, req.username.clone()).await {
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

        return actix_web::HttpResponse::Ok().json(&structures::ServersList {
            token: new_token_holder.token.clone(),
            s_list: sids,
        });
    }
    
    logging::log("no hash", Some(function_name!()));
    actix_web::HttpResponse::NotFound().body("No servers found for user")
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
    .is_none()
    {
        logging::log("no token", Some(function_name!()));
        return actix_web::HttpResponse::Unauthorized().body("Invalid or expired token");
    }
    
    if let Some(pfp_row) = db::users::fetch_user_pfp(&scylla_session, &req.username).await {
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

        return actix_web::HttpResponse::Ok().json(&structures::GetUserPfpResp {
            token: new_token_holder.token.clone(),
            img_url: pfp_row.pfp,
        });
    }
    actix_web::HttpResponse::NotFound().body("User not found.")
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
    .is_none()
    {
        logging::log("no token", Some(function_name!()));
        return actix_web::HttpResponse::Unauthorized().body("Invalid or expired token");
    }

    let img_opt = match req.img_url.as_deref() {
        Some(s) if s.trim().is_empty() => None,
        other => other,
    };

    if db::users::set_user_pfp(&scylla_session, &req.username, img_opt).await.is_err() {
        return actix_web::HttpResponse::InternalServerError()
            .body("Failed to update profile picture.");
    }

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
