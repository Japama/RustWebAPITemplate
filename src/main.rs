#![allow(unused)] // For early development.

// region:    --- Modules

mod config;
mod ctx;
mod error;
mod log;
mod model;
mod web;
// #[cfg(test)] // Commented during early development.

pub use self::error::{Error, Result};
pub use config::config; // = -> use crate::config

use crate::model::ModelManager;
use axum::response::Html;
use axum::routing::get;
use axum::{middleware, Router};
use std::net::SocketAddr;
use tower_cookies::CookieManagerLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;

use crate::ctx::Ctx;
use crate::log::log_request;
use crate::web::routes_static;
use axum::extract::{Path, Query};
use axum::http::{Method, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::get_service;
use axum::Json;
use serde::Deserialize;
use serde_json::json;
use tower_http::services::ServeDir;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .without_time()
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // Initialize ModelManager
    let mm = ModelManager::new().await?;

    let routes_all = Router::new()
        .merge((routes_hello()))
        .merge(web::routes_login::routes())
        // .nest("/api", routes_apis)
        // .layer(middleware::map_response(main_response_mapper))
        .layer(middleware::from_fn_with_state(
            mm.clone(),
            web::mw_auth::mw_ctx_resolver,
        ))
        .layer(CookieManagerLayer::new())
        .fallback_service((routes_static::serve_dir()));

    // region:  Start Server
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("{:<12} - {addr} \n", "LISTENING");
    axum::Server::bind(&addr)
        .serve(routes_all.into_make_service())
        .await
        .unwrap();
    // endregion: --- Start Server

    Ok(())
}

// region: Routes Hello
fn routes_hello() -> Router {
    Router::new()
        .route("/hello", get(handler_hello))
        .route("/helloName/:name", get(handler_hello_name))
}

#[derive(Debug, Deserialize)]
struct HelloParams {
    name: Option<String>,
}
async fn handler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
    println!(" --> {:<12} - handler_hello - {params:?}", "HANDLER");
    let name = params.name.as_deref().unwrap_or("World!");
    Html(format!("Hello <strong>{name}</strong>"))
}

async fn handler_hello_name(Path(name): Path<String>) -> impl IntoResponse {
    println!(" --> {:<12} - handler_hello_name - {name:?}", "HANDLER");
    Html(format!("Hello <strong>{name}</strong>"))
}

// endregion: Handler Hello
