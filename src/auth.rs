use actix_web::{
        body::MessageBody,
        dev::{Payload, ServiceRequest, ServiceResponse},
        error::{ErrorBadRequest, ErrorInternalServerError, ErrorUnauthorized},
        middleware::Next,
        web, Error, FromRequest, HttpMessage, HttpRequest,
    };
use serde::Deserialize;
use std::future::{ready, Ready};

use crate::{
    api::structures::AppState,
    db,
    security::structures::{MokaCache, ScyllaSession},
};


#[derive(Clone)]
pub struct AuthenticatedUser {
    username: String,
}

impl AuthenticatedUser {
    pub fn username(&self) -> &str {
        &self.username
    }
}


impl FromRequest for AuthenticatedUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        ready(
            req.extensions()
                .get::<Self>()
                .cloned()
                .ok_or_else(|| { ErrorInternalServerError(
                        "Authenticated route is missing authentication middleware",
                    )
                }),
        )
    }
}

#[derive(Clone, Copy)]
enum CallerField {
    Username,
    Sender,
    Recipient,
    User,
}

enum AuthPolicy {
    Public,
    Token(CallerField),
}

fn policy(method: &str, path: &str) -> AuthPolicy {
    match (method, path) {
        ("GET", "/version" | "/metrics")
        | ("POST", "/new_user" | "/try_login" | "/spell/cast") => {
            AuthPolicy::Public
        }

        ("GET", path) if is_server_info_path(path) => AuthPolicy::Public,
        
        ("POST", "/send_dm_invite") => {
            AuthPolicy::Token(CallerField::Sender)
        }

        ("POST", "/accept_dm_invite" | "/reject_dm_invite") => {
            AuthPolicy::Token(CallerField::Recipient)
        }

        ("POST", "/delete_friend") => {
            AuthPolicy::Token(CallerField::User)
        }

        _ => AuthPolicy::Token(CallerField::Username),
    }
}

fn is_server_info_path(path: &str) -> bool {
    let parts: Vec<_> = path.split('/').collect();

    matches!(
        parts.as_slice(),
        ["", "servers", sid, "get_server_info"] if !sid.is_empty()
    )
}


// compatibility from here
#[derive(Deserialize)]
struct Credentials {
    token: String,
    username: Option<String>,
    sender: Option<String>,
    recipient: Option<String>,
    user: Option<String>,
}

impl Credentials {
    fn caller(&self, field: CallerField) -> Result<&str, Error> {
        let caller = match field {
            CallerField::Username => self.username.as_deref(),
            CallerField::Sender => self.sender.as_deref(),
            CallerField::Recipient => self.recipient.as_deref(),
            CallerField::User => self.user.as_deref(),
        };

        caller
            .filter(|name| !name.is_empty())
            .ok_or_else(|| ErrorBadRequest("Missing caller identity"))
    }
}

pub async fn authenticate(
    mut req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    let caller_field = match policy(req.method().as_str(), req.path()) {
        AuthPolicy::Public => return next.call(req).await,
        AuthPolicy::Token(field) => field,
    };

    let body = req.extract::<web::Bytes>().await?;

    let credentials: Credentials = serde_json::from_slice(&body)
        .map_err(|_| ErrorBadRequest("Invalid authentication payload"))?;

    let username = credentials.caller(caller_field)?.to_owned();

    if credentials.token.len() < 16
        || !credentials.token.is_char_boundary(16)
    {
        return Err(ErrorUnauthorized("Invalid token"));
    }

    let session = req
        .app_data::<web::Data<ScyllaSession>>()
        .cloned()
        .ok_or_else(|| ErrorInternalServerError("Missing database state"))?;

    let cache = req
        .app_data::<web::Data<MokaCache>>()
        .cloned()
        .ok_or_else(|| ErrorInternalServerError("Missing token cache"))?;

    let app_state = req
        .app_data::<web::Data<AppState>>()
        .cloned()
        .ok_or_else(|| ErrorInternalServerError("Missing app state"))?;

    // You need to release both locks before entering the handler
    
    let valid = {
        let session = session.lock.lock().await;
        let cache = cache.lock.lock().await;

        db::prelude::check_token(
            &session,
            &cache,
            credentials.token,
            Some(username.clone()),
            &app_state.metrics_collector,
        )
        .await
        .is_some()
    };

    if !valid {
        return Err(ErrorUnauthorized("Invalid token"));
    }

    req.extensions_mut()
        .insert(AuthenticatedUser { username });
    
    req.set_payload(body.into());
    next.call(req).await
}
