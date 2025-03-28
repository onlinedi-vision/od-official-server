mod api;
mod security;
mod db;
mod env;
use actix_web::{middleware::Logger};

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // obtaining the TLS certificate configuration
    let tls_config = security::obtain_tls_config();
    
    // connection to scylla-server
    let session = actix_web::web::Data::new(security::structures::ScyllaSession {
        lock: std::sync::Mutex::new(db::new_scylla_session("127.0.0.1:9042").await.expect(""))
    });

    // setting up the API server
    let _ = actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .wrap(Logger::new("%a %{User-Agent}i %U"))
            .app_data(session.clone())                              // sharing scyllaDB session
            .service(actix_files::Files::new("/cdn", "/root/cdn"))  // CDN route
            .service(api::user::new_user_login)                     // API route for signing up
            .service(api::user::try_login)
            .service(api::channel::get_channels)
            .service(api::message::get_channel_messages)
            .service(api::message::send_message)
    })
    .bind_rustls_0_23(("0.0.0.0", 1313),tls_config)?
    .workers(8)
    .run()
    .await;
    Ok(())
}
    
