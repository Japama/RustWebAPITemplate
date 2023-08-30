use crate::ctx::Ctx;
use crate::log::log_request;
use crate::web::error::Error;
use axum::http::{Method, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::get_service;
use axum::{Json, Router};
use serde_json::json;
use tower_http::services::ServeDir;
use tracing::debug;
use uuid::Uuid;

async fn main_response_mapper(
    ctx: Option<Ctx>,
    uri: Uri,
    req_method: Method,
    res: Response,
) -> Response {
    debug!(" {:<12}  - main_response_mapper", "RES_MAPPER");

    let uuid = Uuid::new_v4();

    // -- Get the eventual response error.
    let service_error = res.extensions().get::<Error>();
    let client_status_error = service_error.map(|se| se.client_status_and_error());

    // --if client error, build the new response.
    let error_response = client_status_error
        .as_ref()
        .map(|(status_code, client_error)| {
            let client_error_body = json!({
                "error": {
                    "type" : client_error.as_ref(),
                    "req_uuid": uuid.to_string(),
                }
            });
            println!("    ->> client_error_body{client_error_body}");

            // Build the new response from the client_error_vody
            (*status_code, Json(client_error_body)).into_response()
        });

    // Build and log the server log line.
    let client_error = client_status_error.unzip().1;
    log_request(uuid, req_method, uri, ctx, service_error, client_error).await;

    println!();
    res
}

fn routes_static() -> Router {
    Router::new().nest_service("/", get_service((ServeDir::new("./"))))
}
