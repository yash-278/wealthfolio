use crate::{
    error::{ApiError, ApiResult},
    main_lib::AppState,
};
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Extension, Json, Router,
};
use std::sync::Arc;
use wealthfolio_core::captures::{Capture, CaptureInput};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/captures/{id}/retry", post(retry))
        .route("/captures/{id}/source", axum::routing::delete(clear_source))
        .route("/quick-add/usage", get(usage))
        .route("/quick-add/settings", get(settings).put(configure))
        .route("/capture-reviews", get(reviews))
        .route("/capture-reviews/{id}/resolve", post(resolve))
        .route("/capture-reviews/{id}/dismiss", post(dismiss))
}
pub fn intake_router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/captures", post(accept))
        .route("/captures/{id}", get(receipt))
}
async fn accept(
    Extension(principal): Extension<super::capture_tokens::CapturePrincipal>,
    State(state): State<Arc<AppState>>,
    Json(input): Json<CaptureInput>,
) -> ApiResult<Json<Capture>> {
    Ok(Json(
        state
            .capture_service
            .accept(principal.0.as_deref().unwrap_or("application"), input)
            .await?,
    ))
}
async fn receipt(
    Extension(principal): Extension<super::capture_tokens::CapturePrincipal>,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> ApiResult<Json<Capture>> {
    state
        .capture_service
        .get(&id, principal.0.as_deref())?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

#[derive(serde::Deserialize)]
struct Version {
    version: i64,
}
#[derive(serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct ReviewQuery {
    page: u32,
    page_size: u32,
    reason: Option<String>,
}
impl Default for ReviewQuery {
    fn default() -> Self {
        Self {
            page: 0,
            page_size: 25,
            reason: None,
        }
    }
}
async fn reviews(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ReviewQuery>,
) -> ApiResult<Json<Vec<Capture>>> {
    Ok(Json(state.capture_service.reviews(
        query.page,
        query.page_size,
        query.reason.as_deref(),
    )?))
}
async fn dismiss(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<Version>,
) -> ApiResult<Json<Capture>> {
    Ok(Json(
        state.capture_service.dismiss(&id, body.version).await?,
    ))
}

async fn resolve(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<wealthfolio_core::captures::ResolveReview>,
) -> ApiResult<Json<Capture>> {
    Ok(Json(state.capture_service.resolve(&id, body).await?))
}

async fn settings(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<wealthfolio_core::captures::settings::CaptureSettings>> {
    Ok(Json(state.capture_service.settings()?))
}
async fn configure(
    State(state): State<Arc<AppState>>,
    Json(body): Json<wealthfolio_core::captures::settings::CaptureSettings>,
) -> ApiResult<Json<()>> {
    state.capture_service.configure(body).await?;
    Ok(Json(()))
}

async fn clear_source(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<Version>,
) -> ApiResult<Json<Capture>> {
    Ok(Json(
        state
            .capture_service
            .clear_source(&id, body.version)
            .await?,
    ))
}

async fn retry(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(body): Json<Version>,
) -> ApiResult<Json<Capture>> {
    Ok(Json(state.capture_service.retry(&id, body.version).await?))
}

async fn usage(
    State(state): State<Arc<AppState>>,
) -> ApiResult<Json<wealthfolio_core::captures::settings::CaptureUsage>> {
    Ok(Json(state.capture_service.usage()?))
}
