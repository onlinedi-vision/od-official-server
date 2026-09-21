use actix_web::{
        body::MessageBody,
        dev::{Payload, ServiceRequest, ServiceResponse},
        error::{ErrorBadRequest, ErrorInternalServerError, ErrorUnauthorized},
        middleware::Next,
        web, Error, FromRequest, HttMessage, HttpRequest,
    };
use serde::Deserialize;
use std::future::{ready, Ready};

use crate::{
    api::structures::Appstate,
    db,
    security::structures::{MokaCache, ScyllaSession},
};


#[derive(Clone)]
pub struct AuthenticatedUser {
    username: String,
}

impl AuthenticateUser {
    pub fn username(&self} -> &str {
        &self.username
    }
}



impl FromRequest for AutheticatedUser{
        type Error = Error;
        type Future = Ready<Result<Self, Self::Error>>;

        fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
                ready(
                    req.extensions()
                        .get::<Self>()
                        .cloned()
                        .ok_or_else(|| { ErrorInternalServerError(
                                "Authenticated route is misssing authentication middleware",
                        )
                        }),
                )
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

fn policy(method: &str, path: &str) ->AuthPolicy {
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
            AuthPoicy::Token(CallerField::Recipient)
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


//commpatibity from here
