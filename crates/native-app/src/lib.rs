//! In-process native client engine. No HTTP listener or web view is started.
use axum::{
    body::{to_bytes, Body},
    http::Request,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    ffi::{c_char, CStr, CString},
    sync::{Arc, Mutex, OnceLock},
    time::Duration,
};
use tower::ServiceExt;
use wealthfolio_core::{events::DomainEvent, secrets::SecretStore, settings::SettingsServiceTrait};
use wealthfolio_server::{build_state_with_secret_store, config::Config, AppState};
use wealthfolio_server_sync::ServerSyncClient;

struct NativeSecrets;
impl SecretStore for NativeSecrets {
    fn set_secret(&self, service: &str, secret: &str) -> wealthfolio_core::Result<()> {
        keyring::Entry::new(&format!("wealthfolio_native_{service}"), "default")
            .and_then(|entry| entry.set_password(secret))
            .map_err(|e| wealthfolio_core::Error::Secret(e.to_string()))
    }
    fn get_secret(&self, service: &str) -> wealthfolio_core::Result<Option<String>> {
        match keyring::Entry::new(&format!("wealthfolio_native_{service}"), "default")
            .and_then(|entry| entry.get_password())
        {
            Ok(value) => Ok(Some(value)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(wealthfolio_core::Error::Secret(e.to_string())),
        }
    }
    fn delete_secret(&self, service: &str) -> wealthfolio_core::Result<()> {
        match keyring::Entry::new(&format!("wealthfolio_native_{service}"), "default")
            .and_then(|entry| entry.delete_password())
        {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(wealthfolio_core::Error::Secret(e.to_string())),
        }
    }
}

pub struct Engine {
    router: Router,
    state: Arc<AppState>,
    sync: ServerSyncClient,
    events: Mutex<tokio::sync::broadcast::Receiver<wealthfolio_server::events::ServerEvent>>,
}
#[derive(Deserialize)]
pub struct Command {
    pub method: String,
    pub path: String,
    #[serde(default)]
    pub body: Value,
}
#[derive(Serialize, Deserialize)]
pub struct Reply {
    pub status: u16,
    pub body: Value,
}
impl Engine {
    pub async fn open(directory: &str, secrets: Arc<dyn SecretStore>) -> Result<Self, String> {
        std::fs::create_dir_all(directory).map_err(|e| e.to_string())?;
        let config = Config {
            listen_addr: "127.0.0.1:0".parse().unwrap(),
            db_path: format!("{directory}/app.db"),
            cors_allow: vec![],
            request_timeout: Duration::from_secs(120),
            static_dir: String::new(),
            addons_root: format!("{directory}/addons"),
            // Unused with the injected Keychain store and no HTTP authentication/listener.
            raw_secret_key: vec![],
            secrets_encryption_key: [0; 32],
            auth: None,
            oidc: None,
            mcp_enabled: false,
            mcp_audit_enabled: true,
            mcp_allowed_hosts: None,
        };
        let state = build_state_with_secret_store(&config, Some(secrets.clone()))
            .await
            .map_err(|e| e.to_string())?;
        let sync = ServerSyncClient::new(state.app_sync_repository.clone(), secrets)?;
        let router = wealthfolio_server::api::app_router(state.clone(), &config);
        let events = Mutex::new(state.event_bus.subscribe());
        Ok(Self {
            router,
            state,
            sync,
            events,
        })
    }
    pub async fn request(&self, command: Command) -> Result<Reply, String> {
        if command.path == "/native/events" {
            let mut events = self
                .events
                .lock()
                .map_err(|_| "Could not read local events")?;
            let mut names = Vec::new();
            loop {
                match events.try_recv() {
                    Ok(event) => names.push(event.name),
                    Err(tokio::sync::broadcast::error::TryRecvError::Lagged(_)) => {
                        names.push("refresh")
                    }
                    Err(_) => break,
                }
            }
            return Ok(Reply {
                status: 200,
                body: json!(names),
            });
        }
        if command.path.starts_with("/native/sync") {
            let before = self.sync.view()?.connection.map(|c| c.cursor);
            let view = match command.path.as_str() {
                "/native/sync/status" => self.sync.view()?,
                "/native/sync/connect" => {
                    self.sync
                        .connect(
                            string(&command.body, "endpoint")?,
                            string(&command.body, "password")?,
                        )
                        .await?
                }
                "/native/sync/run" => self.sync.sync().await?,
                "/native/sync/pause" => {
                    self.sync
                        .pause(
                            command.body["paused"]
                                .as_bool()
                                .ok_or("Missing paused state")?,
                        )
                        .await?
                }
                "/native/sync/accept-server" => {
                    self.sync
                        .accept_server(
                            string(&command.body, "eventId")?,
                            string(&command.body, "serverEventId")?,
                        )
                        .await?
                }
                _ => return Err("Unknown sync command".into()),
            };
            if before != view.connection.as_ref().map(|c| c.cursor)
                || command.path == "/native/sync/accept-server"
            {
                let settings = self
                    .state
                    .settings_service
                    .get_settings()
                    .map_err(|e| e.to_string())?;
                *self
                    .state
                    .base_currency
                    .write()
                    .map_err(|_| "Could not update base currency")? = settings.base_currency;
                *self
                    .state
                    .timezone
                    .write()
                    .map_err(|_| "Could not update time zone")? = settings.timezone;
                self.state
                    .domain_event_sink
                    .emit(DomainEvent::device_sync_pull_complete());
            }
            return Ok(Reply {
                status: 200,
                body: serde_json::to_value(view).map_err(|e| e.to_string())?,
            });
        }
        if !command.path.starts_with("/api/v1/") {
            return Err("Invalid local API path".into());
        }
        let body = if command.body.is_null() {
            String::new()
        } else {
            command.body.to_string()
        };
        let request = Request::builder()
            .method(command.method.as_str())
            .uri(&command.path)
            .header("content-type", "application/json")
            .body(Body::from(body))
            .map_err(|e| e.to_string())?;
        let response = self
            .router
            .clone()
            .oneshot(request)
            .await
            .map_err(|e| e.to_string())?;
        let status = response.status().as_u16();
        let bytes = to_bytes(response.into_body(), 32 * 1024 * 1024)
            .await
            .map_err(|e| e.to_string())?;
        let body = if bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&bytes)
                .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&bytes).into_owned()))
        };
        Ok(Reply { status, body })
    }
}
fn string(body: &Value, key: &str) -> Result<String, String> {
    body[key]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("Missing {key}"))
}
struct RuntimeEngine {
    runtime: tokio::runtime::Runtime,
    engine: Engine,
}
static ENGINE: OnceLock<RuntimeEngine> = OnceLock::new();

// Swift owns argument storage for the call; Rust owns replies until wf_native_free.
fn ffi_reply(work: impl FnOnce() -> Result<Value, String>) -> *mut c_char {
    let reply = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(work)) {
        Ok(Ok(value)) => value,
        Ok(Err(error)) => json!({"status":500,"body":{"message":error}}),
        Err(_) => {
            json!({"status":500,"body":{"message":"The local engine could not complete the request"}})
        }
    };
    CString::new(reply.to_string()).unwrap().into_raw()
}
unsafe fn argument(pointer: *const c_char) -> Result<String, String> {
    if pointer.is_null() {
        return Err("Missing argument".into());
    }
    // SAFETY: caller supplies a live NUL-terminated UTF-8 string for the call.
    unsafe { CStr::from_ptr(pointer) }
        .to_str()
        .map(str::to_owned)
        .map_err(|e| e.to_string())
}
/// # Safety
/// directory must point to a live NUL-terminated UTF-8 string.
#[no_mangle]
pub unsafe extern "C" fn wf_native_open(directory: *const c_char) -> *mut c_char {
    ffi_reply(|| {
        let directory = unsafe { argument(directory) }?;
        if ENGINE.get().is_some() {
            return Err("Local engine is already open".into());
        }
        let runtime = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
        let engine = runtime.block_on(Engine::open(&directory, Arc::new(NativeSecrets)))?;
        ENGINE
            .set(RuntimeEngine { runtime, engine })
            .map_err(|_| "Local engine is already open")?;
        Ok(json!({"status":200,"body":null}))
    })
}
/// # Safety
/// command must point to a live NUL-terminated UTF-8 JSON string.
#[no_mangle]
pub unsafe extern "C" fn wf_native_request(command: *const c_char) -> *mut c_char {
    ffi_reply(|| {
        let command = unsafe { argument(command) }?;
        let command = serde_json::from_str(&command).map_err(|e| e.to_string())?;
        let engine = ENGINE.get().ok_or("Local engine is not open")?;
        serde_json::to_value(engine.runtime.block_on(engine.engine.request(command))?)
            .map_err(|e| e.to_string())
    })
}
/// # Safety
/// pointer must be an unfreed reply returned by this library, or null.
#[no_mangle]
pub unsafe extern "C" fn wf_native_free(pointer: *mut c_char) {
    if !pointer.is_null() {
        drop(unsafe { CString::from_raw(pointer) });
    }
}

/// Streams the existing Assistant NDJSON response to Swift without buffering it.
/// # Safety
/// command is a live UTF-8 C string; callback and context remain valid until return.
#[no_mangle]
pub unsafe extern "C" fn wf_native_stream(
    command: *const c_char,
    callback: extern "C" fn(*const c_char, *mut std::ffi::c_void) -> i32,
    context: *mut std::ffi::c_void,
) -> *mut c_char {
    use http_body_util::BodyExt;
    ffi_reply(|| {
        let command: Command =
            serde_json::from_str(&unsafe { argument(command) }?).map_err(|e| e.to_string())?;
        if command.path != "/api/v1/ai/chat/stream" {
            return Err("Unknown native stream".into());
        }
        let engine = ENGINE.get().ok_or("Local engine is not open")?;
        engine.runtime.block_on(async {
            let request = Request::builder()
                .method("POST")
                .uri(command.path)
                .header("content-type", "application/json")
                .body(Body::from(command.body.to_string()))
                .map_err(|e| e.to_string())?;
            let response = engine
                .engine
                .router
                .clone()
                .oneshot(request)
                .await
                .map_err(|e| e.to_string())?;
            if !response.status().is_success() {
                let status = response.status().as_u16();
                let bytes = to_bytes(response.into_body(), 1_000_000)
                    .await
                    .map_err(|e| e.to_string())?;
                let body: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
                return Ok(json!({"status":status,"body":body}));
            }
            let mut body = response.into_body();
            let mut pending = Vec::new();
            while let Some(frame) = body.frame().await {
                let frame = frame.map_err(|e| e.to_string())?;
                if let Ok(data) = frame.into_data() {
                    pending.extend_from_slice(&data);
                    if pending.len() > 8_000_000 {
                        return Err("Assistant event exceeds the supported size".into());
                    }
                    while let Some(end) = pending.iter().position(|b| *b == b'\n') {
                        let bytes: Vec<_> = pending.drain(..=end).collect();
                        let event = CString::new(bytes).map_err(|_| "Invalid Assistant event")?;
                        if callback(event.as_ptr(), context) == 0 {
                            return Ok(json!({"status":200,"body":null}));
                        }
                    }
                }
            }
            Ok(json!({"status":200,"body":null}))
        })
    })
}
