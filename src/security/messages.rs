use crate::security;

pub fn decrypt(enc_message: &str, salt: &str) -> String {
    let dec_salt = &security::aes::decrypt(salt);
    let mush = &security::aes::decrypt(enc_message);
    return security::aes::decrypt_with_key(mush, dec_salt);
}

pub fn encrypt(plain_message: &str, salt: &str) -> (String, String) {
    let mush = security::aes::encrypt_with_key(plain_message, salt);
    return (security::aes::encrypt(&mush), security::aes::encrypt(salt));
}

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_message_encryption() {
        let plain_message = "test";

        let (enc_message, enc_salt) = encrypt(plain_message, &security::salt());
        let dec_message = decrypt(&enc_message, &enc_salt);

        assert_eq!(plain_message, dec_message);
    }
}
