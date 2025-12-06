use crate::api::structures;
use crate::db;
use crate::security;

#[actix_web::post("/api/fetch_friend_list")]
pub async fn fetch_friend_list(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::TokenUser>,
) -> impl actix_web::Responder {
    let scylla_session = match session.lock.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return actix_web::HttpResponse::InternalServerError()
                .body("Internal error: scylla session lock posioned.");
        }
    };
    let cache = match shared_cache.lock.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return actix_web::HttpResponse::InternalServerError()
                .body("Internal error: cache lock poisoned.");
        }
    };

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

    if let Some(friends) = db::friends::fetch_friends(&scylla_session, req.username.clone()).await {
        let friend_info: Vec<structures::FriendInfo> = friends
            .into_iter()
            .map(|(friend_username, friends_since)| structures::FriendInfo {
                username: friend_username,
                friends_since: friends_since.format("%Y-%m-%d %H:%M:%S").to_string(),
            })
            .collect();

        actix_web::HttpResponse::Ok().json(structures::FriendListResp {
            friends: friend_info,
        })
    } else {
        actix_web::HttpResponse::Ok().json(structures::FriendListResp {
            friends: Vec::new(),
        })
    }
}

#[actix_web::post("/api/delete_friend")]
pub async fn delete_friend(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    shared_cache: actix_web::web::Data<security::structures::MokaCache>,
    req: actix_web::web::Json<structures::FriendListReq>,
) -> impl actix_web::Responder {
    let scylla_session = match session.lock.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return actix_web::HttpResponse::InternalServerError()
                .body("Internal error: scylla session lock posioned.");
        }
    };
    let cache = match shared_cache.lock.lock() {
        Ok(guard) => guard,
        Err(_) => {
            return actix_web::HttpResponse::InternalServerError()
                .body("Internal error: cache lock poisoned.");
        }
    };

    if db::prelude::check_token(
        &scylla_session,
        &cache,
        req.token.clone(),
        Some(req.user.clone()),
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
        (Some(Ok(_)), Some(Ok(_))) => {
            actix_web::HttpResponse::Ok().json(structures::DeleteFriendResp {
                status: "friendship_deleted".to_string(),
                user: req.user.clone(),
                friend: req.friend.clone(),
            })
        }
        (Some(Err(e)), _) | (_, Some(Err(e))) => {
            eprintln!("Error detecting friend: {e}");
            actix_web::HttpResponse::InternalServerError().body("Failed to delete friend.")
        }
        _ => actix_web::HttpResponse::InternalServerError().body("Failed to delete friend."),
    }
}
