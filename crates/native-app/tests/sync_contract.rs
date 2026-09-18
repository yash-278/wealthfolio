use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use wealthfolio_core::{
    accounts::AccountServiceTrait, secrets::SecretStore, settings::SettingsServiceTrait,
};
use wealthfolio_native::{Command, Engine};
use wealthfolio_server::{
    api::app_router,
    auth::{AuthConfig, CookieSecurePolicy},
    build_state_with_secret_store,
    config::Config,
};
#[derive(Default)]
struct Secrets(Mutex<HashMap<String, String>>);
impl SecretStore for Secrets {
    fn set_secret(&self, k: &str, v: &str) -> wealthfolio_core::Result<()> {
        self.0.lock().unwrap().insert(k.into(), v.into());
        Ok(())
    }
    fn get_secret(&self, k: &str) -> wealthfolio_core::Result<Option<String>> {
        Ok(self.0.lock().unwrap().get(k).cloned())
    }
    fn delete_secret(&self, k: &str) -> wealthfolio_core::Result<()> {
        self.0.lock().unwrap().remove(k);
        Ok(())
    }
}
async fn call(engine: &Engine, path: &str, body: Value) -> Value {
    let reply = engine
        .request(Command {
            method: "POST".into(),
            path: path.into(),
            body,
        })
        .await
        .unwrap();
    assert_eq!(reply.status, 200, "{}", reply.body);
    reply.body
}
#[tokio::test]
async fn native_app_downloads_web_edits_and_uploads_queued_offline_edits() {
    let server_dir = tempfile::tempdir().unwrap();
    let hash = Argon2::default()
        .hash_password(b"synthetic-password", &SaltString::generate(&mut OsRng))
        .unwrap()
        .to_string();
    let config = Config {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        db_path: server_dir.path().join("app.db").display().to_string(),
        cors_allow: vec![],
        request_timeout: Duration::from_secs(30),
        static_dir: String::new(),
        addons_root: server_dir.path().join("addons").display().to_string(),
        raw_secret_key: vec![],
        secrets_encryption_key: [0; 32],
        auth: Some(AuthConfig {
            password_hash: Some(hash),
            jwt_secret: vec![9; 32],
            access_token_ttl: Duration::from_secs(3600),
            cookie_secure: CookieSecurePolicy::Never,
        }),
        oidc: None,
        mcp_enabled: false,
        mcp_audit_enabled: false,
        mcp_allowed_hosts: None,
    };
    let state = build_state_with_secret_store(&config, Some(Arc::new(Secrets::default())))
        .await
        .unwrap();
    state
        .settings_service
        .update_settings(
            &serde_json::from_value(json!({"baseCurrency":"EUR","timezone":"Europe/Paris"}))
                .unwrap(),
        )
        .await
        .unwrap();
    let account=state.account_service.create_account(serde_json::from_value(json!({"name":"Web account","accountType":"CASH","currency":"EUR","isDefault":true,"isActive":true,"trackingMode":"TRANSACTIONS"})).unwrap()).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("http://{}", listener.local_addr().unwrap());
    let router = app_router(state.clone(), &config);
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap()
    });
    let device_dir = tempfile::tempdir().unwrap();
    let engine = Engine::open(
        device_dir.path().to_str().unwrap(),
        Arc::new(Secrets::default()),
    )
    .await
    .unwrap();
    let configured = engine.request(Command { method: "PUT".into(), path: "/api/v1/settings".into(), body: json!({"baseCurrency":"EUR","timezone":"Europe/Paris","onboardingCompleted":true}) }).await.unwrap();
    assert_eq!(configured.status, 200);
    call(
        &engine,
        "/native/sync/connect",
        json!({"endpoint":url,"password":"synthetic-password"}),
    )
    .await;
    let valuation = call(
        &engine,
        "/api/v1/valuations/current/query",
        json!({"filter":{"type":"all"}}),
    )
    .await;
    assert_eq!(valuation["summary"]["baseCurrency"], "EUR");
    call(&engine, "/native/sync/pause", json!({"paused":true})).await;
    let local=call(&engine,"/api/v1/activities",json!({"accountId":account.id,"activityType":"DEPOSIT","activityDate":"2026-01-15T12:00:00Z","currency":"EUR","amount":"45.67","sourceSystem":"MANUAL"})).await;
    let status = call(&engine, "/native/sync/status", Value::Null).await;
    assert!(status["connection"]["pending"].as_i64().unwrap() > 0);
    call(&engine, "/native/sync/pause", json!({"paused":false})).await;
    call(&engine, "/native/sync/run", Value::Null).await;
    let web=state.activity_service.create_activity(serde_json::from_value(json!({"accountId":account.id,"activityType":"WITHDRAWAL","activityDate":"2026-01-16T12:00:00Z","currency":"EUR","amount":"5.67","sourceSystem":"QUICK_ADD"})).unwrap()).await.unwrap();
    call(&engine, "/native/sync/run", Value::Null).await;
    let rows = call(
        &engine,
        "/api/v1/activities/search",
        json!({"page":0,"pageSize":50}),
    )
    .await;
    let ids: Vec<_> = rows["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&local["id"].as_str().unwrap()));
    assert!(ids.contains(&web.id.as_str()));
    let web_rows = state
        .activity_service
        .search_activities_in_utc_range(0, 50, None, None, None, None, None, None, None, None, None)
        .unwrap();
    assert!(web_rows
        .data
        .iter()
        .any(|r| r.id == local["id"].as_str().unwrap()));
    server.abort();
}
