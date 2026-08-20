use crate::security;
use crate::security::aes::AesError;

pub fn decrypt(enc_message: &str, salt: &str) -> Result<String, AesError> {
    let dec_salt = security::aes::try_decrypt(salt)?;
    let mush = security::aes::try_decrypt(enc_message)?;
    security::aes::try_decrypt_with_key(&mush, &dec_salt)
}

pub fn encrypt(plain_message: &str, salt: &str) -> Result<(String, String), AesError> {
    let mush = security::aes::try_encrypt_with_key(plain_message, salt)?;
    Ok((
        security::aes::try_encrypt(&mush)?,
        security::aes::try_encrypt(salt)?,
    ))
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    fn set_test_aes_env() {
        // SAFETY: tests run single-threaded; env is only read during AES calls.
        unsafe {
            std::env::set_var(crate::env::statics::OD_AES_KEY, TEST_KEY);
            std::env::set_var(crate::env::statics::OD_AES_IV, TEST_IV);
        }
    }

    #[test]
    fn test_message_encryption() {
        set_test_aes_env();
        let plain_message = "test";

        let (enc_message, enc_salt) =
            encrypt(plain_message, &security::salt()).expect("encrypt should succeed");
        let dec_message = decrypt(&enc_message, &enc_salt).expect("decrypt should succeed");

        assert_eq!(plain_message, dec_message);
    }
}
