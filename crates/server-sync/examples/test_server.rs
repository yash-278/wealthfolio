//! Disposable, loopback-only server for Simulator testing. Synthetic data only.
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use std::{sync::Arc, time::Duration};
use wealthfolio_server::{
    api::app_router,
    auth::{AuthConfig, CookieSecurePolicy},
    build_state,
    config::Config,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    std::env::set_var("WF_DB_PATH", dir.path().join("simulator.db"));
    std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
    let hash = Argon2::default()
        .hash_password(b"simulator-only", &SaltString::generate(&mut OsRng))
        .map_err(|e| format!("Test password setup failed: {e}"))?
        .to_string();
    let config = Config {
        listen_addr: "127.0.0.1:8089".parse()?,
        db_path: dir.path().join("simulator.db").display().to_string(),
        cors_allow: vec![],
        request_timeout: Duration::from_secs(60),
        static_dir: "dist".into(),
        addons_root: dir.path().join("addons").display().to_string(),
        raw_secret_key: vec![7; 32],
        secrets_encryption_key: [7; 32],
        auth: Some(AuthConfig {
            password_hash: Some(hash),
            jwt_secret: vec![8; 32],
            access_token_ttl: Duration::from_secs(8 * 3600),
            cookie_secure: CookieSecurePolicy::Never,
        }),
        oidc: None,
        mcp_enabled: false,
        mcp_audit_enabled: true,
        mcp_allowed_hosts: None,
    };
    let state = build_state(&config).await?;
    state
        .goal_service
        .create_goal(serde_json::from_value(serde_json::json!({
            "goalType":"custom_save_up", "title":"Simulator test goal", "targetAmount":100
        }))?)
        .await?;
    let app = app_router(Arc::clone(&state), &config);
    let listener = tokio::net::TcpListener::bind(config.listen_addr).await?;
    println!("Disposable simulator server: http://127.0.0.1:8089");
    println!("Synthetic test password: simulator-only");
    println!("Stopping this process discards the test database. No production data is used.");
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        let _ = tokio::signal::ctrl_c().await;
    })
    .await?;
    Ok(())
}
