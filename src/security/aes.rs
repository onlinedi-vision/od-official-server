use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use hex;
use std::fmt;

type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

use crate::env;

const AES_BLOCK_SIZE: usize = 16;

#[derive(Debug)]
pub enum AesError {
    MissingEnvVar(&'static str),
    InvalidLength {
        name: &'static str,
        expected: usize,
        actual: usize,
    },
    CipherInit(&'static str),
    EncryptionFailed,
    InvalidHex(hex::FromHexError),
    DecryptionFailed,
    InvalidUtf8(std::string::FromUtf8Error),
}

impl fmt::Display for AesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingEnvVar(name) => {
                write!(f, "environment variable {name} is not set")
            }
            Self::InvalidLength {
                name,
                expected,
                actual,
            } => write!(
                f,
                "environment variable {name} must be {expected} bytes, got {actual}"
            ),
            Self::CipherInit(context) => {
                write!(f, "failed to initialize AES cipher for {context}")
            }
            Self::EncryptionFailed => f.write_str("AES encryption failed"),
            Self::InvalidHex(err) => write!(f, "ciphertext is not valid hex: {err}"),
            Self::DecryptionFailed => f.write_str("AES decryption failed"),
            Self::InvalidUtf8(err) => write!(f, "decrypted plaintext is not valid UTF-8: {err}"),
        }
    }
}

impl std::error::Error for AesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidHex(err) => Some(err),
            Self::InvalidUtf8(err) => Some(err),
            _ => None,
        }
    }
}

pub fn validate_config() -> Result<(), AesError> {
    validate_env_var_length(env::statics::OD_AES_KEY)?;
    validate_env_var_length(env::statics::OD_AES_IV)?;
    Ok(())
}

fn validate_env_var_length(name: &'static str) -> Result<(), AesError> {
    if let Some(value) = env::get_option_env_var(name) {
        let actual = value.len();
        if actual != AES_BLOCK_SIZE {
            return Err(AesError::InvalidLength {
                name,
                expected: AES_BLOCK_SIZE,
                actual,
            });
        }
        return Ok(());
    }
    Err(AesError::MissingEnvVar(name))
}

pub fn try_encrypt_with_key(plaintext: &str, key: &str) -> Result<String, AesError> {
    let iv_bytes = env::get_env_var(env::statics::OD_AES_IV);
    let cipher = Aes128CbcEnc::new_from_slices(key.as_bytes(), iv_bytes.as_bytes())
        .map_err(|_| AesError::CipherInit("encrypt_with_key"))?;
    let mut buffer = plaintext.as_bytes().to_vec();
    let pos = plaintext.len();
    let block_size = AES_BLOCK_SIZE;
    let padding_needed = block_size - (pos % block_size);
    buffer.resize(pos + padding_needed, 0);
    let ciphertext = cipher
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, pos)
        .map_err(|_| AesError::EncryptionFailed)?;
    Ok(hex::encode(ciphertext))
}

pub fn try_encrypt(plaintext: &str) -> Result<String, AesError> {
    try_encrypt_with_key(plaintext, &env::get_env_var(env::statics::OD_AES_KEY))
}

pub fn try_decrypt_with_key(ciphertext: &str, key: &str) -> Result<String, AesError> {
    let iv_bytes = env::get_env_var(env::statics::OD_AES_IV);
    let mut ciphertext_bytes = hex::decode(ciphertext).map_err(AesError::InvalidHex)?;
    let cipher = Aes128CbcDec::new_from_slices(key.as_bytes(), iv_bytes.as_bytes())
        .map_err(|_| AesError::CipherInit("decrypt_with_key"))?;
    let plaintext = cipher
        .decrypt_padded_mut::<Pkcs7>(&mut ciphertext_bytes)
        .map_err(|_| AesError::DecryptionFailed)?;
    String::from_utf8(plaintext.to_vec()).map_err(AesError::InvalidUtf8)
}

pub fn try_decrypt(ciphertext: &str) -> Result<String, AesError> {
    try_decrypt_with_key(ciphertext, &env::get_env_var(env::statics::OD_AES_KEY))
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_KEY: &str = "0123456789abcdef";
    const TEST_IV: &str = "fedcba9876543210";

    fn set_test_aes_env() {
        // SAFETY: tests run single-threaded; env is only read during AES calls.
        unsafe {
            std::env::set_var(env::statics::OD_AES_KEY, TEST_KEY);
            std::env::set_var(env::statics::OD_AES_IV, TEST_IV);
        }
    }

    #[test]
    fn validate_config_requires_sixteen_byte_env_vars() {
        set_test_aes_env();
        assert!(validate_config().is_ok());
    }

    #[test]
    fn try_encrypt_decrypt_round_trip() {
        set_test_aes_env();
        let plaintext = "phase-1-round-trip";
        let ciphertext = try_encrypt(plaintext).expect("encrypt should succeed");
        let decrypted = try_decrypt(&ciphertext).expect("decrypt should succeed");
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn try_decrypt_rejects_invalid_hex() {
        set_test_aes_env();
        let err = try_decrypt("not-hex").expect_err("invalid hex should fail");
        assert!(matches!(err, AesError::InvalidHex(_)));
    }

    #[test]
    fn try_decrypt_rejects_corrupt_ciphertext() {
        set_test_aes_env();
        let err = try_decrypt("deadbeef").expect_err("truncated ciphertext should fail");
        assert!(matches!(err, AesError::DecryptionFailed));
    }
}
