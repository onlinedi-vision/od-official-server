use crate::api::structures;
use crate::db;
use crate::security;
use crate::utils::logging;

use ::function_name::named;

#[named]
#[actix_web::post("/api/add_server_role")]
pub async fn add_server_role(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::ServerRoleRequest>,
) -> impl actix_web::Responder {
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
        return actix_web::HttpResponse::Unauthorized().body("Invalid token");
    }
    
    let role = db::structures::ServerRole {
        name: req.name.clone(),
        server_id: req.server_id.clone(),
        id:req.id.clone(),
        color: req.color.clone().unwrap(),
        permissions: req.permissions.clone(),
    };

    if let Some(result) =
        db::roles::insert_server_role(&scylla_session, req.server_id.clone(), role).await
        && result.is_ok() {
            return actix_web::HttpResponse::Ok().body("Role added successfully");
        }
    logging::log(&format!("Error inserting role: {:?}", req.name.clone()), Some(function_name!()));
    actix_web::HttpResponse::InternalServerError().body("Failed to insert role")
}

