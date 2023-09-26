use crate::crypt::{pwd, EncryptContent};
use crate::ctx::Ctx;
use crate::model::user::{UserBmc, UserForLogin};
use crate::model::ModelManager;
use crate::web::{self, remove_token_cookie, Error, Result};
use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use tower_cookies::{Cookie, Cookies};
use tracing::debug;

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/api/login", post(api_login_handler))
        .route("/api/logoff", post(api_logoff_handler))
        .with_state(mm)
}

async fn api_login_handler(
    cookies: Cookies,
    mm: State<ModelManager>,
    // mm: ModelManager, 必须使用State
    Json(payload): Json<LoginPayload>,
) -> Result<Json<Value>> {
    debug!("{:<12} - api_login_handler", "HANDLER");

    /* // Implement real db/auth logic.
    if payload.username != "demo1" || payload.pwd != "welcome" {
        return Err(Error::LoginFail);
    } */
    let LoginPayload {
        username,
        pwd: pwd_clear, // 这里使用：冒号可以修改变量的名字
    } = payload;
    let ctx = Ctx::root_ctx();

    // Get the user
    let user: UserForLogin = UserBmc::first_by_username(&ctx, &mm, &username)
        .await?
        .ok_or(Error::LoginFailUsernameNotFound)?;
    let user_id = user.id;

    // -- Validate the password.
    let Some(pwd) = user.pwd else {
        return Err(Error::LoginFailUserHasNoPwd{ user_id });
    };

    pwd::validate_pwd(
        &EncryptContent {
            salt: user.pwd_salt.to_string(),
            content: pwd_clear,
        },
        &pwd,
    )
    .map_err(|_| Error::LoginFailPwdNotMatching { user_id })?;

    /* // Implement real auth-token generation/signature.
    cookies.add(Cookie::new(web::AUTH_TOKEN, "user-1.exp.sign")); */
    // -- set web token
    web::set_token_cookie(&cookies, &user.username, &user.token_salt.to_string())?;

    // Create the success body.
    let body = Json(json!({
        "result": {
            "success": true
        }
    }));

    Ok(body)
}

async fn api_logoff_handler(
    cookies: Cookies,
    Json(payload): Json<LogoffPayload>,
) -> Result<Json<Value>> {
    debug!("{:<12} - api_logoff_handler", "HANDLER");

    let should_logoff = payload.logoff;

    if should_logoff {
        remove_token_cookie(&cookies)?;
    }

    Ok(Json(json!({
        "result": {
            "logged_off": should_logoff
        }
    })))
}

#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    pwd: String,
}

#[derive(Debug, Deserialize)]
struct LogoffPayload {
    logoff: bool,
}
