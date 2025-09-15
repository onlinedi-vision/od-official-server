#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]
use crate::api::structures;
use crate::db;
use crate::security;

// !TODO: create_server API
#[actix_web::post("/api/create_server")]
pub async fn create_server(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::CreateServer>,
) -> impl actix_web::Responder {
    let scylla_session = session.lock.lock().unwrap();
    if let Some(_) = db::prelude::check_token(
        &scylla_session,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await
    {
        let sid = security::sid();
        if let Some(_) = db::server::create_server(
            &scylla_session,
            sid.clone(),
            &req.desc,
            &req.img_url,
            &req.name,
            req.username.clone(),
        )
        .await
        {
            let _ =
                db::server::create_channel(&scylla_session, sid.clone(), "info".to_string()).await;
            let new_token_holder = structures::TokenHolder {
                token: security::token(),
            };
            let _ = db::prelude::update_user_key(
                &scylla_session,
                db::structures::KeyUser {
                    key: Some(new_token_holder.token.clone()),
                    username: Some(req.username.clone()),
                },
            )
            .await;
            if let Some(_) =
                db::server::add_user_to_server(&scylla_session, sid, req.username.clone()).await
            {
                return actix_web::HttpResponse::Ok().json(&new_token_holder);
            } else {
                println!("SERVERS FAIL: add_user_to_server");
                return actix_web::HttpResponse::InternalServerError()
                    .body("Failed to add user to server");
            }
        } else {
            println!("SERVERS FAIL: create_server");
            return actix_web::HttpResponse::InternalServerError().body("Failed to create server");
        }
    } else {
        println!("SERVERS FAIL: invalid token in create_server");
        return actix_web::HttpResponse::Unauthorized().body("Invalid token");
    }
}

#[actix_web::post("/servers/{sid}/api/join")]
pub async fn join_server(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::TokenUser>,
    http: actix_web::HttpRequest,
) -> impl actix_web::Responder {
    let sid: String = http.match_info().get("sid").unwrap().to_string();
    let scylla_session = session.lock.lock().unwrap();
    if let Some(_) = db::prelude::check_token(
        &scylla_session,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await
    {
        if let Some(_) =
            db::server::add_user_to_server(&scylla_session, sid, req.username.clone()).await
        {
            let new_token_holder = structures::TokenHolder {
                token: security::token(),
            };
            let _ = db::prelude::update_user_key(
                &scylla_session,
                db::structures::KeyUser {
                    key: Some(new_token_holder.token.clone()),
                    username: Some(req.username.clone()),
                },
            )
            .await;

            return actix_web::HttpResponse::Ok().json(&new_token_holder);
        } else {
            println!("SERVERS FAIL: add_user_to_server");
            return actix_web::HttpResponse::InternalServerError()
                .body("Failed to add user to server");
        }
    } else {
        println!("SERVERS FAIL: invalid token in create_server");
        return actix_web::HttpResponse::Unauthorized().body("Invalid token");
    }
}

// !TODO: get_user_servers API
#[actix_web::post("/servers/{sid}/api/get_server_users")]
pub async fn get_server_users(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::TokenUser>,
    http: actix_web::HttpRequest,
) -> impl actix_web::Responder {
    let sid: String = http.match_info().get("sid").unwrap().to_string();
    let scylla_session = session.lock.lock().unwrap();
    if let Some(_) = db::prelude::check_user_is_in_server(
        &scylla_session,
        sid.clone(),
        req.token.clone(),
        req.username.clone(),
    )
    .await
    {
        if let Some(users) = db::server::fetch_server_users(&scylla_session, sid.clone()).await {
            return actix_web::HttpResponse::Ok().json(&structures::UsersList { u_list: users });
        } else {
            return actix_web::HttpResponse::Ok()
                .json(&structures::UsersList { u_list: Vec::new() });
        }
    } else {
        return actix_web::HttpResponse::Unauthorized().body("Invalid token or user not in server");
    }
}

#[actix_web::get("/servers/{sid}/api/get_server_info")]
pub async fn get_server_info(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    http: actix_web::HttpRequest,
) -> impl actix_web::Responder {
    let sid: String = http.match_info().get("sid").unwrap().to_string();
    let scylla_session = session.lock.lock().unwrap();
    if let Some(server_info) = db::server::fetch_server_info(&scylla_session, sid.clone()).await {
        return actix_web::HttpResponse::Ok().json(&server_info);
    } else {
        return actix_web::HttpResponse::NotFound().json(&structures::Status {
            status: "nok".to_string(),
        });
    }
}

#[actix_web::post("/api/send_invite")]
pub async fn send_dm_invite(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::SendInviteReq>,
) -> impl actix_web::Responder {
    let scylla_session = session.lock.lock().unwrap();

    if db::prelude::check_token(
        &scylla_session,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await
    .is_some()
    {
        let (u1, u2) = if req.username < req.recipient {
            (req.username.clone(), req.recipient.clone())
        } else {
            (req.recipient.clone(), req.username.clone())
        };

        if let Some(_) = db::server::fetch_dm_invite(&scylla_session, u1.clone(), u2.clone()).await
        {
            return actix_web::HttpResponse::Ok().json(structures::SendInviteResp {
                status: "already_invited".to_string(),
                invite_id: None,
                u1,
                u2,
                sender: Some(req.username.clone()),
            });
        }

        let invite_id = uuid::Uuid::new_v4().to_string();

        if let Some(_) = db::server::send_dm_invite(
            &scylla_session,
            u1.clone(),
            u2.clone(),
            invite_id.clone(),
            req.username.clone(),
        )
        .await
        {
            return actix_web::HttpResponse::Ok().json(structures::SendInviteResp {
                status: "invite_created".to_string(),
                invite_id: Some(invite_id),
                u1,
                u2,
                sender: Some(req.username.clone()),
            });
        } else {
            return actix_web::HttpResponse::InternalServerError().body("Failed to create invite");
        }
    } else {
        return actix_web::HttpResponse::Unauthorized().body("Invalid token");
    }
}

#[actix_web::post("/api/accept_invite")]
pub async fn accept_dm_invite(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::AcceptInviteReq>,
) -> impl actix_web::Responder {
    let scylla_session = session.lock.lock().unwrap();

    if db::prelude::check_token(
        &scylla_session,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await
    .is_some()
    {
        let (u1, u2) = if req.username < req.other_user {
            (req.username.clone(), req.other_user.clone())
        } else {
            (req.other_user.clone(), req.username.clone())
        };

        if let Some((invite_id, sender)) =
            db::server::fetch_dm_invite(&scylla_session, u1.clone(), u2.clone()).await
        {
            let sid = format!("!{}", security::sid());

            if db::server::create_server(
                &scylla_session,
                sid.clone(),
                &"Direct Message".to_string(),
                &"".to_string(),
                &format!("DM: {} & {}", u1, u2),
                u1.clone(),
            )
            .await
            .is_some()
            {
                let _ =
                    db::server::add_user_to_server(&scylla_session, sid.clone(), u1.clone()).await;
                let _ =
                    db::server::add_user_to_server(&scylla_session, sid.clone(), u2.clone()).await;
                let _ = db::server::create_channel(&scylla_session, sid.clone(), "dm".to_string())
                    .await;
                let _ = db::server::delete_dm_invite(&scylla_session, u1.clone(), u2.clone()).await;

                return actix_web::HttpResponse::Ok().json(structures::AcceptInviteResp {
                    status: "dm_created".to_string(),
                    sid: Some(sid),
                    invite_id,
                    u1,
                    u2,
                    sender: Some(sender),
                });
            } else {
                return actix_web::HttpResponse::InternalServerError()
                    .body("Failed to create DM server");
            }
        } else {
            return actix_web::HttpResponse::NotFound().body("Invite not found");
        }
    } else {
        return actix_web::HttpResponse::Unauthorized().body("Invalid token");
    }
}

#[actix_web::post("/api/fetch_pending_dm_invites")]
pub async fn fetch_pending_dm_invites(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::TokenUser>,
) -> impl actix_web::Responder {
    let scylla_session = session.lock.lock().unwrap();

    if db::prelude::check_token(
        &scylla_session,
        req.token.clone(),
        Some(req.username.clone()),
    )
    .await
    .is_some()
    {
        if let Some(invites) =
            db::server::fetch_pending_dm_invites(&scylla_session, req.username.clone()).await
        {
            let pending: Vec<structures::PendingInvite> = invites
                .into_iter()
                .map(|(invite_id, sender)| structures::PendingInvite { invite_id, sender })
                .collect();

            return actix_web::HttpResponse::Ok()
                .json(structures::PendingInvitesResp { invites: pending });
        } else {
            return actix_web::HttpResponse::Ok().json(structures::PendingInvitesResp {
                invites: Vec::new(),
            });
        }
    } else {
        return actix_web::HttpResponse::Unauthorized().body("Invalid token");
    }
}
