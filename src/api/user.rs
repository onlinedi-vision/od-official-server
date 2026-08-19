use crate::api::{prelude, statics, structures};
use crate::db;
use crate::security;
use crate::utils::logging;

use ::function_name::named;

/// Registers a new user account with a hashed/salted password, and returns a fresh session token.
///
/// ### Request JSON (`NewUser`)
/// ```json
/// {
///   "username": "alice",
///   "email": "alice@example.com",
///   "password": "SuperSecret123"
/// }
/// ```
///
/// ### Example (reqwest)
/// ```rust
/// let client = reqwest::Client::new();
/// let res = client
///     .post("http://localhost:1313/new_user")
///     .json(&serde_json::json!({
///         "username": "alice",
///         "email": "alice@example.com",
///         "password": "SuperSecret123"
///     }))
///     .send()
///     .await?;
/// ```
#[actix_web::post("/new_user")]
pub async fn new_user_login(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::NewUser>,
) -> impl actix_web::Responder {
    if req.username.len() > statics::MAX_USERNAME_LENGTH {
        return actix_web::HttpResponse::LengthRequired().body(format!(
            "Failed to create user: Username longer than {}",
            statics::MAX_SERVER_LENGTH
        ));
    }
    let user_salt = security::salt();
    let password_salt = security::salt();
    let password_plain = match security::aes::try_encrypt_with_key(
        &format!("{}{}", user_salt.clone(), req.password.clone()),
        &password_salt,
    ) {
        Ok(inner) => match security::aes::try_encrypt(&inner) {
            Ok(encrypted) => encrypted,
            Err(err) => {
                logging::log(
                    &format!("Failed to encrypt password for new user: {err}"),
                    Some(function_name!()),
                );
                return actix_web::HttpResponse::InternalServerError()
                    .body("Failed to create user");
            }
        },
        Err(err) => {
            logging::log(
                &format!("Failed to encrypt password for new user: {err}"),
                Some(function_name!()),
            );
            return actix_web::HttpResponse::InternalServerError().body("Failed to create user");
        }
    };
    let password_hash = security::argon(&password_plain);
    let token_holder = structures::TokenHolder {
        token: security::token(),
    };
    let armored_token = armor_token_or!(
        &token_holder.token,
        actix_web::HttpResponse::InternalServerError().body("Failed to create user")
    );
    let enc_user_salt = match security::aes::try_encrypt(&user_salt) {
        Ok(salt) => salt,
        Err(err) => {
            logging::log(
                &format!("Failed to encrypt user salt for new user: {err}"),
                Some(function_name!()),
            );
            return actix_web::HttpResponse::InternalServerError().body("Failed to create user");
        }
    };
    let enc_password_salt = match security::aes::try_encrypt(&password_salt) {
        Ok(salt) => salt,
        Err(err) => {
            logging::log(
                &format!("Failed to encrypt password salt for new user: {err}"),
                Some(function_name!()),
            );
            return actix_web::HttpResponse::InternalServerError().body("Failed to create user");
        }
    };
    let user_instance = db::structures::User::new(
        req.username.clone(),
        req.email.clone(),
        password_hash.clone().expect(
            "Argon2 failed to create a proper hash. Check src/security/mod.rs:argon()"
        ),
        armored_token,
        enc_user_salt,
        enc_password_salt,
    );

    let scylla_session = scylla_session!(session);
    match db::users::insert_new_user(&scylla_session, user_instance).await {
        None => actix_web::HttpResponse::Conflict().body("User already exists or insert failed"),
        Some(_) => actix_web::HttpResponse::Ok().json(&token_holder),
    }
}

/// Updates the message time-to-live (TTL) setting for the authenticated user.
///
/// ### Request JSON (`UpdateUserTTL`)
/// ```json
/// {
///   "username": "alice",
///   "token": "abc123",
///   "ttl": "3600"
/// }
/// ```
///
/// ### Example (reqwest)
/// ```rust
/// let client = reqwest::Client::new();
/// let res = client
///     .patch("http://localhost:1313/user/ttl")
///     .json(&serde_json::json!({
///         "username": "alice",
///         "token": "abc123",
///         "ttl": "3600"
///     }))
///     .send()
///     .await?;
/// ```
#[actix_web::patch("/user/ttl")]
pub async fn patch_user_ttl(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::UpdateUserTTL>,
    shared_collector: actix_web::web::Data<structures::AppState>,
) -> impl actix_web::Responder {
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    let collector = cache_metrics!(shared_collector);
    
    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.username.clone()),
        &collector,
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

/// Attempts to log a user in with a username and password. On success, returns a new session
/// token.
///
/// ### Request JSON (`LoginUser`)
/// ```json
/// {
///   "username": "alice",
///   "password": "SuperSecret123"
/// }
/// ```
///
/// ### Example (reqwest)
/// ```rust
/// let client = reqwest::Client::new();
/// let res = client
///     .post("http://localhost:1313/try_login")
///     .json(&serde_json::json!({
///         "username": "alice",
///         "password": "SuperSecret123"
///     }))
///     .send()
///     .await?;
/// ```
#[named]
#[actix_web::post("/try_login")]
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


// TODO: use this endpoint... when it's ready...
// could also be redone... maybe just a new refresh token... when we have those.
/// Validates that a session token is still valid for the given username, effectively logging
/// the user back in without re-entering credentials.
///
/// ### Request JSON (`TokenUser`)
/// ```json
/// {
///   "username": "alice",
///   "token": "abc123"
/// }
/// ```
///
/// ### Example (reqwest)
/// ```rust
/// let client = reqwest::Client::new();
/// let res = client
///     .post("http://localhost:1313/token_login")
///     .json(&serde_json::json!({
///         "username": "alice",
///         "token": "abc123"
///     }))
///     .send()
///     .await?;
/// ```
#[named]
#[actix_web::post("/token_login")]
pub async fn token_login(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::TokenUser>,
    shared_collector: actix_web::web::Data<structures::AppState>,
) -> impl actix_web::Responder {
    
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    let collector = cache_metrics!(shared_collector);

    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.username.clone()),
        &collector,
    )
    .await
    .is_none()
    {
        logging::log("Failed because user supplied token is incorrect.", Some(function_name!()));
        return actix_web::HttpResponse::Unauthorized().body("Invalid or expired token");
    }

    // TODO: whenever we fix tokens... we must rotate the token here.
    logging::log("Logged in using token succesfully.", Some(function_name!()));
    actix_web::HttpResponse::Ok().body("Logged in.")
    
}

/// Fetches the list of server IDs the authenticated user belongs to, and issues a new session
/// token (the old one is invalidated).
///
/// ### Request JSON (`TokenUser`)
/// ```json
/// {
///   "username": "alice",
///   "token": "abc123"
/// }
/// ```
///
/// ### Example (reqwest)
/// ```rust
/// let client = reqwest::Client::new();
/// let res = client
///     .post("http://localhost:1313/get_user_servers")
///     .json(&serde_json::json!({
///         "username": "alice",
///         "token": "abc123"
///     }))
///     .send()
///     .await?;
/// ```
#[named]
#[actix_web::post("/get_user_servers")]
pub async fn get_user_servers(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::TokenUser>,
    shared_collector: actix_web::web::Data<structures::AppState>,
) -> impl actix_web::Responder {
    let new_token_holder = structures::TokenHolder {
        token: security::token(),
    };
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    let collector = cache_metrics!(shared_collector);
    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.username.clone()),
        &collector,
    )
    .await
    .is_none()
    {
        logging::log("no token", Some(function_name!()));
        return actix_web::HttpResponse::Unauthorized().body("Invalid or expired token");
    }

    if let Some(sids) = db::server::fetch_user_servers(&scylla_session, req.username.clone()).await {
        if let Err(insert_err) = db::prelude::insert_user_token(
            &scylla_session,
            &cache,
            db::structures::KeyUser {
                key: Some(armor_token_or!(
                    &new_token_holder.token,
                    actix_web::HttpResponse::InternalServerError().body("Failed to insert new token")
                )),
                username: Some(req.username.clone()),
            },
        )
        .await {
            logging::log(&format!("Failed to insert token due to error:\n {insert_err}"), Some(function_name!()));
            return actix_web::HttpResponse::InternalServerError().body("Failed to insert new token");
        }


        let _ = db::users::delete_token(
            &scylla_session,
            req.username.clone(),
            armor_token_or!(
                &req.token,
                actix_web::HttpResponse::InternalServerError().body("Failed to rotate token")
            ),
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

/// Fetches the authenticated user's profile picture URL, and issues a new session token
/// (the old one is invalidated).
///
/// ### Request JSON (`TokenUser`)
/// ```json
/// {
///   "username": "alice",
///   "token": "abc123"
/// }
/// ```
///
/// ### Example (reqwest)
/// ```rust
/// let client = reqwest::Client::new();
/// let res = client
///     .post("http://localhost:1313/get_user_pfp")
///     .json(&serde_json::json!({
///         "username": "alice",
///         "token": "abc123"
///     }))
///     .send()
///     .await?;
/// ```
#[named]
#[actix_web::post("/get_user_pfp")]
pub async fn get_user_pfp(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::TokenUser>,
    shared_collector: actix_web::web::Data<structures::AppState>,
) -> impl actix_web::Responder {
    let new_token_holder = structures::TokenHolder {
        token: security::token(),
    };
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    let collector = cache_metrics!(shared_collector);
    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.username.clone()),
        &collector,
    )
    .await
    .is_none()
    {
        logging::log("no token", Some(function_name!()));
        return actix_web::HttpResponse::Unauthorized().body("Invalid or expired token");
    }
    
    if let Some(pfp_row) = db::users::fetch_user_pfp(&scylla_session, &req.username).await {
        if let Err(insert_err) = db::prelude::insert_user_token(
            &scylla_session,
            &cache,
            db::structures::KeyUser {
                key: Some(armor_token_or!(
                    &new_token_holder.token,
                    actix_web::HttpResponse::InternalServerError().body("Failed to insert new token")
                )),
                username: Some(req.username.clone()),
            },
        )
        .await {
            logging::log(&format!("Failed to insert token due to error:\n {insert_err}"), Some(function_name!()));
            return actix_web::HttpResponse::InternalServerError().body("Failed to insert new token");
        }

        let _ = db::users::delete_token(
            &scylla_session,
            req.username.clone(),
            armor_token_or!(
                &req.token,
                actix_web::HttpResponse::InternalServerError().body("Failed to rotate token")
            ),
        )
        .await;

        return actix_web::HttpResponse::Ok().json(&structures::GetUserPfpResp {
            token: new_token_holder.token.clone(),
            img_url: pfp_row.pfp,
        });
    }
    actix_web::HttpResponse::NotFound().body("User not found.")
}

/// Sets (or clears, if `img_url` is `None`/empty) the authenticated user's profile picture URL,
/// and issues a new session token (the old one is invalidated).
///
/// ### Request JSON (`SetUserPfpReq`)
/// ```json
/// {
///   "token": "abc123",
///   "username": "alice",
///   "img_url": "https://example.com/pfp.png"
/// }
/// ```
/// > `img_url` is optional; omit it or set it to `null`/an empty string to clear the picture.
///
/// ### Example (reqwest)
/// ```rust
/// let client = reqwest::Client::new();
/// let res = client
///     .post("http://localhost:1313/set_user_pfp")
///     .json(&serde_json::json!({
///         "token": "abc123",
///         "username": "alice",
///         "img_url": "https://example.com/pfp.png"
///     }))
///     .send()
///     .await?;
/// ```
#[named]
#[actix_web::post("/set_user_pfp")]
pub async fn set_user_pfp(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::SetUserPfpReq>,
    shared_collector: actix_web::web::Data<structures::AppState>,
) -> impl actix_web::Responder {
    let new_token_holder = structures::TokenHolder {
        token: security::token(),
    };
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    let collector = cache_metrics!(shared_collector);
    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.username.clone()),
        &collector,
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

    if let Err(insert_err) = db::prelude::insert_user_token(
        &scylla_session,
        &cache,
        db::structures::KeyUser {
            key: Some(armor_token_or!(
                &new_token_holder.token,
                actix_web::HttpResponse::InternalServerError().body("Failed to insert new token")
            )),
            username: Some(req.username.clone()),
        },
    )
    .await {
        logging::log(&format!("Failed to insert token due to error:\n {insert_err}"), Some(function_name!()));
        return actix_web::HttpResponse::InternalServerError().body("Failed to insert new token");
    }

    let _ = db::users::delete_token(
        &scylla_session,
        req.username.clone(),
        armor_token_or!(
            &req.token,
            actix_web::HttpResponse::InternalServerError().body("Failed to rotate token")
        ),
    )
    .await;

    actix_web::HttpResponse::Ok().json(&structures::GetUserPfpResp {
        token: new_token_holder.token.clone(),
        img_url: req.img_url.clone(),
    })
}
