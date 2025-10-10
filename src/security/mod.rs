use sha2::Digest;
use rand::prelude::*;

pub mod structures;
pub mod aes;

pub fn sha256(secret: String) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(secret.into_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn sha512(secret: String) -> String {   
    let mut hasher = sha2::Sha256::new();
    hasher.update(secret.into_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn token() -> String {
    let salt = uuid::Uuid::now_v7().to_string();
    println!("t{}", salt);
    
    let mut hasher = sha2::Sha256::new();
    
    hasher.update(
        format!(
            "{}", 
            salt, 
        ).to_string().into_bytes()
    );

    format!(
        "{:x}", 
        hasher.finalize()
    )
}

pub fn armor_token(plain_token: String) -> String {
    sha256(
        aes::encrypt(
            &aes::encrypt_with_key(
                &plain_token,
                &plain_token[..16]
            )
        )
    )
}

pub fn sid() -> String {
    format!(
        "{}{}",
        token(),
        rand::rng().random::<u64>()
    )
}

pub fn salt() -> String {
    let mut rng = rand::rng();
    (0..16).map(|_| rng.random_range::<u8, _>(33..127) as char).collect()
}
