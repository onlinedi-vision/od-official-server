use crate::api::structures;
use crate::db;
use crate::security;
use crate::utils::logging;
use crate::metrics;

use ::function_name::named;

#[actix_web::post("/fetch_friend_list")]
pub async fn fetch_friend_list(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::TokenUser>,
    shared_collector: actix_web::web::Data<metrics::prelude::MetricsCollector>,
) -> impl actix_web::Responder {
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    let collector = collector!(shared_collector);

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
        return actix_web::HttpResponse::Unauthorized().body("Invalid token");
    }

    if let Some(friends) = db::friends::fetch_friends(&scylla_session, req.username.clone()).await {
        let friend_info: Vec<structures::FriendInfo> = friends
            .into_iter()
            .map(|(friend_username, friends_since)| structures::FriendInfo {
                username: friend_username,
                friends_since: friends_since.format("%Y-%m-%d %H:%M:%S").to_string(),
            })
            .collect();

        return actix_web::HttpResponse::Ok().json(structures::FriendListResp {
            friends: friend_info,
        });
    }
    actix_web::HttpResponse::Ok().json(structures::FriendListResp {
        friends: Vec::new(),
    })
}

#[named]
#[actix_web::post("/delete_friend")]
pub async fn delete_friend(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::FriendListReq>,
    shared_collector: actix_web::web::Data<metrics::prelude::MetricsCollector>,
) -> impl actix_web::Responder {
    let scylla_session = scylla_session!(session);
    let cache = cache!(shared_cache);
    let collector = collector!(shared_collector);

    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.user.clone()),
        &collector,
    )
    .await
    .is_none()
    {
        return actix_web::HttpResponse::Unauthorized().body("Invalid token");
    }

    let result1 =
        db::friends::delete_friend(&scylla_session, req.user.clone(), req.friend.clone()).await;
    let result2 =
        db::friends::delete_friend(&scylla_session, req.friend.clone(), req.user.clone()).await;

    match (result1, result2) {
        (Some(Ok(())), Some(Ok(()))) => {
            actix_web::HttpResponse::Ok().json(structures::DeleteFriendResp {
                status: "friendship_deleted".to_string(),
                user: req.user.clone(),
                friend: req.friend.clone(),
            })
        }
        (Some(Err(e)), _) | (_, Some(Err(e))) => {
            logging::log(&format!("Error detecting friend: {e}"), Some(function_name!()));
            actix_web::HttpResponse::InternalServerError().body("Failed to delete friend.")
        }
        _ => actix_web::HttpResponse::InternalServerError().body("Failed to delete friend."),
    }
}
