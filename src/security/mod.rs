use rand::prelude::*;
use sha2::Digest;
use password_hash::{PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng};

use ::function_name::named;

pub mod aes;
pub mod messages;
pub mod structures;

/// Check whether a plain text input matches an argon hash.
///
/// # Example usage
/// ```rs
/// if security::argon_check(&user_password_plain, &password_hash) {
///     // matched...
/// }
/// ```
pub fn argon_check(plain_text: &str, hash: &str) -> bool {
    match argon2::password_hash::PasswordHash::new(hash) {
        Ok(parsed_hash) => argon2::Argon2::default()
            .verify_password(plain_text.as_bytes(), &parsed_hash)
            .is_ok(),
        Err(_) => false,
    }
}

/// Hashes (using SHA-256) a secret.
pub fn sha256(secret: String) -> String {
    let mut hasher = sha2::Sha256::new();
    hasher.update(secret.into_bytes());
    format!("{:x}", hasher.finalize())
}

/// Creates a user token.
pub fn token() -> String {
    let salt = uuid::Uuid::now_v7().to_string();
    let mut hasher = sha2::Sha256::new();

    hasher.update(salt.clone().into_bytes());

    format!("{:x}", hasher.finalize())
}

/// Obfuscates token before storage.
pub fn armor_token(plain_token: &str) -> Result<String, aes::AesError> {
    let encrypted = aes::try_encrypt(&aes::try_encrypt_with_key(
        plain_token,
        &plain_token[..16],
    )?)?;
    Ok(sha256(encrypted))
}

#[named]
pub fn armor_token_logged(token: &str) -> Option<String> {
    match armor_token(token) {
        Ok(armored) => Some(armored),
        Err(err) => {
            crate::utils::logging::log(
                &format!("Failed to armor token: {err}"),
                Some(function_name!()),
            );
            None
        }
    }
}

/// Generates random server id.
pub fn sid() -> String {
    format!("{}{}", token(), rand::rng().random::<u64>())
}

/// Generates random salt.
pub fn salt() -> String {
    let mut rng = rand::rng();
    (0..16)
        .map(|_| rng.random_range::<u8, _>(33..127) as char)
        .collect()
}

/// Applies argon hashing to given input.
pub fn argon(secret: &str) -> Option<String> {
    let salt = SaltString::generate(&mut OsRng);
    // TODO: note the '?'... wtf
    Some(
        argon2::Argon2::default()
            .hash_password(
                secret.as_bytes(),
                &salt
            ).ok()?
        .to_string()
    )
}

#[cfg(test)]
mod tests {
    use super::{armor_token, sha256, argon, argon_check};

    #[test]
    fn test_token_armor() {
        const TEST_KEY: &str = "0123456789abcdef";
        const TEST_IV: &str = "fedcba9876543210";
        // SAFETY: tests run single-threaded; env is only read during AES calls.
        unsafe {
            std::env::set_var(crate::env::statics::OD_AES_KEY, TEST_KEY);
            std::env::set_var(crate::env::statics::OD_AES_IV, TEST_IV);
        }

        assert_eq!(
            "cadcfb296aab1c214b9b99fe01a649453efe18d41df4e3c6bb686fe71bb93695",
            armor_token("token12345678901234567890").expect("armor_token should succeed")
        );
    }

    #[test]
    fn test_sha256() {
        assert_eq!(
            "2bb80d537b1da3e38bd30361aa855686bde0eacd7162fef6a25fe97bf527a25b",
            sha256("secret".to_string())
        );
    }

    #[test]
    fn test_argon2() {
        let plain_text_secret: String = "pre_hash".to_string();
        let argon_hash: String = argon(&plain_text_secret).expect(
            "Argon2 failed to create a proper hash. Check src/security/mod.rs:argon()"
        );

        assert!(argon_check(&plain_text_secret, &argon_hash));
    }
}
