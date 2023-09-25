mod error;
pub mod pwd;

use hmac::{Hmac, Mac};
use sha2::Sha512;

pub use error::{Error, Result};

pub struct EncryptContent {
    pub content: String, // clear content
    pub salt: String,    // clear salt
}

pub fn encrypt_into_b64u(key: &[u8], enc_content: &EncryptContent) -> Result<String> {
    let EncryptContent { content, salt } = enc_content;

    // -- Create a HMAC-SHA-512 from key.
    let mut hmac_sha512 = Hmac::<Sha512>::new_from_slice(key).map_err(|_| Error::KeyFailHmac)?;

    // -- Add content
    hmac_sha512.update(content.as_bytes());
    hmac_sha512.update(salt.as_bytes());

    // -- Finalize and b64u encode.
    let hmac_result = hmac_sha512.finalize();
    let result_bytes = hmac_result.into_bytes();

    let result = base64_url::encode(&result_bytes);

    Ok(result)
}

// region:    ---Tests
#[cfg(test)]
mod tests {
    #![allow(unused)]
    use super::*;
    use anyhow::Result;
    use rand::RngCore;

    #[test]
    fn test_encrypt_into_b64u_ok() -> Result<()> {
        // -- Setup & Fixtures
        let mut fx_key = [0u8; 64];
        rand::thread_rng().fill_bytes(&mut fx_key);
        let fx_enc_context = EncryptContent {
            content: "hello world".to_string(),
            salt: "some pepper".to_string(),
        };

        // TODO: Need to fix fx_key, and precompute fx_enc_context.
        let fx_result = encrypt_into_b64u(&fx_key, &fx_enc_context)?;
        // print!("fx_result: {}", fx_result);
        // Y8LiKaQ0z2Fpx7KGt-B4-byfjd5kVIRtnFvuSS7Vk9A-gRqgFmbdkAw1iI3TnPG9oCQJSTlQz574Zn8EjfMGqQtest

        // -- Exec
        let res = encrypt_into_b64u(&fx_key, &fx_enc_context)?;

        // -- Check
        assert_eq!(fx_result, res);

        Ok(())
    }
}
// endregion: ---Tests
