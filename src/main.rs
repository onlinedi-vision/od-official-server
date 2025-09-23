#![cfg_attr(rustfmt, rustfmt_skip)]
mod api;
mod security;
mod db;
mod env;

use actix_web::{middleware::Logger};

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // connection to scylla-server
    let session = actix_web::web::Data::new(security::structures::ScyllaSession {
        lock: std::sync::Mutex::new(db::prelude::new_scylla_session("onlinedi.vision:9042").await.expect(""))
    });

    // setting up the API server
    let _ = actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .wrap(Logger::new("%a %{User-Agent}i %U"))
            .app_data(session.clone())                                             // sharing scyllaDB session

            .service(api::get_api_version)
            .service(api::user::new_user_login)                     // API route for signing up
            .service(api::user::try_login)
            .service(api::user::get_user_servers)
 
            .service(api::server::create_server)                
            .service(api::server::join_server)                      // change token !!
            .service(api::server::get_server_users)                 
            .service(api::server::get_server_info)
            
            .service(api::invites::send_dm_invite)
            .service(api::invites::accept_dm_invite)
            .service(api::invites::reject_dm_invite)
            .service(api::invites::fetch_pending_dm_invites)

            .service(api::friends::fetch_friend_list)
            .service(api::friends::delete_friend)


            .service(api::channel::get_channels)
            .service(api::channel::create_channel)

            .service(api::message::get_channel_messages)
            .service(api::message::send_message)
            .service(api::message::get_channel_messages)


            .service(api::roles::add_server_role)
            .service(api::roles::assign_role_to_user)
            .service(api::roles::remove_role_from_user)
            .service(api::roles::fetch_server_roles)
            .service(api::roles::fetch_user_roles)
    })
    .bind(("0.0.0.0", env::get_env_var("API_PORT").parse()?))?
    .workers(32)
    .run()
    .await;
    Ok(())
}
    
