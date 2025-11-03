use sha2::Digest;
use rand::prelude::*;
use argon2::{
    PasswordHasher,
    PasswordVerifier
};

pub mod structures;
pub mod aes;

mod tests {
    #[allow(unused_imports)] // This is for `use super::*;` -- for some reason it doesn't like it without allowing used imports
    use super::*;
    
    #[test]
    fn test_token_armor() {
        assert_eq!("6249cbd6de7d01973b4a73eacc503c275ec9a92b49306b5a713e998ce2104182", armor_token("token12345678901234567890".to_string()));
    }

    #[test]
    fn test_sha256() {
        assert_eq!("2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b", sha256("secret".to_string()));
    }

    #[test]
    fn test_hashers(){
        let pre_hash_string: String = "pre_hash".to_string();
        assert_ne!(sha256(pre_hash_string.clone()), sha512(pre_hash_string));
    }

    #[test]
    fn test_argon2() {
        let plain_text_secret: String = "pre_hash".to_string();
        let argon_hash: String = argon(plain_text_secret.clone());

        assert!(argon_check(plain_text_secret, argon_hash));
    }
}

pub fn argon(secret:String) -> String {
    let salt = argon2::password_hash::SaltString::generate(
        &mut argon2::password_hash::rand_core::OsRng
    );
    
    match argon2::Argon2::default().hash_password(secret.as_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(_) => "".to_string()
    }
}

pub fn argon_check(plain_text: String, hash: String) -> bool {
    match argon2::password_hash::PasswordHash::new(&hash) {
        Ok(parsed_hash) => {
            argon2::Argon2::default()
                .verify_password(plain_text.as_bytes(), &parsed_hash)
                .is_ok()
        },
        Err(_) => false
    }
}

pub fn sha256(secret: String) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(secret.into_bytes());
    format!("{:x}", hasher.finalize())
}

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
