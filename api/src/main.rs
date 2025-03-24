mod api;
mod security;
mod db;
mod env;
use actix_web::{middleware::Logger};
//use crate::security::structures::ScyllaSession;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // obtaining the TLS certificate configuration
    let tls_config = security::obtain_tls_config();
    
    // connection to scylla-server
    let session = actix_web::web::Data::new(security::structures::ScyllaSession {
        lock: std::sync::Mutex::new(db::new_scylla_session("127.0.0.1:9042").await.expect(""))
    });

    /*let test_user: db::structures::User = db::structures::User {
        email: Some("test".to_string()),
        password_hash: Some("test".to_string()),
        key: Some("test".to_string()),
        bio: Some("test".to_string()),
        username: Some("test".to_string())
    }; 


    let scylla_session = session.lock.lock().unwrap();
    let i = db::insert_new_user(&scylla_session, test_user).await;
    match i {
        Ok(_) => println!("yes"),
        Err(e) => println!("{:?}", e),
    }
    */
    
    // setting up the API server
    let _ = actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .wrap(Logger::new("%a %{User-Agent}i %U"))
            .app_data(session.clone())
            .service(actix_files::Files::new("/cdn", "/root/cdn"))   // CDN route
            .service(api::get_test)                                  // test API route
            .service(api::new_user_login)                            // API route for signing up
            .service(api::try_login)
            .service(api::json_test)
            .service(api::get_channels)
    })
    .bind_rustls_0_23(("0.0.0.0", 1313),tls_config)?
    .workers(8)
    .run()
    .await;
    Ok(())
}
    
