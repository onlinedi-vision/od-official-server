use crate::db;
use crate::api::structures;
use crate::security;

#[actix_web::post("/api/new_user")]
pub async fn new_user_login(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::NewUser>
) -> impl actix_web::Responder {
    println!("test"); 
    let user_salt = security::salt();
    let password_salt = security::salt();
    let password_hash = security::sha512(
        security::aes::encrypt(
            &security::aes::encrypt_with_key(
                &format!("{}{}", user_salt.clone(), req.password.clone()),
                &password_salt
            )
        )
    );
    let token_holder = structures::TokenHolder {
        token: security::token()
    };
    let user_instance = db::structures::User::new(
        req.username.clone(),
        req.email.clone(),
        password_hash.clone(),
        token_holder.token.clone(),
        security::aes::encrypt(&user_salt),
        security::aes::encrypt(&password_salt)
    );
    
    let scylla_session = session.lock.lock().unwrap();
    match db::insert_new_user(&scylla_session, user_instance).await {
        None => {
            return actix_web::HttpResponse::Conflict().body("User already exists or insert failed");
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
    req: actix_web::web::Json<structures::LoginUser>
) -> impl actix_web::Responder {

    let new_token_holder = structures::TokenHolder {
        token: security::token()
    };
    let username = db::structures::UserUsername {
        username: Some(req.username.clone())
    };
    let scylla_session = session.lock.lock().unwrap();
    match db::get_user_password_hash(&scylla_session, username).await {
        Some(secrets) => {
            let password_hash = secrets[0].password_hash.clone().unwrap();
            let user_salt = secrets[0].user_salt.clone().unwrap();
            let password_salt = secrets[0].password_salt.clone().unwrap();
            let decrypted_user_salt = security::aes::decrypt(&user_salt);
            let decrypted_password_salt = security::aes::decrypt(&password_salt);        
            let user_password_hash = security::sha512(
                security::aes::encrypt(
                    &security::aes::encrypt_with_key(
                        &format!("{}{}", decrypted_user_salt.clone(), req.password.clone()),
                        &decrypted_password_salt
                    )
                )
            );
            if user_password_hash == password_hash {
                let _ = db::prelude::update_user_key(
                    &scylla_session, 
                    db::structures::KeyUser{
                        key: Some(new_token_holder.token.clone()), 
                        username: Some(req.username.clone())
                    }
                ).await;
                actix_web::HttpResponse::Ok().json(
                    &new_token_holder
                )
            } else {
                println!("not matchy");
                actix_web::HttpResponse::Unauthorized().body("Invalid username or password")
            }
        },
        _ => {
            println!("no hash");
            actix_web::HttpResponse::Unauthorized().body("Invalid username or password")
        }
    }   
}

#[actix_web::post("/api/token_login")]
pub async fn token_login(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::TokenLoginUser>
) -> impl actix_web::Responder {

    let new_token_holder = structures::TokenHolder {
        token: security::token()
    };
    let username = db::structures::UserUsername {
        username: Some(req.username.clone())
    };
    let scylla_session = session.lock.lock().unwrap();
    if let Some(_) = db::prelude::check_token(&scylla_session, req.token.clone(), Some(req.username.clone())).await {
        match db::get_user_password_hash(&scylla_session, username).await {
            Some(secrets) => {
                let password_hash = secrets[0].password_hash.clone().unwrap();
                let user_salt = secrets[0].user_salt.clone().unwrap();
                let password_salt = secrets[0].password_salt.clone().unwrap();
                let user_password_hash = security::sha512(req.password.clone());
                if user_password_hash == password_hash {
                    let _ = db::prelude::update_user_key(
                        &scylla_session, 
                        db::structures::KeyUser{
                            key: Some(new_token_holder.token.clone()), 
                            username: Some(req.username.clone())
                        }
                    ).await;
                    actix_web::HttpResponse::Ok().json(
                        &new_token_holder
                    )
                } else {
                    println!("not matchy");
                    actix_web::HttpResponse::Unauthorized().body("Invalid password")
                }
            },
            _ => {
                println!("no hash");
                actix_web::HttpResponse::Unauthorized().body("Invalid password")
            }
        }   
    } else {
        println!("no token");
        actix_web::HttpResponse::Unauthorized().body("Invalid or expired token")
    }
}

// !TODO: get_user_servers API
#[actix_web::post("/api/get_user_servers")] 
pub async fn get_user_servers(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Json<structures::TokenUser>
) ->impl actix_web::Responder {
    let new_token_holder = structures::TokenHolder {
        token: security::token()
    };
    let username = db::structures::UserUsername {
        username: Some(req.username.clone())
    };
    let scylla_session = session.lock.lock().unwrap();
    if let Some(_) = db::prelude::check_token(&scylla_session, req.token.clone(), Some(req.username.clone())).await {
        match db::server::fetch_user_servers(&scylla_session, req.username.clone()).await {
            Some(sids) => {
                    let _ = db::prelude::update_user_key(
                        &scylla_session, 
                        db::structures::KeyUser{
                            key: Some(new_token_holder.token.clone()), 
                            username: Some(req.username.clone())
                        }
                    ).await;
                    actix_web::HttpResponse::Ok().json(
                        &structures::ServersList {
                            token: new_token_holder.token.clone(),
                            s_list: sids
                        }
                    )
            },
            None => {
                println!("no hash");
                actix_web::HttpResponse::NotFound().body("No servers found for user")
            }
        }   
    } else {
        println!("no token");
        actix_web::HttpResponse::Unauthorized().body("Invalid or expired token")
    }
}
