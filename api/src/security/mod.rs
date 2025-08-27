use sha2::Digest;
use rand::prelude::*;

pub mod structures;


pub fn sha512(secret: String) -> String {   
    let mut hasher = sha2::Sha512::new();
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

pub fn sid() -> String {
    format!(
        "{}{}",
        token(),
        rand::rng().random::<u64>()
    )
}

pub fn salt() -> String {
    
}
