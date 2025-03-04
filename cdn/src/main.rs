use actix_web;
use actix_files;

use std::io::BufReader;
use std::fs::File;

mod api;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let mut cert_file = BufReader::new(File::open("/etc/letsencrypt/live/onlinedi.vision/cert.pem").unwrap());
    let mut priv_key = BufReader::new(File::open("/etc/letsencrypt/live/onlinedi.vision/privkey.pem").unwrap());
    
    println!("{:?} \n\n {:?}", cert_file, priv_key);

    let tls_certificates = rustls_pemfile::certs(&mut cert_file)
        .collect::<Result<Vec<_>,_>>()
        .unwrap();
    let tls_key = rustls_pemfile::pkcs8_private_keys(&mut priv_key)
        .next()
        .unwrap()
        .unwrap();

    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certificates, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
        .unwrap();

    let _ = actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .service(actix_files::Files::new("/cdn", "/root/cdn"))
    })
    .bind_rustls_0_23(("0.0.0.0", 1313),tls_config)?
    .workers(8)
    .run()
    .await;
    Ok(())
}
    
