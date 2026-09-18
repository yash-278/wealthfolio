use crate::context::ServiceContext;
use std::sync::Arc;
use tauri::State;
use wealthfolio_core::captures::{Capture, CaptureInput};
#[tauri::command]
pub async fn submit_capture(
    state: State<'_, Arc<ServiceContext>>,
    input: CaptureInput,
) -> Result<Capture, String> {
    state
        .capture_service
        .accept("application", input)
        .await
        .map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn get_capture(
    state: State<'_, Arc<ServiceContext>>,
    id: String,
) -> Result<Capture, String> {
    state
        .capture_service
        .get(&id, None)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Capture not found".into())
}

#[tauri::command]
pub async fn get_capture_reviews(
    state: State<'_, Arc<ServiceContext>>,
    page: Option<u32>,
    page_size: Option<u32>,
    reason: Option<String>,
) -> Result<Vec<Capture>, String> {
    state
        .capture_service
        .reviews(
            page.unwrap_or(0),
            page_size.unwrap_or(25),
            reason.as_deref(),
        )
        .map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn resolve_capture_review(
    state: State<'_, Arc<ServiceContext>>,
    id: String,
    resolution: wealthfolio_core::captures::ResolveReview,
) -> Result<Capture, String> {
    state
        .capture_service
        .resolve(&id, resolution)
        .await
        .map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn dismiss_capture_review(
    state: State<'_, Arc<ServiceContext>>,
    id: String,
    version: i64,
) -> Result<Capture, String> {
    state
        .capture_service
        .dismiss(&id, version)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_capture_settings(
    state: State<'_, Arc<ServiceContext>>,
) -> Result<wealthfolio_core::captures::settings::CaptureSettings, String> {
    state.capture_service.settings().map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn update_capture_settings(
    state: State<'_, Arc<ServiceContext>>,
    settings: wealthfolio_core::captures::settings::CaptureSettings,
) -> Result<(), String> {
    state
        .capture_service
        .configure(settings)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn retry_capture(
    state: State<'_, Arc<ServiceContext>>,
    id: String,
    version: i64,
) -> Result<Capture, String> {
    state
        .capture_service
        .retry(&id, version)
        .await
        .map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn clear_capture_source(
    state: State<'_, Arc<ServiceContext>>,
    id: String,
    version: i64,
) -> Result<Capture, String> {
    state
        .capture_service
        .clear_source(&id, version)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_capture_usage(
    state: State<'_, Arc<ServiceContext>>,
) -> Result<wealthfolio_core::captures::settings::CaptureUsage, String> {
    state.capture_service.usage().map_err(|e| e.to_string())
}
