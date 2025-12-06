use rand::prelude::*;
use sha2::Digest;

pub mod aes;
pub mod messages;
pub mod structures;

mod tests {
    #[allow(unused_imports)]
    // This is for `use super::*;` -- for some reason it doesn't like it without allowing used imports
    use super::*;

    #[test]
    fn test_token_armor() {
        assert_eq!(
            "486ad2d394c6ceeb3c5e9939303e3329dd1edbe5e5e22fdeea6356acafe8a4fe",
            armor_token("token12345678901234567890".to_string())
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
    fn test_hashers() {
        let pre_hash_string: String = "pre_hash".to_string();
        assert_ne!(sha256(pre_hash_string.clone()), sha512(pre_hash_string));
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
    println!("t{salt}");

    let mut hasher = sha2::Sha256::new();

    hasher.update(salt.to_string().to_string().into_bytes());

    format!("{:x}", hasher.finalize())
}

pub fn armor_token(plain_token: String) -> String {
    sha256(aes::encrypt(&aes::encrypt_with_key(
        &plain_token,
        &plain_token[..16],
    )))
}

pub fn sid() -> String {
    format!("{}{}", token(), rand::rng().random::<u64>())
}

pub fn salt() -> String {
    let mut rng = rand::rng();
    (0..16)
        .map(|_| rng.random_range::<u8, _>(33..127) as char)
        .collect()
}
