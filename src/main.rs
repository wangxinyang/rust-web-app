#![allow(unused)] // For early development.

// region:    --- Modules

mod config;
mod crypt;
mod ctx;
mod error;
mod log;
mod model;
mod utils;
mod web;

// Commented during early development
mod _dev_utils;

// pub use self::error::{Error, Result};
use self::error::{Error, Result};
use axum::response::Html;
use axum::routing::get;
// re-export something
pub use config::config;

use crate::model::ModelManager;
use crate::web::mw_auth::{mw_ctx_require, mw_ctx_resolve};
use crate::web::mw_res_map::mw_response_mapper;
use crate::web::{routes_login, routes_static};
use axum::{middleware, Router};
use std::net::SocketAddr;
use tower_cookies::CookieManagerLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

// endregion: --- Modules

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .without_time() // 2023-09-07T07:28:29.251142Z  INFO LISTENING    - 127.0.0.1:8080
        .with_target(false) // true-> INFO rust_web_app: LISTENING    - 127.0.0.1:8080
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    _dev_utils::init_dev().await;

    // Initialize ModelManager.
    let mm = ModelManager::new().await?;

    // -- Define Routes
    // let routes_rpc = rpc::routes(mm.clone())
    //   .route_layer(middleware::from_fn(mw_ctx_require));
    let routes_hello = Router::new()
        .route("/hello", get(|| async { Html("Hello, world!") }))
        .route_layer(middleware::from_fn(mw_ctx_require));

    let routes_all = Router::new()
        .merge(routes_login::routes(mm.clone()))
        .merge(routes_hello)
        // .nest("/api", routes_rpc)
        .layer(middleware::map_response(mw_response_mapper))
        .layer(middleware::from_fn_with_state(mm.clone(), mw_ctx_resolve))
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static::serve_dir());

    // region:    --- Start Server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("{:<12} - {addr}\n", "LISTENING");
    axum::Server::bind(&addr)
        .serve(routes_all.into_make_service())
        .await
        .unwrap();
    // endregion: --- Start Server

    Ok(())
}
