use crate::api::structures;
use crate::db;
use crate::security;

#[actix_web::post("/api/send_dm_invite")]
pub async fn send_dm_invite(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::SendInviteReq>,
) -> impl actix_web::Responder {
    let scylla_session = session.lock.lock().unwrap();
    if db::prelude::check_token(&scylla_session, req.token.clone(), Some(req.sender.clone()))
        .await
        .is_some()
    {
        let (u1, u2) = if req.sender < req.recipient {
            (req.sender.clone(), req.recipient.clone())
        } else {
            (req.recipient.clone(), req.sender.clone())
        };
        if let Some((invite_id, sender)) =
            db::invites::fetch_dm_invite(&scylla_session, u1.clone(), u2.clone()).await
        {
            return actix_web::HttpResponse::Ok().json(structures::SendInviteResp {
                status: "already_invited".to_string(),
                invite_id: Some(invite_id),
                u1,
                u2,
                sender: Some(req.sender.clone()),
            });
        }
        let invite_id = uuid::Uuid::new_v4().to_string();
        if let Some(_) = db::invites::send_dm_invite(
            &scylla_session,
            u1.clone(),
            u2.clone(),
            invite_id.clone(),
            req.sender.clone(),
        )
        .await
        {
            return actix_web::HttpResponse::Ok().json(structures::SendInviteResp {
                status: "invite_created".to_string(),
                invite_id: Some(invite_id),
                u1,
                u2,
                sender: Some(req.sender.clone()),
            });
        } else {
            return actix_web::HttpResponse::InternalServerError().body("Failed to create invite");
        }
    } else {
        return actix_web::HttpResponse::Unauthorized().body("Invalid token");
    }
}

#[actix_web::post("/api/accept_dm_invite")]
pub async fn accept_dm_invite(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::AcceptInviteReq>,
) -> impl actix_web::Responder {
    let scylla_session = session.lock.lock().unwrap();
    if db::prelude::check_token(
        &scylla_session,
        req.token.clone(),
        Some(req.recipient.clone()),
    )
    .await
    .is_some()
    {
        let (u1, u2) = if req.recipient < req.sender {
            (req.recipient.clone(), req.sender.clone())
        } else {
            (req.sender.clone(), req.recipient.clone())
        };
        if let Some((invite_id, sender)) =
            db::invites::fetch_dm_invite(&scylla_session, u1.clone(), u2.clone()).await
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
                let _ =
                    db::invites::delete_dm_invite(&scylla_session, u1.clone(), u2.clone()).await;
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

#[actix_web::post("/api/reject_dm_invite")]
pub async fn reject_dm_invite(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::RejectInviteReq>,
) -> impl actix_web::Responder {
    let scylla_session = session.lock.lock().unwrap();

    if db::prelude::check_token(
        &scylla_session,
        req.token.clone(),
        Some(req.recipient.clone()),
    )
    .await
    .is_some()
    {
        let (u1, u2) = if req.recipient < req.sender {
            (req.recipient.clone(), req.sender.clone())
        } else {
            (req.sender.clone(), req.recipient.clone())
        };

        if let Some((invite_id, sender)) =
            db::invites::fetch_dm_invite(&scylla_session, u1.clone(), u2.clone()).await
        {
            let _ = db::invites::delete_dm_invite(&scylla_session, u1.clone(), u2.clone()).await;
            return actix_web::HttpResponse::Ok().json(structures::RejectInviteResp {
                status: "invite_rejected".to_string(),
                invite_id,
                u1,
                u2,
            });
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
            db::invites::fetch_pending_dm_invites(&scylla_session, req.username.clone()).await
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
