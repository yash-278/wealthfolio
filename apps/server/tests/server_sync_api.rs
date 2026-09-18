use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use axum::{
    body::{to_bytes, Body},
    extract::ConnectInfo,
    http::Request,
};
use serde_json::{json, Value};
use std::{net::SocketAddr, time::Duration};
use tower::ServiceExt;
use wealthfolio_server::{
    api::app_router,
    auth::{AuthConfig, CookieSecurePolicy},
    build_state,
    config::Config,
};

async fn request(
    app: axum::Router,
    method: &str,
    path: &str,
    cookie: &str,
    body: Value,
) -> axum::response::Response {
    let mut request = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json")
        .header("cookie", cookie)
        .body(Body::from(body.to_string()))
        .unwrap();
    request
        .extensions_mut()
        .insert(ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345))));
    app.oneshot(request).await.unwrap()
}
async fn json_body(response: axum::response::Response) -> Value {
    serde_json::from_slice(&to_bytes(response.into_body(), 10_000_000).await.unwrap()).unwrap()
}

#[tokio::test]
async fn owner_server_sync_round_trip_and_auth_boundaries() {
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("WF_DB_PATH", dir.path().join("app.db"));
    std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
    let hash = Argon2::default()
        .hash_password(b"test-password", &SaltString::generate(&mut OsRng))
        .unwrap()
        .to_string();
    let mut config = Config {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        db_path: dir.path().join("app.db").display().to_string(),
        cors_allow: vec!["http://localhost".into()],
        request_timeout: Duration::from_secs(30),
        static_dir: "dist".into(),
        addons_root: dir.path().join("addons").display().to_string(),
        raw_secret_key: vec![7; 32],
        secrets_encryption_key: [7; 32],
        auth: Some(AuthConfig {
            password_hash: Some(hash),
            jwt_secret: vec![8; 32],
            access_token_ttl: Duration::from_secs(3600),
            cookie_secure: CookieSecurePolicy::Never,
        }),
        oidc: None,
        mcp_enabled: false,
        mcp_audit_enabled: true,
        mcp_allowed_hosts: None,
    };
    let state = build_state(&config).await.unwrap();
    let app = app_router(state.clone(), &config);
    for (method, path) in [
        ("GET", "/server-sync"),
        ("POST", "/server-sync"),
        ("GET", "/server-sync/snapshot"),
        ("GET", "/server-sync/changes"),
        ("POST", "/server-sync/changes"),
    ] {
        assert_eq!(
            request(
                app.clone(),
                method,
                &format!("/api/v1{path}"),
                "",
                json!({})
            )
            .await
            .status(),
            401
        );
    }
    let login = request(
        app.clone(),
        "POST",
        "/api/v1/auth/login",
        "",
        json!({"password":"test-password"}),
    )
    .await;
    assert_eq!(login.status(), 200);
    let cookie = login.headers()["set-cookie"]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_string();
    let enable = request(
        app.clone(),
        "POST",
        "/api/v1/server-sync",
        &cookie,
        json!({}),
    )
    .await;
    assert_eq!(enable.status(), 200);
    let head = json_body(enable).await;
    #[cfg(feature = "device-sync")]
    assert_eq!(
        request(
            app.clone(),
            "GET",
            "/api/v1/sync/device/current",
            &cookie,
            Value::Null
        )
        .await
        .status(),
        403
    );

    let event_id = uuid::Uuid::now_v7().to_string();
    let mutation = json!({"serverId":head["serverId"],"eventId":event_id,"entity":"goal","entityId":"goal-api",
        "op":"create","baseEventId":null,"payload":{"id":"goal-api","title":"API test","target_amount":100}});
    let response = request(
        app.clone(),
        "POST",
        "/api/v1/server-sync/changes",
        &cookie,
        mutation.clone(),
    )
    .await;
    assert_eq!(response.status(), 200);
    assert_eq!(json_body(response).await["status"], "applied");
    let response = request(
        app.clone(),
        "POST",
        "/api/v1/server-sync/changes",
        &cookie,
        mutation.clone(),
    )
    .await;
    assert_eq!(json_body(response).await["status"], "duplicate");
    let mut stale = mutation.clone();
    stale["eventId"] = json!(uuid::Uuid::now_v7().to_string());
    stale["op"] = json!("update");
    let conflict = request(
        app.clone(),
        "POST",
        "/api/v1/server-sync/changes",
        &cookie,
        stale,
    )
    .await;
    assert_eq!(conflict.status(), 409);
    assert_eq!(json_body(conflict).await["currentEventId"], event_id);
    let page = request(
        app.clone(),
        "GET",
        &format!(
            "/api/v1/server-sync/changes?serverId={}&cursor=0",
            head["serverId"].as_str().unwrap()
        ),
        &cookie,
        Value::Null,
    )
    .await;
    assert_eq!(page.status(), 200);
    assert_eq!(page.headers()["cache-control"], "no-store");
    assert_eq!(
        json_body(page).await["changes"].as_array().unwrap().len(),
        1
    );
    let snapshot = request(
        app.clone(),
        "GET",
        "/api/v1/server-sync/snapshot",
        &cookie,
        Value::Null,
    )
    .await;
    assert_eq!(snapshot.status(), 200);
    assert_eq!(snapshot.headers()["x-sync-cursor"], "1");
    let bytes = to_bytes(snapshot.into_body(), 10_000_000).await.unwrap();
    assert!(bytes.starts_with(b"SQLite format 3"));
    let denied = request(
        app.clone(),
        "POST",
        "/api/v1/server-sync/changes",
        &cookie,
        json!({"extra":"x".repeat(1_000_001)}),
    )
    .await;
    assert_eq!(denied.status(), 413);

    // The normal web API sees the uploaded record, and its edits enter the same feed.
    let goal_response = request(
        app.clone(),
        "GET",
        "/api/v1/goals/goal-api",
        &cookie,
        Value::Null,
    )
    .await;
    assert_eq!(goal_response.status(), 200);
    let mut goal = json_body(goal_response).await;
    assert_eq!(goal["title"], "API test");
    goal["title"] = json!("Edited on the web");
    let web_edit = request(app.clone(), "PUT", "/api/v1/goals", &cookie, goal).await;
    assert_eq!(web_edit.status(), 200);
    let updates = request(
        app.clone(),
        "GET",
        &format!(
            "/api/v1/server-sync/changes?serverId={}&cursor=1",
            head["serverId"].as_str().unwrap()
        ),
        &cookie,
        Value::Null,
    )
    .await;
    let updates = json_body(updates).await;
    assert_eq!(
        updates["changes"][0]["payload"]["title"],
        "Edited on the web"
    );
    // An older successful upload remains a duplicate after the web edit.
    let retry = request(
        app.clone(),
        "POST",
        "/api/v1/server-sync/changes",
        &cookie,
        mutation,
    )
    .await;
    assert_eq!(json_body(retry).await["status"], "duplicate");

    // Opt-in is durable across a new service instance.
    let reopened = build_state(&config).await.unwrap();
    assert_eq!(
        reopened
            .app_sync_repository
            .server_sync_head()
            .unwrap()
            .server_id,
        head["serverId"]
    );
    assert_eq!(
        reopened
            .app_sync_repository
            .server_sync_head()
            .unwrap()
            .cursor,
        2
    );
    // A server without owner authentication may not expose the sync API.
    config.auth = None;
    let open_state = build_state(&config).await.unwrap();
    let open_app = app_router(open_state, &config);
    assert_eq!(
        request(open_app, "POST", "/api/v1/server-sync", "", json!({}))
            .await
            .status(),
        403
    );
}
