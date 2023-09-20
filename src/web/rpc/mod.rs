// region: Modules

mod activity_rpc;
mod task_rpc;

use crate::ctx::Ctx;
use crate::model::ModelManager;
use crate::web::rpc::activity_rpc::{
    create_activity, delete_activity, get_activity, list_activities, update_activity,
};
use crate::web::rpc::task_rpc::{create_task, delete_task, list_tasks, update_task};
use crate::web::{Error, Result};
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{from_value, json, to_value, Value};
use tracing::debug;

// endregion: Modules

// region: JSON-RPC Request Body

#[derive(Deserialize)]
struct RpcRequest {
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Deserialize)]
pub struct ParamsForCreate<D> {
    data: D,
}

#[derive(Deserialize)]
pub struct ParamsForUpdate<D> {
    id: i64,
    data: D,
}
#[derive(Deserialize)]
pub struct ParamsForUpdateMongo<D> {
    id: String,
    data: D,
}

#[derive(Deserialize)]
pub struct ParamsIded {
    id: i64,
}

#[derive(Deserialize)]
pub struct ParamsIdedMongo {
    id: String,
}

// endregion: JSON-RPC Request Body

pub fn routes(mm: ModelManager) -> Router {
    Router::new()
        .route("/rpc", post(rpc_handler))
        .with_state(mm)
}

async fn rpc_handler(
    State(mm): State<ModelManager>,
    ctx: Ctx,
    Json(rpc_req): Json<RpcRequest>,
) -> Response {
    // Create the RPC info to be set to the respose.extensions
    let rpc_info = RpcInfo {
        id: rpc_req.id.clone(),
        method: rpc_req.method.clone(),
    };

    // Exec & store RpcInfo in response.
    let mut res = _rpc_handler(ctx, mm, rpc_req).await.into_response();
    res.extensions_mut().insert(rpc_info);

    res
}

// RPC basic information holding the id and method for further loggin
#[derive(Debug)]
pub struct RpcInfo {
    pub id: Option<Value>,
    pub method: String,
}

macro_rules! exec_rpc_fn {
    // With Params
    ($rpc_fn:expr, $ctx:expr, $mm:expr, $rpc_params:expr) => {{
        let rpc_fn_name = stringify!($rpc_fn);
        let params = $rpc_params.ok_or(Error::RpcMissingParams {
            rpc_method: rpc_fn_name.to_string(),
        })?;

        let params = from_value(params).map_err(|_| Error::RpcFailJsonParams {
            rpc_method: rpc_fn_name.to_string(),
        })?;

        $rpc_fn($ctx, $mm, params).await.map(to_value)??
    }};

    // Without Params
    ($rpc_fn:expr, $ctx:expr, $mm:expr) => {
        $rpc_fn($ctx, $mm).await.map(to_value)??
    };
}

async fn _rpc_handler(ctx: Ctx, mm: ModelManager, rpc_req: RpcRequest) -> Result<Json<Value>> {
    let RpcRequest {
        id: rpc_id,
        method: rpc_method,
        params: rpc_params,
    } = rpc_req;

    debug!("{:<12} - rpc_handler - method: {rpc_method}", "HANDLER");

    let result_json: Value = match rpc_method.as_str() {
        // Task RPC methods
        "create_task" => exec_rpc_fn!(create_task, ctx, mm, rpc_params),
        "list_tasks" => exec_rpc_fn!(list_tasks, ctx, mm),
        "update_task" => exec_rpc_fn!(update_task, ctx, mm, rpc_params),
        "delete_task" => exec_rpc_fn!(delete_task, ctx, mm, rpc_params),

        // Activity RPC methods
        "create_activity" => exec_rpc_fn!(create_activity, ctx, mm, rpc_params),
        "get_activity" => exec_rpc_fn!(get_activity, ctx, mm, rpc_params),
        "list_activities" => exec_rpc_fn!(list_activities, ctx, mm),
        "update_activity" => exec_rpc_fn!(update_activity, ctx, mm, rpc_params),
        "delete_activity" => exec_rpc_fn!(delete_activity, ctx, mm, rpc_params),

        // Fallback as Err
        _ => return Err(Error::RpcMethodUnknown(rpc_method)),
    };

    let body_response = json!({
        "id": rpc_id,
        "result": result_json
    });

    Ok(Json(body_response))
}
