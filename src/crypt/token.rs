use std::{fmt::Display, str::FromStr};

use crate::{
    config,
    crypt::{encrypt_into_b64u, EncryptContent, Error, Result},
    utils::{b64u_decode, b64u_encode, now_utc, now_utc_plus_sec_str, parse_utc},
};

// String format: `ident_b64u.exp_b64u.sign_b64u`
#[derive(Debug)]
pub struct Token {
    pub ident: String,     // Identifier (username for example)
    pub exp: String,       // Expiration date in Rfc3339.
    pub sign_b64u: String, // Signature, base64url encoded.
}

impl FromStr for Token {
    type Err = Error;

    fn from_str(token_str: &str) -> Result<Self> {
        let splits: Vec<&str> = token_str.split(".").collect();

        if splits.len() != 3 {
            return Err(Error::TokenInvalidFormat);
        }

        let (ident_b64u, exp_b64u, sign_b64u) = (splits[0], splits[1], splits[2]);

        Ok(Token {
            ident: b64u_decode(ident_b64u).map_err(|_| Error::TokenCannotDecodeIdent)?,
            exp: b64u_decode(exp_b64u).map_err(|_| Error::TokenCannotDecodeExp)?,
            sign_b64u: sign_b64u.to_string(),
        })
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}.{}.{}",
            b64u_encode(&self.ident),
            b64u_encode(&self.exp),
            &self.sign_b64u
        )
    }
}

// region:    --- Web Token Gen and Validations

pub fn generate_web_token(user: &str, salt: &str) -> Result<Token> {
    let config = &config();
    _generate_token(user, config.TOKEN_DURATION_SEC, salt, &config.TOKEN_KEY)
}

pub fn validate_web_token(token: &Token, salt: &str) -> Result<()> {
    let config = &config();
    _validate_token_sign_and_exp(token, salt, &config.TOKEN_KEY)
}

// endregion: --- Web Token Gen and Validations

// _generate_token有下划线的原因是和pub的函数名相同，所有使用下划线，并不是私有的方法都要添加下划线
fn _generate_token(ident: &str, duration: f64, salt: &str, key: &[u8]) -> Result<Token> {
    let ident = ident.to_string();
    let exp = now_utc_plus_sec_str(duration);

    // -- Sign the two first components
    let sign_b64u = _token_sign_into_b64u(&ident, &exp, salt, key)?;
    Ok(Token {
        ident,
        exp,
        sign_b64u,
    })
}

fn _validate_token_sign_and_exp(origin_token: &Token, salt: &str, key: &[u8]) -> Result<()> {
    let new_sign_b64u = _token_sign_into_b64u(&origin_token.ident, &origin_token.exp, salt, key)?;

    if new_sign_b64u != origin_token.sign_b64u {
        return Err(Error::TokenSignatureNotMatching);
    }

    // -- Validate expiration.
    let origin_exp = parse_utc(&origin_token.exp).map_err(|_| Error::TokenExpNotIso)?;
    let now = now_utc();

    if now > origin_exp {
        return Err(Error::TokenExpired);
    }
    Ok(())
}

/// Create token signature from token parts
/// and salt.
fn _token_sign_into_b64u(ident: &str, exp: &str, salt: &str, key: &[u8]) -> Result<String> {
    let content = format!("{}.{}", b64u_encode(ident), b64u_encode(exp));
    let signature = encrypt_into_b64u(
        key,
        &EncryptContent {
            content,
            salt: salt.to_string(),
        },
    )?;
    Ok(signature)
}

// region:    ---Tests
#[cfg(test)]
mod tests {

    use std::{thread, time::Duration};

    use super::*;
    use anyhow::Result;

    #[test]
    fn test_token_display_ok() -> Result<()> {
        // Fixture
        let fx_token_str = "ZngtaWRlbnQtMDE.MjAyMy0wNS0xN1QxNTozMDowMFo.some-sign-b64u-encoded";
        let fx_token = Token {
            ident: "fx-ident-01".to_string(),
            exp: "2023-05-17T15:30:00Z".to_string(),
            sign_b64u: "some-sign-b64u-encoded".to_string(),
        };

        // -- Exec & Check
        assert_eq!(fx_token.to_string(), fx_token_str);

        Ok(())
    }

    #[test]
    fn test_token_from_str_ok() -> Result<()> {
        // Fixture
        let fx_token_str = "ZngtaWRlbnQtMDE.MjAyMy0wNS0xN1QxNTozMDowMFo.some-sign-b64u-encoded";
        let fx_token = Token {
            ident: "fx-ident-01".to_string(),
            exp: "2023-05-17T15:30:00Z".to_string(),
            sign_b64u: "some-sign-b64u-encoded".to_string(),
        };

        // -- Exec
        let token = fx_token_str.parse::<Token>()?;

        // -- Check
        assert_eq!(format!("{token:?}"), format!("{fx_token:?}"));

        Ok(())
    }

    #[test]
    fn test_validate_web_token_ok() -> Result<()> {
        // Fixture
        let fx_user = "user_one";
        let fx_salt = "pepper";
        let fx_duration_sec = 0.02; // 20ms
        let token_key = &config().TOKEN_KEY;
        let fx_token = _generate_token(fx_user, fx_duration_sec, fx_salt, token_key)?;

        // -- Exec
        thread::sleep(Duration::from_millis(10));
        let res = validate_web_token(&fx_token, fx_salt);

        // -- Check
        res?;

        Ok(())
    }

    #[test]
    fn test_validate_web_token_err_expired() -> Result<()> {
        // Fixture
        let fx_user = "user_one";
        let fx_salt = "pepper";
        let fx_duration_sec = 0.01; // 20ms
        let token_key = &config().TOKEN_KEY;
        let fx_token = _generate_token(fx_user, fx_duration_sec, fx_salt, token_key)?;

        // -- Exec
        thread::sleep(Duration::from_millis(20));
        let res = validate_web_token(&fx_token, fx_salt);

        // -- Check
        assert!(
            matches!(res, Err(Error::TokenExpired)),
            "Should have matched `Err(Error::TokenExpired)` but was `{res:?}`"
        );

        Ok(())
    }
}
// endregion: ---Tests
