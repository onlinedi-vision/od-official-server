mod api;
mod tls;
mod db;
mod env;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    
    let tls_config = tls::obtain_tls_config();

    let session = db::new_scylla_session("127.0.0.1:9042").await;

    let test_user: db::structures::User = db::structures::User {
        email: Some("test".to_string()),
        password_hash: Some("test".to_string()),
        key: Some("test".to_string()),
        bio: Some("test".to_string()),
        username: Some("test".to_string())
    }; 


    if let Ok(scylla_session) = session {
        println!("flag");
        let i = db::insert_new_user(&scylla_session, test_user).await;
        match i {
            Ok(_) => println!("yes"),
            Err(e) => println!("{:?}", e),
        }
    }
    /*let _ = actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .service(actix_files::Files::new("/cdn", "/root/cdn"))
            .service(api::get_test)
    })
    .bind_rustls_0_23(("0.0.0.0", 1313),tls_config)?
    .workers(8)
    .run()
    .await;*/
    Ok(())
}
    
