macro_rules! scylla_session {
    ($session:ident) => {
        match $session.lock.lock() {
            Ok(guard) => guard,
            Err(_) => {
                return actix_web::HttpResponse::InternalServerError()
                    .body("Internal error: scylla session lock poisoned.");
            }
        }
    };
}

macro_rules! cache {
    ($shared_cache:ident) => {
        match $shared_cache.lock.lock() {
            Ok(guard) => guard,
            Err(_) => {
                return actix_web::HttpResponse::InternalServerError()
                    .body("Internal error: cache lock poisoned.");
            }
        }
    };
}

macro_rules! param {
    ($http:expr, $name:expr) => {
        match $http.match_info().get($name) {
            Some(param) => param.to_string(),
            None => {
                return actix_web::HttpResponse::BadRequest()
                    .body(format!("missing `{}` parameter", $name));
            }
        }
    };
}
