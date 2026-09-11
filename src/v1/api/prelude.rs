
macro_rules! cache_metrics {
    ($col:ident) => {
        $col.metrics_collector.clone()
    };
}

macro_rules! scylla_session {
    ($session:ident) => {
        $session.lock.lock().await
    };
}

macro_rules! cache {
    ($shared_cache:ident) => {
        $shared_cache.lock.lock().await
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

    ($http:expr, $name:expr, $scylla_session:expr) => {
        match $http.match_info().get($name) {
            Some(param) => {
                if ! db::prelude::check_sid($scylla_session, param.to_string().clone()).await {
                    return actix_web::HttpResponse::NotFound().body(format!("Couldn't find that server. ({}) :(", param.to_string().clone()));
                }
                param.to_string()
            },
            None => {
                return actix_web::HttpResponse::BadRequest()
                    .body(format!("missing `{}` parameter", $name));
            }
        }
    };

    ($http:expr, $name:expr, $scylla_session:expr, $sid:expr) => {
        match $http.match_info().get($name) {
            Some(param) => {
                if ! db::prelude::check_channel_name($scylla_session, $sid.clone(), param.to_string().clone()).await {
                    return actix_web::HttpResponse::NotFound().body(format!("Couldn't find that channel. ({}) :(", param.to_string().clone()));
                }
                param.to_string()
            },
            None => {
                return actix_web::HttpResponse::BadRequest()
                    .body(format!("missing `{}` parameter", $name));
            }
        }
    };
}
