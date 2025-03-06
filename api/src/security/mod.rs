use sha2::Digest;
pub mod structures;
use rand::Rng;

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

pub fn sha512(secret: String) -> String {   
    let mut hasher = sha2::Sha512::new();
    hasher.update(secret.into_bytes());
    match std::str::from_utf8(&(hasher.finalize()[..])) {
        Ok(hash) => hash.to_string(),
        Err(_) => "".to_string()
    }
}

pub fn token() -> String {
    let salt = rand::rng().random::<i32>();
    let mut hasher = sha2::Sha256::new();
    hasher.update(format!("{}", salt).to_string().into_bytes());
    match std::str::from_utf8(&(hasher.finalize()[..])) {
        Ok(hash) => hash.to_string(),
        Err(_) => "".to_string()
    }
}
