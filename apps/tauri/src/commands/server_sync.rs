use crate::context::ServiceContext;
use std::sync::Arc;
use tauri::State;
use wealthfolio_core::events::DomainEvent;
use wealthfolio_server_sync::{ServerSyncClient, SyncView};

#[tauri::command]
pub fn own_server_sync_status(
    client: State<'_, Arc<ServerSyncClient>>,
) -> Result<SyncView, String> {
    client.view()
}
#[tauri::command]
pub async fn own_server_sync_connect(
    client: State<'_, Arc<ServerSyncClient>>,
    context: State<'_, Arc<ServiceContext>>,
    endpoint: String,
    password: String,
) -> Result<SyncView, String> {
    let result = client.connect(endpoint, password).await?;
    context
        .domain_event_sink
        .emit(DomainEvent::device_sync_pull_complete());
    Ok(result)
}
#[tauri::command]
pub async fn own_server_sync_run(
    client: State<'_, Arc<ServerSyncClient>>,
    context: State<'_, Arc<ServiceContext>>,
) -> Result<SyncView, String> {
    let before = client.view()?.connection.map(|s| s.cursor);
    let result = client.sync().await?;
    if before != result.connection.as_ref().map(|s| s.cursor) {
        context
            .domain_event_sink
            .emit(DomainEvent::device_sync_pull_complete());
    }
    Ok(result)
}
#[tauri::command]
pub async fn own_server_sync_pause(
    client: State<'_, Arc<ServerSyncClient>>,
    paused: bool,
) -> Result<SyncView, String> {
    client.pause(paused).await
}
#[tauri::command]
pub async fn own_server_sync_accept_server(
    client: State<'_, Arc<ServerSyncClient>>,
    context: State<'_, Arc<ServiceContext>>,
    event_id: String,
    server_event_id: String,
) -> Result<SyncView, String> {
    let result = client.accept_server(event_id, server_event_id).await?;
    context
        .domain_event_sink
        .emit(DomainEvent::device_sync_pull_complete());
    Ok(result)
}
