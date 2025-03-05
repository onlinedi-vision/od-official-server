
pub fn obtain_tls_config() -> rustls::ServerConfig {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    let mut cert_file = std::io::BufReader::new(std::fs::File::open("/etc/letsencrypt/live/onlinedi.vision/cert.pem").unwrap());
    let mut priv_key = std::io::BufReader::new(std::fs::File::open("/etc/letsencrypt/live/onlinedi.vision/privkey.pem").unwrap());
    
    let tls_certificates = rustls_pemfile::certs(&mut cert_file)
        .collect::<Result<Vec<_>,_>>()
        .unwrap();
    let tls_key = rustls_pemfile::pkcs8_private_keys(&mut priv_key)
        .next()
        .unwrap()
        .unwrap();

    rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(tls_certificates, rustls::pki_types::PrivateKeyDer::Pkcs8(tls_key))
        .unwrap()
}
