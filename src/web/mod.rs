mod error;
pub mod mw_auth;
pub mod mw_res_map;
pub mod routes_login;
pub mod routes_static;

use crate::crypt::token::generate_web_token;
use tower_cookies::{Cookie, Cookies};

pub use self::error::ClientError;
pub use self::error::{Error, Result};

const AUTH_TOKEN: &str = "auth_token";

fn set_token_cookie(cookies: &Cookies, user: &str, salt: &str) -> Result<()> {
    let token = generate_web_token(user, salt)?;

    let mut cookie = Cookie::new(AUTH_TOKEN, token.to_string());
    cookie.set_path("/");
    cookie.set_http_only(true);

    cookies.add(cookie);

    Ok(())
}

fn remove_token_cookie(cookies: &Cookies) -> Result<()> {
    // create a new cookie
    let mut cookie = Cookie::named(AUTH_TOKEN);
    cookie.set_path("/");

    cookies.remove(cookie);

    Ok(())
}
