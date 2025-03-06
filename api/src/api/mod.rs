mod structures;
use crate::security;
use crate::db;


#[actix_web::get("/api/test")]
pub async fn get_test(
    _: actix_web::web::Data<security::structures::ScyllaSession>,
    req: actix_web::web::Query<structures::TestParamsStruct>
) -> impl actix_web::Responder {  
    actix_web::HttpResponse::Ok().body(
        format!("{:?}{:?}", req.param1, req.param2)
    )
}

#[actix_web::post("/api/new_user")]
pub async fn new_user_login(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    form: actix_web::web::Form<structures::NewUser>
) -> impl actix_web::Responder {
    
    let password_hash = security::sha512(form.password.clone());
    let token_holder = structures::TokenHolder {
        token: security::token()
    };
    let user_instance = db::structures::User::new(
        form.username.clone(),
        form.email.clone(),
        password_hash.clone(),
        token_holder.token.clone()
    );
    
    let scylla_session = session.lock.lock().unwrap();
    let _ = db::insert_new_user(&scylla_session, user_instance).await;
    actix_web::HttpResponse::Ok().json(
        &token_holder
    )
}

#[actix_web::post("/api/try_login")]
pub async fn try_login(
    session: actix_web::web::Data<security::structures::ScyllaSession>,
    form: actix_web::web::Form<structures::LoginUser>
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
                }).await;
                actix_web::HttpResponse::Ok().json(
                    &new_token_holder
                )
            } else {
                actix_web::HttpResponse::Ok().json(
                    &structures::TokenHolder {
                        token: "".to_string()
                    }
                )
            }
        },
        None => {
            actix_web::HttpResponse::Ok().json(
                &structures::TokenHolder {
                    token: "".to_string()
                }
            )
        }
    }
    
}
