#![allow(unused_variables)]

mod structures;
use crate::security;
use crate::api::structures::TokenHolder;
use crate::db;

#[actix_web::get("/api/test")]
pub async fn get_test(
    _: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Query<structures::TestParamsStruct>
) -> impl actix_web::Responder {  
    println!("working");
    actix_web::HttpResponse::Ok().body(
        format!("{:?}{:?}", req.param1, req.param2)
    )
}

#[actix_web::post("/api/json_test")]
pub async fn json_test(bytes: actix_web::web::Bytes) -> impl actix_web::Responder {
    println!("{:?}", std::str::from_utf8(&bytes[..]));
    actix_web::HttpResponse::Ok()
}

#[actix_web::post("/api/new_user")]
pub async fn new_user_login(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    form: actix_web::web::Json<structures::NewUser>
) -> impl actix_web::Responder {
    println!("test"); 
    let password_hash = security::sha512(form.password.clone());
    let mut token_holder = structures::TokenHolder {
        token: security::token()
    };
    let user_instance = db::structures::User::new(
        form.username.clone(),
        form.email.clone(),
        password_hash.clone(),
        token_holder.token.clone()
    );
    
    let scylla_session = session.lock.lock().unwrap();
    match db::insert_new_user(&scylla_session, user_instance).await {
        None => {
            token_holder.token = "".to_string();
            return actix_web::HttpResponse::Ok().json(
                &token_holder
            );
        },
        Some(_) => {
            return actix_web::HttpResponse::Ok().json(
                &token_holder
            );
        }
    }
}

#[actix_web::post("/api/try_login")]
pub async fn try_login(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    form: actix_web::web::Json<structures::LoginUser>
) -> impl actix_web::Responder {

    let new_token_holder = structures::TokenHolder {
        token: security::token()
    };
    let username = db::structures::UserUsername {
        username: Some(form.username.clone())
    };
    let scylla_session = session.lock.lock().unwrap();
    match db::get_user_password_hash(&scylla_session, username).await {
        Some(password_hash) => {
            let user_password_hash = security::sha512(form.password.clone());
            if user_password_hash == password_hash {
                let _ = db::update_user_key(
                    &scylla_session, 
                    db::structures::KeyUser{
                        key: Some(new_token_holder.token.clone()), 
                        username: Some(form.username.clone())
                    }
                ).await;
                actix_web::HttpResponse::Ok().json(
                    &new_token_holder
                )
            } else {
                println!("not matchy");
                actix_web::HttpResponse::Ok().json(
                    &structures::TokenHolder {
                        token: "".to_string()
                    }
                )
            }
        },
        None => {
            println!("no hash");
            actix_web::HttpResponse::Ok().json(
                &structures::TokenHolder {
                    token: "".to_string()
                }
            )
        }
    }   
}

#[actix_web::post("/servers/{server_name}/api/get_channels")]
pub async fn get_channels (
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<TokenHolder>,
    http: actix_web::HttpRequest
) -> impl actix_web::Responder {
    
    let server_name: String = http.match_info().get("server_name").unwrap().to_string();
    let scylla_session = session.lock.lock().unwrap();
    match db::check_token(&scylla_session, req.token.clone()).await {
        Some(_) => {
            
            match db::fetch_server_channels(&scylla_session, server_name).await {
                Some(channels) => {
                        return actix_web::HttpResponse::Ok().json(
                            &structures::Channels {
                                c_list: channels
                            }
                        );
                },
                None => {
                    return actix_web::HttpResponse::Ok().json(
                        &structures::Channels {
                            c_list: Vec::new()
                        }
                    );
                }
            }
        },
        _ => {
            return actix_web::HttpResponse::Ok().json(
                &structures::Channels {
                    c_list: Vec::new()
                }
            );
        }
    };
}

#[actix_web::get("/servers/{server_name}/api/get_messages")] 
pub async fn get_channel_messages(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Query<structures::TokenHolder>
) -> impl actix_web::Responder {

    actix_web::HttpResponse::Ok().json(
        &structures::TokenHolder {
            token: "".to_string()
        }
    )
}
