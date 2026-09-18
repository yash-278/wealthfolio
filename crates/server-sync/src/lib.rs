//! Shared native client for owner-operated Wealthfolio servers.
use reqwest::{
    header::{COOKIE, SET_COOKIE},
    Client, Response, Url,
};
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    sync::{Arc, Mutex},
    time::Duration,
};
use wealthfolio_core::secrets::SecretStore;
use wealthfolio_storage_sqlite::sync::app_sync::{
    AppSyncRepository, ServerClientStatus, ServerSyncHead, ServerSyncPage, ServerSyncPushResult,
};

const SESSION_KEY: &str = "own_server_sync_session";
#[derive(Serialize, Deserialize)]
struct Session {
    endpoint: String,
    cookie: String,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConflictSummary {
    pub event_id: String,
    pub server_event_id: String,
    pub label: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncView {
    pub connection: Option<ServerClientStatus>,
    pub error: Option<String>,
    pub conflict: Option<ConflictSummary>,
}

pub struct ServerSyncClient {
    repo: Arc<AppSyncRepository>,
    secrets: Arc<dyn SecretStore>,
    http: Client,
    gate: tokio::sync::Mutex<()>,
    error: Mutex<Option<String>>,
}
fn local_error(_: impl std::fmt::Display) -> String {
    "Could not update local sync data".into()
}
fn network_error(_: reqwest::Error) -> String {
    "Server unavailable. Your pending edits remain on this device".into()
}

pub fn validate_endpoint(value: &str) -> Result<String, String> {
    let url = Url::parse(value.trim()).map_err(|_| "Enter a valid HTTPS server address")?;
    let local_debug = cfg!(debug_assertions)
        && url.scheme() == "http"
        && matches!(url.host_str(), Some("localhost" | "127.0.0.1" | "[::1]"));
    if (url.scheme() != "https" && !local_debug)
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path() != "/"
    {
        return Err(
            "Use an HTTPS server origin without a path, query, or embedded credentials".into(),
        );
    }
    Ok(url.as_str().trim_end_matches('/').into())
}
async fn bytes(mut response: Response, max: usize) -> Result<Vec<u8>, String> {
    if response.status() == 401 {
        return Err("Your server session expired. Sign in again".into());
    }
    if !response.status().is_success() {
        return Err(format!(
            "Server rejected the request ({})",
            response.status().as_u16()
        ));
    }
    let mut data = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(network_error)? {
        if data.len().saturating_add(chunk.len()) > max {
            return Err("The server response exceeds the supported size".into());
        }
        data.extend_from_slice(&chunk);
    }
    Ok(data)
}
async fn json<T: serde::de::DeserializeOwned>(response: Response) -> Result<T, String> {
    serde_json::from_slice(&bytes(response, 8_000_000).await?)
        .map_err(|_| "The server returned an incompatible sync response".into())
}

impl ServerSyncClient {
    pub fn new(
        repo: Arc<AppSyncRepository>,
        secrets: Arc<dyn SecretStore>,
    ) -> Result<Self, String> {
        Ok(Self {
            repo,
            secrets,
            http: Client::builder()
                .timeout(Duration::from_secs(60))
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(network_error)?,
            gate: tokio::sync::Mutex::new(()),
            error: Mutex::new(None),
        })
    }
    pub fn view(&self) -> Result<SyncView, String> {
        let conflict = self
            .repo
            .next_server_change()
            .map_err(local_error)?
            .and_then(|queued| {
                queued.conflict.map(|remote| ConflictSummary {
                    event_id: queued.request.event_id,
                    server_event_id: remote.event_id,
                    label: queued
                        .request
                        .payload
                        .get("title")
                        .or_else(|| queued.request.payload.get("name"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("Record with pending edits")
                        .to_owned(),
                })
            });
        Ok(SyncView {
            connection: self.repo.server_client_status().map_err(local_error)?,
            error: self.error.lock().unwrap().clone(),
            conflict,
        })
    }
    pub async fn connect(&self, endpoint: String, password: String) -> Result<SyncView, String> {
        let _guard = self.gate.lock().await;
        let endpoint = validate_endpoint(&endpoint)?;
        let current = self.repo.server_client_status().map_err(local_error)?;
        if let Some(current) = &current {
            if current.endpoint != endpoint {
                return Err(
                    "This device is paired with another server. Existing data will not be replaced"
                        .into(),
                );
            }
        } else {
            self.repo.require_empty_server_client().map_err(|_| {
                "Connect from a fresh installation. Existing local records will not be overwritten"
            })?;
        }
        let login = self
            .http
            .post(format!("{endpoint}/api/v1/auth/login"))
            .json(&serde_json::json!({"password":password}))
            .send()
            .await
            .map_err(network_error)?;
        if !login.status().is_success() {
            return Err("Could not sign in. Check the server address and password".into());
        }
        let cookie = login
            .headers()
            .get_all(SET_COOKIE)
            .iter()
            .filter_map(|h| h.to_str().ok())
            .filter_map(|s| s.split(';').next())
            .find(|s| s.starts_with("wf_session=") && s.len() > 11)
            .ok_or("The server did not return an owner session")?
            .to_owned();
        let session = Session {
            endpoint: endpoint.clone(),
            cookie,
        };
        let head: ServerSyncHead = json(
            self.http
                .post(format!("{endpoint}/api/v1/server-sync"))
                .header(COOKIE, &session.cookie)
                .send()
                .await
                .map_err(network_error)?,
        )
        .await?;
        if let Some(current) = current {
            if current.server_id != head.server_id {
                return Err(
                    "Server identity changed. Existing local data has been preserved".into(),
                );
            }
        } else {
            let response = self
                .http
                .get(format!("{endpoint}/api/v1/server-sync/snapshot"))
                .header(COOKIE, &session.cookie)
                .send()
                .await
                .map_err(network_error)?;
            if response
                .headers()
                .get("x-sync-protocol-version")
                .and_then(|h| h.to_str().ok())
                != Some("1")
                || response
                    .headers()
                    .get("x-sync-server-id")
                    .and_then(|h| h.to_str().ok())
                    != Some(&head.server_id)
            {
                return Err("Server snapshot identity or protocol is incompatible".into());
            }
            let cursor: i64 = response
                .headers()
                .get("x-sync-cursor")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok())
                .filter(|v| *v >= 0)
                .ok_or("Invalid snapshot cursor")?;
            let data = bytes(response, 100_000_000).await?;
            if !data.starts_with(b"SQLite format 3\0") {
                return Err("Invalid server snapshot".into());
            }
            let mut file = tempfile::NamedTempFile::new().map_err(local_error)?;
            file.write_all(&data).map_err(local_error)?;
            self.repo
                .bootstrap_server_client(
                    file.path().to_string_lossy().into_owned(),
                    endpoint,
                    head.server_id,
                    cursor,
                )
                .await
                .map_err(|_| {
                    "Could not import the server snapshot. Existing local data was preserved"
                })?;
        }
        // The password is never persisted; the owner session is stored in the OS credential store.
        self.secrets
            .set_secret(
                SESSION_KEY,
                &serde_json::to_string(&session).map_err(local_error)?,
            )
            .map_err(|_| "Could not save the server session in the device credential store")?;
        self.repo
            .pause_server_client(false)
            .await
            .map_err(local_error)?;
        *self.error.lock().unwrap() = None;
        self.view()
    }
    pub async fn pause(&self, paused: bool) -> Result<SyncView, String> {
        let _guard = self.gate.lock().await;
        self.repo
            .pause_server_client(paused)
            .await
            .map_err(local_error)?;
        self.view()
    }
    pub async fn sync(&self) -> Result<SyncView, String> {
        let _guard = self.gate.lock().await;
        let result = self.cycle().await;
        *self.error.lock().unwrap() = result.err();
        self.view()
    }
    async fn cycle(&self) -> Result<(), String> {
        let Some(state) = self.repo.server_client_status().map_err(local_error)? else {
            return Ok(());
        };
        if state.paused {
            return Ok(());
        }
        let session: Session = serde_json::from_str(
            &self
                .secrets
                .get_secret(SESSION_KEY)
                .map_err(local_error)?
                .ok_or("Sign in to your server to resume sync")?,
        )
        .map_err(local_error)?;
        if session.endpoint != state.endpoint {
            return Err("Server session does not match this device's pairing".into());
        }
        // Bound each foreground cycle, preserving queue order across subsequent cycles.
        for _ in 0..100 {
            let Some(queued) = self.repo.next_server_change().map_err(local_error)? else {
                break;
            };
            if queued.conflict.is_some() {
                break;
            }
            let response = self
                .http
                .post(format!("{}/api/v1/server-sync/changes", state.endpoint))
                .header(COOKIE, &session.cookie)
                .json(&queued.request)
                .send()
                .await
                .map_err(network_error)?;
            if response.status() == 409 {
                break;
            } // Download the current server version for review.
            let result: ServerSyncPushResult = json(response).await?;
            match result {
                ServerSyncPushResult::Applied { .. } | ServerSyncPushResult::Duplicate { .. } => {
                    self.repo
                        .acknowledge_server_change(queued.request.event_id)
                        .await
                        .map_err(local_error)?
                }
                ServerSyncPushResult::Conflict { .. } => break,
            }
        }
        let mut cursor = state.cursor;
        for _ in 0..100 {
            let page: ServerSyncPage = json(
                self.http
                    .get(format!("{}/api/v1/server-sync/changes", state.endpoint))
                    .header(COOKIE, &session.cookie)
                    .query(&[
                        ("serverId", state.server_id.clone()),
                        ("cursor", cursor.to_string()),
                        ("limit", "100".into()),
                    ])
                    .send()
                    .await
                    .map_err(network_error)?,
            )
            .await?;
            let next = page.cursor;
            let more = page.has_more;
            if more && next <= cursor {
                return Err("Server sync did not advance its cursor".into());
            }
            self.repo
                .apply_server_page(cursor, page)
                .await
                .map_err(local_error)?;
            cursor = next;
            if !more {
                break;
            }
        }
        Ok(())
    }
    pub async fn accept_server(
        &self,
        event_id: String,
        server_event_id: String,
    ) -> Result<SyncView, String> {
        let _guard = self.gate.lock().await;
        self.repo
            .accept_server_conflict(event_id, server_event_id)
            .await
            .map_err(local_error)?;
        self.view()
    }
}
