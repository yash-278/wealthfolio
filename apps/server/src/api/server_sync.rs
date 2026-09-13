//! Authenticated owner sync routes. No Wealthfolio Connect session is involved.
use crate::{
    error::{ApiError, ApiResult},
    main_lib::AppState,
};
use axum::{
    extract::{DefaultBodyLimit, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;
use wealthfolio_core::{
    errors::{Error, ValidationError},
    events::DomainEvent,
};
use wealthfolio_storage_sqlite::sync::app_sync::{ServerSyncPush, ServerSyncPushResult};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/server-sync", get(status).post(enable))
        .route("/server-sync/snapshot", get(snapshot))
        .route("/server-sync/changes", get(pull).post(push))
        .layer(DefaultBodyLimit::max(1_000_000))
}

fn require_auth(state: &AppState) -> ApiResult<()> {
    if state.auth.is_none() {
        return Err(ApiError::Forbidden(
            "Configure server authentication before enabling device sync".into(),
        ));
    }
    Ok(())
}

// Storage errors can include payload values; never expose those through sync responses.
fn sync_error(error: Error) -> ApiError {
    match error {
        Error::Validation(ValidationError::InvalidInput(message)) => ApiError::BadRequest(message),
        _ => ApiError::Internal("Server sync operation failed".into()),
    }
}

async fn status(State(state): State<Arc<AppState>>) -> ApiResult<Response> {
    require_auth(&state)?;
    Ok(Json(
        state
            .app_sync_repository
            .server_sync_head()
            .map_err(sync_error)?,
    )
    .into_response())
}

async fn enable(State(state): State<Arc<AppState>>) -> ApiResult<Response> {
    require_auth(&state)?;
    Ok(Json(
        state
            .app_sync_repository
            .enable_server_sync()
            .await
            .map_err(sync_error)?,
    )
    .into_response())
}

async fn snapshot(State(state): State<Arc<AppState>>) -> ApiResult<Response> {
    require_auth(&state)?;
    if state
        .app_sync_repository
        .server_sync_head()
        .map_err(sync_error)?
        .enabled
        != 1
    {
        return Err(ApiError::BadRequest(
            "Enable server sync before downloading a snapshot".into(),
        ));
    }
    let (bytes, head) = state
        .app_sync_repository
        .export_server_sync_snapshot()
        .await
        .map_err(sync_error)?;
    Ok((
        [
            (header::CONTENT_TYPE, "application/vnd.sqlite3".to_string()),
            (header::CACHE_CONTROL, "no-store".to_string()),
            ("x-sync-server-id".parse().unwrap(), head.server_id),
            ("x-sync-cursor".parse().unwrap(), head.cursor.to_string()),
            ("x-sync-protocol-version".parse().unwrap(), "1".to_string()),
        ],
        bytes,
    )
        .into_response())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PullQuery {
    server_id: String,
    cursor: i64,
    #[serde(default = "page_size")]
    limit: i64,
}
fn page_size() -> i64 {
    100
}

async fn pull(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PullQuery>,
) -> ApiResult<Response> {
    require_auth(&state)?;
    let repo = Arc::clone(&state.app_sync_repository);
    let page = tokio::task::spawn_blocking(move || {
        repo.pull_server_sync(&query.server_id, query.cursor, query.limit)
    })
    .await
    .map_err(|_| ApiError::Internal("Server sync read failed".into()))?
    .map_err(sync_error)?;
    Ok(([(header::CACHE_CONTROL, "no-store")], Json(page)).into_response())
}

async fn push(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ServerSyncPush>,
) -> ApiResult<Response> {
    require_auth(&state)?;
    let result = state
        .app_sync_repository
        .push_server_sync(request)
        .await
        .map_err(sync_error)?;
    let status = match &result {
        ServerSyncPushResult::Conflict { .. } => StatusCode::CONFLICT,
        ServerSyncPushResult::Applied { .. } => {
            state
                .domain_event_sink
                .emit(DomainEvent::device_sync_pull_complete());
            StatusCode::OK
        }
        ServerSyncPushResult::Duplicate { .. } => StatusCode::OK,
    };
    Ok((status, [(header::CACHE_CONTROL, "no-store")], Json(result)).into_response())
}

/// Snapshot replacement by the other sync service would bypass this server's journal.
#[cfg(feature = "device-sync")]
pub(super) async fn require_connect_sync_mode(
    State(state): State<Arc<AppState>>,
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> ApiResult<Response> {
    if state
        .app_sync_repository
        .server_sync_head()
        .map_err(sync_error)?
        .enabled
        == 1
    {
        return Err(ApiError::Forbidden(
            "Wealthfolio Connect device sync is unavailable while server sync is enabled".into(),
        ));
    }
    Ok(next.run(request).await)
}
