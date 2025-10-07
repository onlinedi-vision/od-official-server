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

    let no_of_workers = match env::get_option_env_var("NO_OF_WORKERS") {
        Some(s_workers_count) => {
            if let Ok(workers_count) = s_workers_count.parse::<usize>() {
                workers_count
            } else {512}
        },
        None => 512,
    };

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
    .workers(no_of_workers)
    .run()
    .await;
    Ok(())
}

#[macro_use] extern crate time_test;
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_salting() {
        time_test!();
        
        let password_hash = security::sha512(
            security::aes::encrypt(
                &security::aes::encrypt_with_key(
                    &format!("{}{}", "1234567890123456".to_string(), "strong_password123".to_string()),
                    "1234567890123456"
                )
            )
        );
        assert_eq!(&password_hash, "f581032d7304d19b58afd055b92c883e9137cb0b6a7ff5549581f7ec0ae82e5fdaba6cc639b6631c9b56e44128eb86d476842727c1b4ffb3e668cc0e31d33166")
    }

    #[test]
    fn test_hashing() {
        time_test!();

        let hash = security::sha512("test".to_string());

        assert_eq!(&hash, "ee26b0dd4af7e749aa1a8ee3c10ae9923f618980772e473f8819a5d4940e0db27ac185f8a0e1d5f84f88bc887fd67b143732c304cc5fa9ad8e6f57f50028a8ff");
    }

    #[test]
    // #[ignore = "(WIP) Haven't yet found a way to check a async function within a non-async one... :|"]
    fn test_scylla_connection() {
        // let _scylla_connection = db::prelude::new_scylla_session("onlinedi.vision:9042").await;
        let tokio_result = tokio_test::block_on(async {
            db::prelude::new_scylla_session("onlinedi.vision:9042").await
        });
        println!("{:?}", tokio_result);
        if let Ok(_) = tokio_result {
            assert!(0 == 0);
        } else {
            assert!(1 == 0);
        }
    }
}   
