use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use hex;

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

use crate::env;

#[inline(always)]
pub fn encrypt_with_key(plaintext: &str, key: &str) -> String {
    let iv_bytes = env::get_env_var(env::statics::OD_AES_IV);
    let cipher = Aes128CbcEnc::new_from_slices(key.as_bytes(), iv_bytes.as_bytes());
    let mut buffer = plaintext.as_bytes().to_vec();
    let pos = plaintext.len();
    let block_size = 16;
    let padding_needed = block_size - (pos % block_size);
    buffer.resize(pos + padding_needed, 0);
    let ciphertext = cipher
        .expect("Failed to load cipher for encrypt_with_key.")
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, pos)
        .unwrap();
    hex::encode(ciphertext)
}

#[inline(always)]
pub fn encrypt(plaintext: &str) -> String {
    encrypt_with_key(plaintext, &env::get_env_var(env::statics::OD_AES_KEY))
}

#[inline(always)]
pub fn decrypt_with_key(ciphertext: &str, key: &str) -> String {
    let key_bytes = key.to_string();
    let iv_bytes = env::get_env_var(env::statics::OD_AES_IV);
    let mut ciphertext_bytes = hex::decode(ciphertext).unwrap();
    let cipher = Aes128CbcDec::new_from_slices(key_bytes.as_bytes(), iv_bytes.as_bytes());
    let plaintext = cipher
        .expect("Failed to load cipher for decrypt_with_key.")
        .decrypt_padded_mut::<Pkcs7>(&mut ciphertext_bytes)
        .unwrap();
    String::from_utf8(plaintext.to_vec()).expect("Failed to obtain string from plaintext in decrypt_with_key")
}

#[inline(always)]
pub fn decrypt(ciphertext: &str) -> String {
    decrypt_with_key(ciphertext, &env::get_env_var(env::statics::OD_AES_KEY))
}
