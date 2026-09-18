use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use wealthfolio_core::{
    accounts::AccountServiceTrait,
    activities::ActivityRepositoryTrait,
    goals::{GoalRepositoryTrait, NewGoal},
    secrets::SecretStore,
};
use wealthfolio_server::{
    api::app_router,
    auth::{AuthConfig, CookieSecurePolicy},
    build_state,
    config::Config,
};
use wealthfolio_server_sync::{validate_endpoint, ServerSyncClient};
use wealthfolio_storage_sqlite::{
    db::{create_pool, run_migrations, write_actor::spawn_writer},
    goals::GoalRepository,
    sync::app_sync::AppSyncRepository,
};

#[derive(Default)]
struct Secrets(Mutex<HashMap<String, String>>);
impl SecretStore for Secrets {
    fn set_secret(&self, key: &str, value: &str) -> wealthfolio_core::Result<()> {
        self.0.lock().unwrap().insert(key.into(), value.into());
        Ok(())
    }
    fn get_secret(&self, key: &str) -> wealthfolio_core::Result<Option<String>> {
        Ok(self.0.lock().unwrap().get(key).cloned())
    }
    fn delete_secret(&self, key: &str) -> wealthfolio_core::Result<()> {
        self.0.lock().unwrap().remove(key);
        Ok(())
    }
}
struct Device {
    _dir: tempfile::TempDir,
    repo: Arc<AppSyncRepository>,
    goals: GoalRepository,
    activities: wealthfolio_storage_sqlite::activities::ActivityRepository,
    secrets: Arc<Secrets>,
}
impl Device {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("device.db");
        run_migrations(path.to_str().unwrap()).unwrap();
        let pool = create_pool(path.to_str().unwrap()).unwrap();
        let writer = spawn_writer(pool.as_ref().clone()).unwrap();
        Self {
            _dir: dir,
            repo: Arc::new(AppSyncRepository::new(pool.clone(), writer.clone())),
            goals: GoalRepository::new(pool.clone(), writer.clone()),
            activities: wealthfolio_storage_sqlite::activities::ActivityRepository::new(
                pool, writer,
            ),
            secrets: Arc::new(Secrets::default()),
        }
    }
    fn client(&self) -> ServerSyncClient {
        ServerSyncClient::new(self.repo.clone(), self.secrets.clone()).unwrap()
    }
    async fn edit(&self, id: &str, title: &str) {
        let mut goal = self.goals.load_goal(id).unwrap();
        goal.title = title.into();
        self.goals.update_goal(goal).await.unwrap();
    }
}
fn goal(id: &str, title: &str) -> NewGoal {
    serde_json::from_value(
        serde_json::json!({"id":id,"goalType":"custom_save_up","title":title,"targetAmount":100}),
    )
    .unwrap()
}
#[test]
fn endpoints_reject_credential_leaks_and_remote_cleartext() {
    for url in [
        "http://example.com",
        "https://owner:password@example.com",
        "https://example.com/api",
        "https://example.com?token=bad",
        "file:///tmp/app",
    ] {
        assert!(validate_endpoint(url).is_err());
    }
    assert_eq!(
        validate_endpoint("https://example.com/").unwrap(),
        "https://example.com"
    );
}
#[tokio::test]
async fn two_devices_sync_offline_edits_conflicts_pause_and_restart() {
    let server_dir = tempfile::tempdir().unwrap();
    std::env::set_var("WF_DB_PATH", server_dir.path().join("server.db"));
    std::env::set_var("WF_SECRET_FILE", server_dir.path().join("secrets.json"));
    let hash = Argon2::default()
        .hash_password(b"test-password", &SaltString::generate(&mut OsRng))
        .unwrap()
        .to_string();
    let config = Config {
        listen_addr: "127.0.0.1:0".parse().unwrap(),
        db_path: server_dir.path().join("server.db").display().to_string(),
        cors_allow: vec![],
        request_timeout: Duration::from_secs(60),
        static_dir: "dist".into(),
        addons_root: server_dir.path().join("addons").display().to_string(),
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
    let goal_id = state
        .goal_service
        .create_goal(goal("test-goal", "Initial server goal"))
        .await
        .unwrap()
        .id;
    let cash_account = state.account_service.create_account(serde_json::from_value(
        serde_json::json!({"name":"Synthetic cash", "accountType":"CASH", "currency":"USD", "isDefault":false, "isActive":true})
    ).unwrap()).await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = app_router(state.clone(), &config);
    let server = tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await
        .unwrap()
    });
    let url = format!("http://{address}");
    let a = Device::new();
    let b = Device::new();
    let ca = a.client();
    let cb = b.client();
    ca.connect(url.clone(), "test-password".into())
        .await
        .unwrap();
    cb.connect(url.clone(), "test-password".into())
        .await
        .unwrap();
    assert_eq!(
        a.goals.load_goal(&goal_id).unwrap().title,
        "Initial server goal"
    );
    a.edit(&goal_id, "Phone A edit").await;
    assert_eq!(ca.view().unwrap().connection.unwrap().pending, 1);
    assert!(ca.sync().await.unwrap().error.is_none());
    assert!(cb.sync().await.unwrap().error.is_none());
    assert_eq!(b.goals.load_goal(&goal_id).unwrap().title, "Phone A edit");
    assert_eq!(
        state.goal_service.get_goal(&goal_id).unwrap().title,
        "Phone A edit"
    );
    // Quick Add posts through this same activity service with its own source marker.
    // A phone paired before the web write must receive it incrementally.
    let quick_add = state.activity_service.create_activity(serde_json::from_value(
        serde_json::json!({"accountId":cash_account.id, "activityType":"WITHDRAWAL",
            "activityDate":"2026-09-13T12:00:00Z", "amount":"25", "currency":"USD",
            "sourceSystem":"QUICK_ADD", "sourceRecordId":"synthetic-reference", "needsReview":false})
    ).unwrap()).await.unwrap();
    assert!(cb.sync().await.unwrap().error.is_none());
    assert!(
        b.activities.get_activity(&quick_add.id).is_ok(),
        "Quick Add web transaction must arrive on an already-paired phone"
    );
    // A newly paired device must also include the same transaction in its snapshot.
    let fresh = Device::new();
    fresh
        .client()
        .connect(url.clone(), "test-password".into())
        .await
        .unwrap();
    assert!(
        fresh.activities.get_activity(&quick_add.id).is_ok(),
        "Quick Add transaction must be included in initial download"
    );
    state
        .activity_service
        .update_activity(
            serde_json::from_value(serde_json::json!({
                "id":quick_add.id, "accountId":cash_account.id, "activityType":"WITHDRAWAL",
                "activityDate":"2026-09-13T12:00:00Z", "amount":"30", "currency":"USD"
            }))
            .unwrap(),
        )
        .await
        .unwrap();
    assert!(cb.sync().await.unwrap().error.is_none());
    assert_eq!(
        b.activities
            .get_activity(&quick_add.id)
            .unwrap()
            .amount
            .unwrap()
            .to_string(),
        "30"
    );
    state
        .activity_service
        .delete_activity(quick_add.id.clone())
        .await
        .unwrap();
    assert!(cb.sync().await.unwrap().error.is_none());
    assert!(b.activities.get_activity(&quick_add.id).is_err());
    // Two offline edits based on the same server version must not silently overwrite one another.
    a.edit(&goal_id, "Winning server edit").await;
    b.edit(&goal_id, "Preserved offline edit").await;
    assert!(ca.sync().await.unwrap().error.is_none());
    let conflict = cb.sync().await.unwrap();
    assert!(conflict.error.is_none(), "{:?}", conflict.error);
    let conflict = conflict.conflict.expect("conflict should be visible");
    assert_eq!(
        b.goals.load_goal(&goal_id).unwrap().title,
        "Preserved offline edit"
    );
    cb.accept_server(conflict.event_id, conflict.server_event_id)
        .await
        .unwrap();
    assert_eq!(
        b.goals.load_goal(&goal_id).unwrap().title,
        "Winning server edit"
    );
    // Pausing still journals edits, and a new client instance resumes the same durable queue.
    cb.pause(true).await.unwrap();
    b.edit(&goal_id, "Paused edit").await;
    cb.sync().await.unwrap();
    assert_eq!(
        state.goal_service.get_goal(&goal_id).unwrap().title,
        "Winning server edit"
    );
    let restarted = b.client();
    assert_eq!(restarted.view().unwrap().connection.unwrap().pending, 1);
    restarted.pause(false).await.unwrap();
    assert!(restarted.sync().await.unwrap().error.is_none());
    assert_eq!(
        state.goal_service.get_goal(&goal_id).unwrap().title,
        "Paused edit"
    );
    // Several edits to a record retain the revision chain when uploaded in order.
    b.edit(&goal_id, "Queued edit one").await;
    b.edit(&goal_id, "Queued edit two").await;
    assert!(restarted.sync().await.unwrap().error.is_none());
    assert_eq!(
        state.goal_service.get_goal(&goal_id).unwrap().title,
        "Queued edit two"
    );
    // A web-side edit comes back through the same feed.
    let mut web = state.goal_service.get_goal(&goal_id).unwrap();
    web.title = "Web edit".into();
    state.goal_service.update_goal(web).await.unwrap();
    assert!(restarted.sync().await.unwrap().error.is_none());
    assert_eq!(b.goals.load_goal(&goal_id).unwrap().title, "Web edit");
    // First pairing refuses to destroy existing local records.
    let occupied = Device::new();
    let local_id = occupied
        .goals
        .insert_new_goal(goal("local", "Keep me"))
        .await
        .unwrap()
        .id;
    assert!(occupied
        .client()
        .connect(url, "test-password".into())
        .await
        .is_err());
    assert_eq!(
        occupied.goals.load_goal(&local_id).unwrap().title,
        "Keep me"
    );
    // Simulate termination after the server accepted an upload but before local acknowledgement.
    b.edit(&goal_id, "Accepted before restart").await;
    let queued = b.repo.next_server_change().unwrap().unwrap();
    state
        .app_sync_repository
        .push_server_sync(queued.request)
        .await
        .unwrap();
    let recovered = b.client();
    assert!(recovered.sync().await.unwrap().error.is_none());
    assert_eq!(recovered.view().unwrap().connection.unwrap().pending, 0);
    assert_eq!(
        state.goal_service.get_goal(&goal_id).unwrap().title,
        "Accepted before restart"
    );

    // Losing connectivity does not lose edits or regenerate their identities.
    server.abort();
    b.edit(&goal_id, "Offline after disconnect").await;
    let before = b
        .repo
        .next_server_change()
        .unwrap()
        .unwrap()
        .request
        .event_id;
    assert!(restarted.sync().await.unwrap().error.is_some());
    assert_eq!(
        b.repo
            .next_server_change()
            .unwrap()
            .unwrap()
            .request
            .event_id,
        before
    );
    assert!(b
        .secrets
        .0
        .lock()
        .unwrap()
        .values()
        .all(|s| !s.contains("test-password")));
}
