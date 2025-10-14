#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use scylla::client::session::Session;

use crate::api::structures;
use crate::db;
use crate::security;

#[actix_web::post("/api/spell/cast")]
pub async fn spell_cast(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::SpellCaster>,
) -> impl actix_web::Responder {
    let scylla_session = session.lock.lock().unwrap();

    let new_key   = security::token();
    let new_spell = security::token();

    if let Some(_) = db::spell_caster::spell(&scylla_session, new_key.clone(), new_spell.clone(), req.username.clone()).await {
        actix_web::HttpResponse::Ok().json(
            &db::structures::Spell {
                key: Some(new_key),
                spell: Some(new_spell)
            }
        )
    } else {
        actix_web::HttpResponse::InternalServerError().body(
            "Spell couldn't be cast."
        )
    }  
}


#[actix_web::post("/api/spell/check")]
pub async fn spell_check(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::SpellChecker>,
) -> impl actix_web::Responder {
    let scylla_session = session.lock.lock().unwrap();
    
    if let Some(_) = db::prelude::check_token(
        &scylla_session,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await
    {
        if let Some(spell) = db::spell_caster::spell_check(&scylla_session, req.key.clone(), req.username.clone()).await {
            let _ = db::spell_caster::spell_repel(&scylla_session, req.key.clone()).await;

            actix_web::HttpResponse::Ok().body(
                spell
            )
        } else {
            actix_web::HttpResponse::InternalServerError().body("Could not find Spell...")
        }
    } else {
        actix_web::HttpResponse::Unauthorized().body("Invalid token")
    }

}
