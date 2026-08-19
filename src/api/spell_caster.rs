#![allow(unused_imports)]
use scylla::client::session::Session;

use crate::api::structures;
use crate::db;
use crate::security;
// use crate::metrics;

/// Casts a "spell" for a user: generates a one-time key/spell pair and stores it server-side,
/// to be later verified via `/spell/check`.
///
/// ### Request JSON (`SpellCaster`)
/// ```json
/// {
///   "username": "alice"
/// }
/// ```
///
/// ### Example (reqwest)
/// ```rust
/// let client = reqwest::Client::new();
/// let res = client
///     .post("http://localhost:1313/spell/cast")
///     .json(&serde_json::json!({
///         "username": "alice"
///     }))
///     .send()
///     .await?;
/// ```
#[actix_web::post("/spell/cast")]
pub async fn spell_cast(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::SpellCaster>,
) -> impl actix_web::Responder {
    let scylla_session = scylla_session!(session);

    let new_key = security::token();
    let new_spell = security::token();

    if db::spell_caster::spell(
        &scylla_session,
        new_key.clone(),
        new_spell.clone(),
        req.username.clone(),
    )
    .await
    .is_some()
    {
        return actix_web::HttpResponse::Ok().json(&db::structures::Spell {
            key: Some(new_key),
            spell: Some(new_spell),
        });
    }
    
    actix_web::HttpResponse::InternalServerError().body("Spell couldn't be cast.")

}

/// Verifies a previously cast spell for a user using its `key`, and returns (then invalidates)
/// the associated spell value. Requires a valid session token.
///
/// ### Request JSON (`SpellChecker`)
/// ```json
/// {
///   "username": "alice",
///   "token": "abc123",
///   "key": "spell-key-xyz"
/// }
/// ```
///
/// ### Example (reqwest)
/// ```rust
/// let client = reqwest::Client::new();
/// let res = client
///     .post("http://localhost:1313/spell/check")
///     .json(&serde_json::json!({
///         "username": "alice",
///         "token": "abc123",
///         "key": "spell-key-xyz"
///     }))
///     .send()
///     .await?;
/// ```
#[actix_web::post("/spell/check")]
pub async fn spell_check(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::SpellChecker>,
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
    .is_some()
    {
        if let Some(spell) =
            db::spell_caster::spell_check(&scylla_session, req.key.clone(), req.username.clone())
                .await
        {
            let _ = db::spell_caster::spell_repel(&scylla_session, req.key.clone()).await;

            return actix_web::HttpResponse::Ok().body(spell);
        }
        return actix_web::HttpResponse::InternalServerError().body("Could not find Spell...");
    }
    
    actix_web::HttpResponse::Unauthorized().body("Invalid token")

}
