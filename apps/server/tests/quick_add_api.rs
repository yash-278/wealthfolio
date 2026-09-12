use std::{net::SocketAddr, time::Duration};

use axum::{
    body::{to_bytes, Body},
    http::Request,
};
use tempfile::tempdir;
use tower::ServiceExt;
use wealthfolio_server::{api::app_router, build_state, config::Config};

static ENV_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

fn test_config(db_path: String, addons_root: String) -> Config {
    Config {
        listen_addr: "127.0.0.1:0".parse::<SocketAddr>().unwrap(),
        db_path,
        cors_allow: vec!["*".to_string()],
        request_timeout: Duration::from_secs(30),
        static_dir: "dist".to_string(),
        addons_root,
        raw_secret_key: vec![7; 32],
        secrets_encryption_key: [7; 32],
        auth: None,
        oidc: None,
        mcp_enabled: false,
        mcp_audit_enabled: true,
        mcp_allowed_hosts: None,
    }
}

async fn capture(app: axum::Router, text: &str) -> (axum::http::StatusCode, serde_json::Value) {
    let response = app.oneshot(Request::builder().method("POST")
        .uri("/api/v1/captures").header("content-type", "application/json")
        .body(Body::from(serde_json::json!({"clientRequestId":"request-1","text":text,"inputKind":"bank_alert"}).to_string())).unwrap()).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null),
    )
}

#[tokio::test]
async fn capture_is_durable_and_request_identity_is_immutable() {
    let _guard = ENV_LOCK.lock().await;
    let dir = tempdir().unwrap();
    std::env::set_var("WF_DB_PATH", dir.path().join("app.db"));
    std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
    let config = test_config(
        dir.path().join("app.db").display().to_string(),
        dir.path().join("addons").display().to_string(),
    );
    let state = build_state(&config).await.unwrap();
    let app = app_router(state, &config);
    let (first, again) = tokio::join!(
        capture(app.clone(), "A bank alert"),
        capture(app.clone(), "A bank alert")
    );
    assert!(first.0.is_success(), "capture rejected: {:?}", first);
    assert_eq!(first.1["id"], again.1["id"]);
    assert_eq!(first.1["status"], "needs_review");
    assert_eq!(
        capture(app.clone(), "Different payload").await.0,
        axum::http::StatusCode::CONFLICT
    );
    let state = build_state(&config).await.unwrap();
    let restored = capture(app_router(state, &config), "A bank alert").await;
    assert_eq!(restored.1["id"], first.1["id"]);
}

async fn request(
    app: axum::Router,
    method: &str,
    path: &str,
    body: serde_json::Value,
) -> (axum::http::StatusCode, serde_json::Value) {
    let response = app
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null),
    )
}

#[tokio::test]
async fn review_is_visible_and_stale_dismissal_cannot_overwrite_a_resolution() {
    let _guard = ENV_LOCK.lock().await;
    let dir = tempdir().unwrap();
    std::env::set_var("WF_DB_PATH", dir.path().join("app.db"));
    std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
    let config = test_config(
        dir.path().join("app.db").display().to_string(),
        dir.path().join("addons").display().to_string(),
    );
    let app = app_router(build_state(&config).await.unwrap(), &config);
    let saved = capture(app.clone(), "Unclear message").await.1;
    let reviews = request(
        app.clone(),
        "GET",
        "/api/v1/capture-reviews",
        serde_json::Value::Null,
    )
    .await;
    assert!(reviews.0.is_success());
    assert_eq!(reviews.1[0]["input"]["text"], "Unclear message");
    let path = format!(
        "/api/v1/capture-reviews/{}/dismiss",
        saved["reviews"][0]["id"].as_str().unwrap()
    );
    let dismissed = request(app.clone(), "POST", &path, serde_json::json!({"version":1})).await;
    assert!(dismissed.0.is_success(), "{:?}", dismissed);
    assert_eq!(dismissed.1["reviews"][0]["status"], "dismissed");
    assert!(
        request(app.clone(), "POST", &path, serde_json::json!({"version":1}))
            .await
            .0
            .is_success()
    );
    assert_eq!(
        request(app.clone(), "POST", &path, serde_json::json!({"version":2}))
            .await
            .0,
        axum::http::StatusCode::CONFLICT
    );
    assert_eq!(
        request(
            app,
            "GET",
            "/api/v1/capture-reviews",
            serde_json::Value::Null
        )
        .await
        .1,
        serde_json::json!([])
    );
}

#[tokio::test]
async fn corrected_review_posts_once_and_retains_category_followup() {
    let _guard = ENV_LOCK.lock().await;
    let dir = tempdir().unwrap();
    std::env::set_var("WF_DB_PATH", dir.path().join("app.db"));
    std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
    let config = test_config(
        dir.path().join("app.db").display().to_string(),
        dir.path().join("addons").display().to_string(),
    );
    let app = app_router(build_state(&config).await.unwrap(), &config);
    let account = request(app.clone(), "POST", "/api/v1/accounts", serde_json::json!({"name":"Test bank","accountType":"CASH","currency":"INR","isDefault":false,"isActive":true,"trackingMode":"TRANSACTIONS"})).await;
    assert!(account.0.is_success(), "{:?}", account);
    let saved = capture(app.clone(), "Paid 120 INR on 2026-09-10").await.1;
    let path = format!(
        "/api/v1/capture-reviews/{}/resolve",
        saved["reviews"][0]["id"].as_str().unwrap()
    );
    let body = serde_json::json!({"version":1,"fields":{"accountId":account.1["id"],"amount":"120","currency":"INR","date":"2026-09-10","direction":"debit","merchant":"Test cafe","reference":null,"kind":"payment"}});
    let result = request(app.clone(), "POST", &path, body.clone()).await;
    assert!(result.0.is_success(), "{:?}", result);
    assert_eq!(result.1["candidates"][0]["status"], "posted");
    assert_eq!(result.1["reviews"][1]["reason"], "category_required");
    assert!(request(app.clone(), "POST", &path, body)
        .await
        .0
        .is_success());
    assert!(request(
        app.clone(),
        "PUT",
        "/api/v1/spending/settings",
        serde_json::json!({"enabled":true,"accountIds":[account.1["id"]]})
    )
    .await
    .0
    .is_success());
    let category_path = format!(
        "/api/v1/capture-reviews/{}/resolve",
        result.1["reviews"][1]["id"].as_str().unwrap()
    );
    let categorized = request(app.clone(), "POST", &category_path, serde_json::json!({"version":result.1["version"],"action":"categorize","taxonomyId":"spending_categories","categoryId":"cat_food_coffee"})).await;
    assert!(categorized.0.is_success(), "{:?}", categorized);
    let activity_id = result.1["candidates"][0]["activityId"].as_str().unwrap();
    let assignments = request(
        app.clone(),
        "GET",
        &format!("/api/v1/spending/activities/{activity_id}/assignments"),
        serde_json::Value::Null,
    )
    .await;
    assert_eq!(assignments.1[0]["categoryId"], "cat_food_coffee");
    let activities = request(
        app,
        "POST",
        "/api/v1/activities/search",
        serde_json::json!({"page":0,"pageSize":20}),
    )
    .await;
    assert!(activities.0.is_success(), "{:?}", activities);
    assert_eq!(activities.1["data"].as_array().unwrap().len(), 1);
    assert_eq!(activities.1["data"][0]["accountId"], account.1["id"]);
    assert_eq!(activities.1["data"][0]["activityType"], "WITHDRAWAL");
    assert_eq!(activities.1["data"][0]["needsReview"], false);
}

#[tokio::test]
async fn configured_capture_extracts_with_a_tool_free_schema_and_waits_for_supervised_review() {
    let _guard = ENV_LOCK.lock().await;
    let dir = tempdir().unwrap();
    std::env::set_var("WF_DB_PATH", dir.path().join("app.db"));
    std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
    let alert = "INR 120 debited from account 1234 on 2026-09-10 at Test cafe. Ref 001234.";
    let extraction = serde_json::json!({"events":[{"accountHint":"1234","amount":"120","currency":"INR","date":"2026-09-10","direction":"debit","merchant":"Test cafe","reference":"001234","kind":"payment","state":"completed","sourceText":alert}]});
    let model = axum::Router::new().route("/v1/chat/completions", axum::routing::post(move |axum::Json(body): axum::Json<serde_json::Value>| {
        let extraction = extraction.clone(); async move {
            assert_eq!(body["model"], "openai.gpt-5.6-luna");
            assert_eq!(body["response_format"]["json_schema"]["strict"], true);
            assert!(body.get("tools").is_none());
            axum::Json(serde_json::json!({"choices":[{"finish_reason":"stop","message":{"content":extraction.to_string()}}],"usage":{"prompt_tokens":500,"completion_tokens":120}}))
        }
    }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}/v1", listener.local_addr().unwrap());
    let model_task = tokio::spawn(async move { axum::serve(listener, model).await.unwrap() });
    let config = test_config(
        dir.path().join("app.db").display().to_string(),
        dir.path().join("addons").display().to_string(),
    );
    let state = build_state(&config).await.unwrap();
    state
        .secret_store
        .set_secret("ai_bedrock", "synthetic-key")
        .unwrap();
    let app = app_router(state, &config);
    let provider = request(
        app.clone(),
        "PUT",
        "/api/v1/ai/providers/settings",
        serde_json::json!({"providerId":"bedrock","enabled":true,"customUrl":endpoint}),
    )
    .await;
    assert!(provider.0.is_success(), "{:?}", provider);
    let account = request(app.clone(), "POST", "/api/v1/accounts", serde_json::json!({"name":"Test bank","accountType":"CASH","currency":"INR","isDefault":false,"isActive":true,"trackingMode":"TRANSACTIONS"})).await.1;
    let settings = request(app.clone(), "PUT", "/api/v1/quick-add/settings", serde_json::json!({"provider":"bedrock","model":"openai.gpt-5.6-luna","monthlyBudgetMicros":1000000,"timezone":"Asia/Kolkata","mappings":[{"alias":"1234","accountId":account["id"]}]})).await;
    assert!(settings.0.is_success(), "{:?}", settings);
    let receipt = capture(app.clone(), alert).await;
    assert!(receipt.0.is_success(), "{:?}", receipt);
    let path = format!("/api/v1/captures/{}", receipt.1["id"].as_str().unwrap());
    let completed = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let receipt = request(app.clone(), "GET", &path, serde_json::Value::Null)
                .await
                .1;
            if receipt["status"] != "processing" {
                break receipt;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    model_task.abort();
    assert_eq!(
        request(
            app.clone(),
            "GET",
            "/api/v1/quick-add/usage",
            serde_json::Value::Null
        )
        .await
        .1["reservedOrUsedMicros"],
        269
    );
    assert_eq!(
        completed["candidates"][0]["fields"]["accountId"],
        account["id"]
    );
    assert_eq!(completed["candidates"][0]["fields"]["amount"], "120");
    assert_eq!(completed["reviews"][0]["reason"], "supervised_review");
    assert_eq!(
        request(
            app.clone(),
            "POST",
            "/api/v1/activities/search",
            serde_json::json!({"page":0,"pageSize":20})
        )
        .await
        .1["meta"]["totalRowCount"],
        0
    );
    let reviewed = request(app, "POST", &format!("/api/v1/capture-reviews/{}/resolve", completed["reviews"][0]["id"].as_str().unwrap()), serde_json::json!({"version":completed["version"],"fields":completed["candidates"][0]["fields"]})).await;
    assert!(reviewed.0.is_success(), "{:?}", reviewed);
    assert_eq!(reviewed.1["candidates"].as_array().unwrap().len(), 1);
    assert_eq!(reviewed.1["candidates"][0]["status"], "posted");
}

async fn settled(app: axum::Router, id: &str) -> serde_json::Value {
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let receipt = request(
                app.clone(),
                "GET",
                &format!("/api/v1/captures/{id}"),
                serde_json::Value::Null,
            )
            .await
            .1;
            if receipt["status"] != "processing" {
                break receipt;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn exhausted_budget_accepts_the_text_into_review_without_model_processing() {
    let _guard = ENV_LOCK.lock().await;
    let dir = tempdir().unwrap();
    std::env::set_var("WF_DB_PATH", dir.path().join("app.db"));
    std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
    let config = test_config(
        dir.path().join("app.db").display().to_string(),
        dir.path().join("addons").display().to_string(),
    );
    let app = app_router(build_state(&config).await.unwrap(), &config);
    let result = request(app.clone(), "PUT", "/api/v1/quick-add/settings", serde_json::json!({"provider":"bedrock","model":"openai.gpt-5.6-luna","monthlyBudgetMicros":1,"timezone":"Asia/Kolkata"})).await;
    assert!(result.0.is_success());
    let receipt = capture(app.clone(), "Paid INR 100").await;
    assert!(receipt.0.is_success());
    let receipt = settled(app, receipt.1["id"].as_str().unwrap()).await;
    assert_eq!(receipt["reviews"][0]["reason"], "budget_exhausted");
    assert_eq!(receipt["input"]["text"], "Paid INR 100");
}

async fn authorized(
    app: axum::Router,
    method: &str,
    path: &str,
    token: &str,
    body: serde_json::Value,
) -> (axum::http::StatusCode, serde_json::Value) {
    let response = app
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null),
    )
}

#[tokio::test]
async fn shortcut_credentials_only_submit_and_read_their_own_receipts_and_can_be_revoked() {
    use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
    let _guard = ENV_LOCK.lock().await;
    let dir = tempdir().unwrap();
    std::env::set_var("WF_DB_PATH", dir.path().join("app.db"));
    std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
    let mut config = test_config(
        dir.path().join("app.db").display().to_string(),
        dir.path().join("addons").display().to_string(),
    );
    config.auth = Some(wealthfolio_server::auth::AuthConfig {
        password_hash: Some(
            Argon2::default()
                .hash_password(
                    b"synthetic-password",
                    &SaltString::encode_b64(b"synthetic-salt-123").unwrap(),
                )
                .unwrap()
                .to_string(),
        ),
        jwt_secret: vec![9; 32],
        access_token_ttl: Duration::from_secs(3600),
        cookie_secure: wealthfolio_server::auth::CookieSecurePolicy::Never,
    });
    let state = build_state(&config).await.unwrap();
    let admin = state.auth.as_ref().unwrap().issue_token().unwrap();
    let app = app_router(state, &config);
    let one = authorized(
        app.clone(),
        "POST",
        "/api/v1/quick-add/tokens",
        &admin,
        serde_json::json!({"name":"Phone one"}),
    )
    .await;
    assert!(one.0.is_success(), "{:?}", one);
    assert!(one.1.get("tokenHash").is_none());
    let two = authorized(
        app.clone(),
        "POST",
        "/api/v1/quick-add/tokens",
        &admin,
        serde_json::json!({"name":"Phone two"}),
    )
    .await
    .1;
    let token = one.1["token"].as_str().unwrap();
    let capture_body = serde_json::json!({"clientRequestId":"phone-1","text":"Paid INR 20","inputKind":"typed_note"});
    let receipt = authorized(
        app.clone(),
        "POST",
        "/api/v1/captures",
        token,
        capture_body.clone(),
    )
    .await;
    assert!(receipt.0.is_success(), "{:?}", receipt);
    let path = format!("/api/v1/captures/{}", receipt.1["id"].as_str().unwrap());
    assert!(
        authorized(app.clone(), "GET", &path, token, serde_json::Value::Null)
            .await
            .0
            .is_success()
    );
    assert_eq!(
        authorized(
            app.clone(),
            "GET",
            &path,
            two["token"].as_str().unwrap(),
            serde_json::Value::Null
        )
        .await
        .0,
        axum::http::StatusCode::NOT_FOUND
    );
    assert_eq!(
        authorized(
            app.clone(),
            "GET",
            "/api/v1/accounts",
            token,
            serde_json::Value::Null
        )
        .await
        .0,
        axum::http::StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        authorized(
            app.clone(),
            "GET",
            "/api/v1/capture-reviews",
            token,
            serde_json::Value::Null
        )
        .await
        .0,
        axum::http::StatusCode::UNAUTHORIZED
    );
    let path = format!("/api/v1/quick-add/tokens/{}", one.1["id"].as_str().unwrap());
    assert!(authorized(
        app.clone(),
        "DELETE",
        &path,
        &admin,
        serde_json::Value::Null
    )
    .await
    .0
    .is_success());
    assert_eq!(
        authorized(app, "POST", "/api/v1/captures", token, capture_body)
            .await
            .0,
        axum::http::StatusCode::UNAUTHORIZED
    );
}

#[tokio::test]
async fn manual_balance_followup_completes_without_adding_another_payment() {
    let _guard = ENV_LOCK.lock().await;
    let dir = tempdir().unwrap();
    std::env::set_var("WF_DB_PATH", dir.path().join("app.db"));
    std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
    let config = test_config(
        dir.path().join("app.db").display().to_string(),
        dir.path().join("addons").display().to_string(),
    );
    let app = app_router(build_state(&config).await.unwrap(), &config);
    let account = request(app.clone(), "POST", "/api/v1/accounts", serde_json::json!({"name":"Test bank","accountType":"CASH","currency":"INR","isDefault":false,"isActive":true,"trackingMode":"TRANSACTIONS"})).await.1;
    let capture = capture(app.clone(), "Paid loan installment INR 1000")
        .await
        .1;
    let path = format!(
        "/api/v1/capture-reviews/{}/resolve",
        capture["reviews"][0]["id"].as_str().unwrap()
    );
    let result = request(app.clone(), "POST", &path, serde_json::json!({"version":1,"fields":{"accountId":account["id"],"amount":"1000","currency":"INR","date":"2026-09-10","direction":"debit","merchant":"Test loan","reference":null,"kind":"loan_payment"}})).await;
    assert!(result.0.is_success(), "{:?}", result);
    let review = result.1["reviews"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["reason"] == "balance_update_required")
        .expect("balance review");
    assert_eq!(
        review["activityId"],
        result.1["candidates"][0]["activityId"]
    );
    let path = format!(
        "/api/v1/capture-reviews/{}/resolve",
        review["id"].as_str().unwrap()
    );
    let completed = request(
        app.clone(),
        "POST",
        &path,
        serde_json::json!({"version":result.1["version"],"action":"complete"}),
    )
    .await;
    assert!(completed.0.is_success(), "{:?}", completed);
    assert_eq!(
        request(
            app,
            "POST",
            "/api/v1/activities/search",
            serde_json::json!({"page":0,"pageSize":20})
        )
        .await
        .1["meta"]["totalRowCount"],
        1
    );
}

#[tokio::test]
async fn restart_recovers_a_payment_written_before_its_receipt_was_saved() {
    let _guard = ENV_LOCK.lock().await;
    let dir = tempdir().unwrap();
    std::env::set_var("WF_DB_PATH", dir.path().join("app.db"));
    std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
    let config = test_config(
        dir.path().join("app.db").display().to_string(),
        dir.path().join("addons").display().to_string(),
    );
    let state = build_state(&config).await.unwrap();
    let (written, _release) = state.capture_service.pause_after_write_for_test();
    let app = app_router(state.clone(), &config);
    let account = request(app.clone(), "POST", "/api/v1/accounts", serde_json::json!({"name":"Test bank","accountType":"CASH","currency":"INR","isDefault":false,"isActive":true,"trackingMode":"TRANSACTIONS"})).await.1;
    let captured = capture(app.clone(), "Paid INR 45").await.1;
    let id = captured["id"].as_str().unwrap().to_string();
    let review = captured["reviews"][0]["id"].as_str().unwrap().to_string();
    let write_app = app.clone();
    let write_path = format!("/api/v1/capture-reviews/{review}/resolve");
    let fields = serde_json::json!({"accountId":account["id"],"amount":"45","currency":"INR","date":"2026-09-10","direction":"debit","merchant":"Cafe","reference":null,"kind":"payment"});
    let write_fields = fields.clone();
    let saving = tokio::spawn(async move {
        request(
            write_app,
            "POST",
            &write_path,
            serde_json::json!({"version":1,"fields":write_fields}),
        )
        .await
    });
    tokio::time::timeout(Duration::from_secs(5), written.notified())
        .await
        .unwrap();
    let busy = request(
        app.clone(),
        "GET",
        &format!("/api/v1/captures/{id}"),
        serde_json::Value::Null,
    )
    .await
    .1;
    let mut changed = fields;
    changed["amount"] = serde_json::json!("90");
    assert_eq!(
        request(
            app.clone(),
            "POST",
            &format!("/api/v1/capture-reviews/{review}/resolve"),
            serde_json::json!({"version":busy["version"],"fields":changed})
        )
        .await
        .0,
        axum::http::StatusCode::CONFLICT
    );
    assert_eq!(
        request(
            app.clone(),
            "POST",
            &format!("/api/v1/capture-reviews/{review}/dismiss"),
            serde_json::json!({"version":busy["version"]})
        )
        .await
        .0,
        axum::http::StatusCode::CONFLICT
    );
    saving.abort();
    let _ = saving.await;
    drop(app);
    drop(state);
    let restarted = build_state(&config).await.unwrap();
    restarted.capture_service.advance_lease_clock_for_test(121);
    let app = app_router(restarted, &config);
    let recovered = settled(app.clone(), &id).await;
    assert_eq!(recovered["candidates"][0]["status"], "posted");
    assert!(recovered["reviews"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["reason"] == "category_required"));
    assert_eq!(
        request(
            app,
            "POST",
            "/api/v1/activities/search",
            serde_json::json!({"page":0,"pageSize":20})
        )
        .await
        .1["meta"]["totalRowCount"],
        1
    );
}

#[tokio::test]
async fn text_cleanup_preserves_unresolved_evidence_and_request_deduplication() {
    let _guard = ENV_LOCK.lock().await;
    let dir = tempdir().unwrap();
    std::env::set_var("WF_DB_PATH", dir.path().join("app.db"));
    std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
    let config = test_config(
        dir.path().join("app.db").display().to_string(),
        dir.path().join("addons").display().to_string(),
    );
    let app = app_router(build_state(&config).await.unwrap(), &config);
    let captured = capture(app.clone(), "Unclear private text").await.1;
    let path = format!(
        "/api/v1/captures/{}/source",
        captured["id"].as_str().unwrap()
    );
    assert_eq!(
        request(
            app.clone(),
            "DELETE",
            &path,
            serde_json::json!({"version":1})
        )
        .await
        .0,
        axum::http::StatusCode::CONFLICT
    );
    let dismissed = request(
        app.clone(),
        "POST",
        &format!(
            "/api/v1/capture-reviews/{}/dismiss",
            captured["reviews"][0]["id"].as_str().unwrap()
        ),
        serde_json::json!({"version":1}),
    )
    .await
    .1;
    assert!(request(
        app.clone(),
        "DELETE",
        &path,
        serde_json::json!({"version":dismissed["version"]})
    )
    .await
    .0
    .is_success());
    let replay = capture(app, "Unclear private text").await.1;
    assert_eq!(replay["id"], captured["id"]);
    assert_eq!(replay["input"]["text"], "");
}

#[tokio::test]
async fn concurrent_captures_of_the_same_referenced_payment_share_one_activity() {
    let _guard = ENV_LOCK.lock().await;
    let dir = tempdir().unwrap();
    std::env::set_var("WF_DB_PATH", dir.path().join("app.db"));
    std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
    let config = test_config(
        dir.path().join("app.db").display().to_string(),
        dir.path().join("addons").display().to_string(),
    );
    let app = app_router(build_state(&config).await.unwrap(), &config);
    let account = request(app.clone(), "POST", "/api/v1/accounts", serde_json::json!({"name":"Bank","accountType":"CASH","currency":"INR","isDefault":false,"isActive":true,"trackingMode":"TRANSACTIONS"})).await.1;
    let first = request(app.clone(), "POST", "/api/v1/captures", serde_json::json!({"clientRequestId":"first","text":"INR 10 debited ref 000123","inputKind":"bank_alert"})).await.1;
    let second = request(app.clone(), "POST", "/api/v1/captures", serde_json::json!({"clientRequestId":"second","text":"Payment INR 10 ref 000123","inputKind":"bank_alert"})).await.1;
    let body = serde_json::json!({"version":1,"fields":{"accountId":account["id"],"amount":"10","currency":"INR","date":"2026-09-10","direction":"debit","merchant":"Cafe","reference":"000123","kind":"payment"}});
    let path1 = format!(
        "/api/v1/capture-reviews/{}/resolve",
        first["reviews"][0]["id"].as_str().unwrap()
    );
    let path2 = format!(
        "/api/v1/capture-reviews/{}/resolve",
        second["reviews"][0]["id"].as_str().unwrap()
    );
    let (a, b) = tokio::join!(
        request(app.clone(), "POST", &path1, body.clone()),
        request(app.clone(), "POST", &path2, body)
    );
    assert!(a.0.is_success() && b.0.is_success(), "{:?} {:?}", a, b);
    assert_eq!(
        a.1["candidates"][0]["activityId"],
        b.1["candidates"][0]["activityId"]
    );
    assert_eq!(
        request(
            app,
            "POST",
            "/api/v1/activities/search",
            serde_json::json!({"page":0,"pageSize":20})
        )
        .await
        .1["meta"]["totalRowCount"],
        1
    );
}

#[tokio::test]
async fn qualified_automatic_capture_posts_an_evidenced_payment_and_keeps_category_review() {
    let _guard = ENV_LOCK.lock().await;
    let dir = tempdir().unwrap();
    std::env::set_var("WF_DB_PATH", dir.path().join("app.db"));
    std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
    let alert = "INR 120 debited from account XX1234 on 2026-09-10 at Test cafe. Ref 001234.";
    let extraction = serde_json::json!({"events":[{"accountHint":"XX1234","amount":"120","currency":"INR","date":"2026-09-10","direction":"debit","merchant":"Test cafe","reference":"001234","kind":"payment","state":"completed","sourceText":alert}]});
    let model = axum::Router::new().route("/v1/chat/completions", axum::routing::post(move |axum::Json(body): axum::Json<serde_json::Value>| {
        let extraction = extraction.clone(); async move {
            assert_eq!(body["model"], "openai.gpt-5.6-luna");
            assert_eq!(body["response_format"]["json_schema"]["strict"], true);
            assert!(body.get("tools").is_none());
            axum::Json(serde_json::json!({"choices":[{"finish_reason":"stop","message":{"content":extraction.to_string()}}],"usage":{"prompt_tokens":500,"completion_tokens":120}}))
        }
    }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}/v1", listener.local_addr().unwrap());
    let model_task = tokio::spawn(async move { axum::serve(listener, model).await.unwrap() });
    let config = test_config(
        dir.path().join("app.db").display().to_string(),
        dir.path().join("addons").display().to_string(),
    );
    let state = build_state(&config).await.unwrap();
    state
        .secret_store
        .set_secret("ai_bedrock", "synthetic-key")
        .unwrap();
    let app = app_router(state, &config);
    let provider = request(
        app.clone(),
        "PUT",
        "/api/v1/ai/providers/settings",
        serde_json::json!({"providerId":"bedrock","enabled":true,"customUrl":endpoint}),
    )
    .await;
    assert!(provider.0.is_success(), "{:?}", provider);
    let account = request(app.clone(), "POST", "/api/v1/accounts", serde_json::json!({"name":"Test bank","accountType":"CASH","currency":"INR","isDefault":false,"isActive":true,"trackingMode":"TRANSACTIONS"})).await.1;
    let settings = request(app.clone(), "PUT", "/api/v1/quick-add/settings", serde_json::json!({"provider":"bedrock","model":"openai.gpt-5.6-luna","monthlyBudgetMicros":1000000,"automaticPosting":true,"evaluationPassed":true,"extractionVersion":"bedrock-luna-capture-v3","supervisedTrialCompleted":true,"timezone":"Asia/Kolkata","mappings":[{"alias":"1234","accountId":account["id"]}]})).await;
    assert!(settings.0.is_success(), "{:?}", settings);
    let receipt = capture(app.clone(), alert).await;
    assert!(receipt.0.is_success(), "{:?}", receipt);
    let path = format!("/api/v1/captures/{}", receipt.1["id"].as_str().unwrap());
    let completed = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let receipt = request(app.clone(), "GET", &path, serde_json::Value::Null)
                .await
                .1;
            if receipt["status"] != "processing" {
                break receipt;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    model_task.abort();
    assert_eq!(
        request(
            app.clone(),
            "GET",
            "/api/v1/quick-add/usage",
            serde_json::Value::Null
        )
        .await
        .1["reservedOrUsedMicros"],
        269
    );
    assert_eq!(
        completed["candidates"][0]["fields"]["accountId"],
        account["id"]
    );
    assert_eq!(completed["candidates"][0]["fields"]["amount"], "120");
    assert_eq!(completed["candidates"][0]["status"], "posted");
    assert!(completed["reviews"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["reason"] == "category_required" && r["status"] == "open"));
    assert_eq!(
        request(
            app,
            "POST",
            "/api/v1/activities/search",
            serde_json::json!({"page":0,"pageSize":20})
        )
        .await
        .1["meta"]["totalRowCount"],
        1
    );
}

#[tokio::test]
async fn retry_keeps_capture_identity_and_does_not_duplicate_review() {
    let _guard = ENV_LOCK.lock().await;
    let dir = tempdir().unwrap();
    std::env::set_var("WF_DB_PATH", dir.path().join("app.db"));
    std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
    let config = test_config(
        dir.path().join("app.db").display().to_string(),
        dir.path().join("addons").display().to_string(),
    );
    let app = app_router(build_state(&config).await.unwrap(), &config);
    let original = capture(app.clone(), "Paid INR 45").await.1;
    let settings = request(app.clone(), "PUT", "/api/v1/quick-add/settings", serde_json::json!({"provider":"bedrock","model":"openai.gpt-5.6-luna","monthlyBudgetMicros":1,"timezone":"Asia/Kolkata"})).await;
    assert!(settings.0.is_success());
    let retried = request(
        app.clone(),
        "POST",
        &format!(
            "/api/v1/captures/{}/retry",
            original["id"].as_str().unwrap()
        ),
        serde_json::json!({"version":original["version"]}),
    )
    .await;
    assert!(retried.0.is_success(), "{:?}", retried);
    assert_eq!(retried.1["id"], original["id"]);
    let finished = settled(app.clone(), original["id"].as_str().unwrap()).await;
    assert_eq!(
        finished["reviews"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["status"] == "open")
            .count(),
        1
    );
    assert_eq!(finished["reviews"][0]["reason"], "budget_exhausted");
    assert_eq!(
        request(
            app,
            "POST",
            "/api/v1/activities/search",
            serde_json::json!({"page":0,"pageSize":20})
        )
        .await
        .1["meta"]["totalRowCount"],
        0
    );
}

#[tokio::test]
async fn referenced_statement_and_capture_reconcile_in_both_orders() {
    let _guard = ENV_LOCK.lock().await;
    for statement_first in [true, false] {
        let dir = tempdir().unwrap();
        std::env::set_var("WF_DB_PATH", dir.path().join("app.db"));
        std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
        let config = test_config(
            dir.path().join("app.db").display().to_string(),
            dir.path().join("addons").display().to_string(),
        );
        let app = app_router(build_state(&config).await.unwrap(), &config);
        let account=request(app.clone(),"POST","/api/v1/accounts",serde_json::json!({"name":"Test bank","accountType":"CASH","currency":"INR","isDefault":false,"isActive":true,"trackingMode":"TRANSACTIONS"})).await.1;
        let import = serde_json::json!({"activities":[{"date":"2026-09-10","symbol":"","activityType":"WITHDRAWAL","amount":"120","currency":"INR","accountId":account["id"],"isDraft":false,"isValid":true,"bankReference":"00123456"}]});
        if statement_first {
            let result = request(
                app.clone(),
                "POST",
                "/api/v1/activities/import",
                import.clone(),
            )
            .await;
            assert!(result.0.is_success(), "{:?}", result);
        }
        let saved = capture(app.clone(), "INR 120 debited. Ref 00123456")
            .await
            .1;
        let result=request(app.clone(),"POST",&format!("/api/v1/capture-reviews/{}/resolve",saved["reviews"][0]["id"].as_str().unwrap()),serde_json::json!({"version":saved["version"],"fields":{"accountId":account["id"],"amount":"120","currency":"INR","date":"2026-09-10","direction":"debit","merchant":"Cafe","reference":"00123456","kind":"payment"}})).await;
        assert!(result.0.is_success(), "{:?}", result);
        if !statement_first {
            assert!(request(
                app.clone(),
                "PUT",
                "/api/v1/spending/settings",
                serde_json::json!({"enabled":true,"accountIds":[account["id"]]})
            )
            .await
            .0
            .is_success());
            let category_review = result.1["reviews"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["reason"] == "category_required")
                .unwrap();
            assert!(request(app.clone(),"POST",&format!("/api/v1/capture-reviews/{}/resolve",category_review["id"].as_str().unwrap()),serde_json::json!({"version":result.1["version"],"action":"categorize","taxonomyId":"spending_categories","categoryId":"cat_food_coffee"})).await.0.is_success());
            let imported = request(
                app.clone(),
                "POST",
                "/api/v1/activities/import",
                import.clone(),
            )
            .await;
            assert!(imported.0.is_success(), "{:?}", imported);
            let activity_id = result.1["candidates"][0]["activityId"].as_str().unwrap();
            let assignments = request(
                app.clone(),
                "GET",
                &format!("/api/v1/spending/activities/{activity_id}/assignments"),
                serde_json::Value::Null,
            )
            .await
            .1;
            assert_eq!(assignments[0]["categoryId"], "cat_food_coffee");
        }
        let mut contradictory = import;
        contradictory["activities"][0]["date"] = serde_json::json!("2026-09-11");
        let conflict = request(
            app.clone(),
            "POST",
            "/api/v1/activities/import",
            contradictory,
        )
        .await
        .1;
        assert!(
            conflict["activities"][0]["errors"]["bankReference"].is_array(),
            "{conflict}"
        );
        let activities = request(
            app,
            "POST",
            "/api/v1/activities/search",
            serde_json::json!({"page":0,"pageSize":20}),
        )
        .await
        .1;
        assert_eq!(
            activities["meta"]["totalRowCount"], 1,
            "statement_first={statement_first}: {activities}"
        );
    }
}

#[tokio::test]
async fn merchant_correction_categorizes_a_future_capture_without_rewriting_history() {
    let _guard = ENV_LOCK.lock().await;
    let dir = tempdir().unwrap();
    std::env::set_var("WF_DB_PATH", dir.path().join("app.db"));
    std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
    let config = test_config(
        dir.path().join("app.db").display().to_string(),
        dir.path().join("addons").display().to_string(),
    );
    let app = app_router(build_state(&config).await.unwrap(), &config);
    let account=request(app.clone(),"POST","/api/v1/accounts",serde_json::json!({"name":"Test bank","accountType":"CASH","currency":"INR","isDefault":false,"isActive":true,"trackingMode":"TRANSACTIONS"})).await.1;
    assert!(request(
        app.clone(),
        "PUT",
        "/api/v1/spending/settings",
        serde_json::json!({"enabled":true,"accountIds":[account["id"]]})
    )
    .await
    .0
    .is_success());
    for index in 0..2 {
        let capture=request(app.clone(),"POST","/api/v1/captures",serde_json::json!({"clientRequestId":format!("learn-{index}"),"text":"Coffee","inputKind":"typed_note"})).await.1;
        let saved=request(app.clone(),"POST",&format!("/api/v1/capture-reviews/{}/resolve",capture["reviews"][0]["id"].as_str().unwrap()),serde_json::json!({"version":capture["version"],"fields":{"accountId":account["id"],"amount":"120","currency":"INR","date":format!("2026-09-1{index}"),"direction":"debit","merchant":"Test cafe","reference":null,"kind":"payment"}})).await.1;
        if index == 0 {
            let review = saved["reviews"]
                .as_array()
                .unwrap()
                .iter()
                .find(|r| r["reason"] == "category_required")
                .unwrap();
            let result=request(app.clone(),"POST",&format!("/api/v1/capture-reviews/{}/resolve",review["id"].as_str().unwrap()),serde_json::json!({"version":saved["version"],"action":"categorize","taxonomyId":"spending_categories","categoryId":"cat_food_coffee"})).await;
            assert!(result.0.is_success(), "{:?}", result);
        } else {
            assert_eq!(saved["status"], "complete", "{saved}");
            let id = saved["candidates"][0]["activityId"].as_str().unwrap();
            let assignments = request(
                app.clone(),
                "GET",
                &format!("/api/v1/spending/activities/{id}/assignments"),
                serde_json::Value::Null,
            )
            .await
            .1;
            assert_eq!(assignments[0]["categoryId"], "cat_food_coffee");
        }
    }
    assert_eq!(
        request(
            app,
            "POST",
            "/api/v1/activities/search",
            serde_json::json!({"page":0,"pageSize":20})
        )
        .await
        .1["meta"]["totalRowCount"],
        2
    );
}

#[tokio::test]
async fn an_owned_transfer_side_is_neutral_and_waits_for_its_counterpart() {
    let _guard = ENV_LOCK.lock().await;
    let dir = tempdir().unwrap();
    std::env::set_var("WF_DB_PATH", dir.path().join("app.db"));
    std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
    let config = test_config(
        dir.path().join("app.db").display().to_string(),
        dir.path().join("addons").display().to_string(),
    );
    let app = app_router(build_state(&config).await.unwrap(), &config);
    let account=request(app.clone(),"POST","/api/v1/accounts",serde_json::json!({"name":"Test bank","accountType":"CASH","currency":"INR","isDefault":false,"isActive":true,"trackingMode":"TRANSACTIONS"})).await.1;
    let captured = capture(app.clone(), "Transfer to my other account").await.1;
    let saved=request(app.clone(),"POST",&format!("/api/v1/capture-reviews/{}/resolve",captured["reviews"][0]["id"].as_str().unwrap()),serde_json::json!({"version":captured["version"],"fields":{"accountId":account["id"],"amount":"120","currency":"INR","date":"2026-09-10","direction":"debit","merchant":null,"reference":"00008888","kind":"transfer"}})).await;
    assert!(saved.0.is_success(), "{:?}", saved);
    assert!(saved.1["reviews"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["reason"] == "counterpart_required" && r["status"] == "open"));
    let activities = request(
        app,
        "POST",
        "/api/v1/activities/search",
        serde_json::json!({"page":0,"pageSize":20}),
    )
    .await
    .1;
    assert_eq!(activities["meta"]["totalRowCount"], 1);
    assert_eq!(activities["data"][0]["activityType"], "TRANSFER_OUT");
    assert_eq!(
        activities["data"][0]["metadata"]["flow"]["is_external"],
        false
    );
}

#[tokio::test]
async fn matching_owned_transfer_alerts_link_without_reposting_either_side() {
    let _guard = ENV_LOCK.lock().await;
    let dir = tempdir().unwrap();
    std::env::set_var("WF_DB_PATH", dir.path().join("app.db"));
    std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
    let config = test_config(
        dir.path().join("app.db").display().to_string(),
        dir.path().join("addons").display().to_string(),
    );
    let app = app_router(build_state(&config).await.unwrap(), &config);
    let mut receipts = Vec::new();
    for (index, direction) in ["debit", "credit"].iter().enumerate() {
        let account=request(app.clone(),"POST","/api/v1/accounts",serde_json::json!({"name":format!("Transfer bank {index}"),"accountType":"CASH","currency":"INR","isDefault":false,"isActive":true,"trackingMode":"TRANSACTIONS"})).await.1;
        let captured=request(app.clone(),"POST","/api/v1/captures",serde_json::json!({"clientRequestId":format!("transfer-{index}"),"text":"Synthetic owned transfer", "inputKind":"bank_alert"})).await.1;
        let saved=request(app.clone(),"POST",&format!("/api/v1/capture-reviews/{}/resolve",captured["reviews"][0]["id"].as_str().unwrap()),serde_json::json!({"version":captured["version"],"fields":{"accountId":account["id"],"amount":"120","currency":"INR","date":"2026-09-10","direction":direction,"merchant":null,"reference":"000777","kind":"transfer"}})).await;
        assert!(saved.0.is_success(), "Transfer side {index} rejected");
        receipts.push(saved.1);
    }
    let ledger = request(
        app.clone(),
        "POST",
        "/api/v1/activities/search",
        serde_json::json!({"page":0,"pageSize":20}),
    )
    .await
    .1;
    assert_eq!(ledger["meta"]["totalRowCount"], 2);
    assert!(ledger["data"][0]["sourceGroupId"].is_string());
    assert_eq!(
        ledger["data"][0]["sourceGroupId"],
        ledger["data"][1]["sourceGroupId"]
    );
    for receipt in receipts {
        let receipt = request(
            app.clone(),
            "GET",
            &format!("/api/v1/captures/{}", receipt["id"].as_str().unwrap()),
            serde_json::Value::Null,
        )
        .await
        .1;
        assert!(!receipt["reviews"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["reason"] == "counterpart_required" && r["status"] == "open"));
    }
}

#[tokio::test]
async fn legacy_references_match_but_ambiguous_and_contradictory_references_do_not() {
    let _guard = ENV_LOCK.lock().await;
    for existing_count in [0, 1, 2] {
        let dir = tempdir().unwrap();
        std::env::set_var("WF_DB_PATH", dir.path().join("app.db"));
        std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
        let config = test_config(
            dir.path().join("app.db").display().to_string(),
            dir.path().join("addons").display().to_string(),
        );
        let app = app_router(build_state(&config).await.unwrap(), &config);
        let account=request(app.clone(),"POST","/api/v1/accounts",serde_json::json!({"name":"Reference test bank","accountType":"CASH","currency":"INR","isDefault":false,"isActive":true,"trackingMode":"TRANSACTIONS"})).await.1;
        for index in 0..existing_count {
            let created = if index == 0 {
                request(app.clone(),"POST","/api/v1/activities",serde_json::json!({"accountId":account["id"],"activityType":"WITHDRAWAL","activityDate":"2026-09-10T12:00:00Z","amount":"120","currency":"INR","sourceRecordId":"000999","sourceSystem":"CSV"})).await
            } else {
                request(app.clone(),"POST","/api/v1/activities/import",serde_json::json!({"activities":[{"date":"2026-09-10","symbol":"","activityType":"WITHDRAWAL","amount":"120","currency":"INR","accountId":account["id"],"isDraft":false,"isValid":true,"bankReference":"000999","forceImport":true}]})).await
            };
            assert!(
                created.0.is_success(),
                "Legacy fixture creation failed: {:?}",
                created
            );
        }
        if existing_count == 0 {
            let mut rows = serde_json::json!({"activities":[{"date":"2026-09-10","symbol":"","activityType":"WITHDRAWAL","amount":"120","currency":"INR","accountId":account["id"],"isDraft":false,"isValid":true,"bankReference":"000999"},{"date":"2026-09-10","symbol":"","activityType":"TRANSFER_OUT","amount":"120","currency":"INR","accountId":account["id"],"isDraft":false,"isValid":true,"bankReference":"000999"}]});
            let mut repeated = rows["activities"][0].clone();
            rows["activities"]
                .as_array_mut()
                .unwrap()
                .insert(1, repeated.clone());
            repeated["forceImport"] = serde_json::json!(true);
            rows["activities"].as_array_mut().unwrap().push(repeated);
            let imported = request(app.clone(), "POST", "/api/v1/activities/import", rows).await;
            assert!(imported.0.is_success());
            assert!(imported.1["activities"]
                .as_array()
                .unwrap()
                .iter()
                .all(|row| row["errors"]["bankReference"].is_array()));
        } else {
            let captured = capture(app.clone(), "Legacy reference 000999").await.1;
            let saved=request(app.clone(),"POST",&format!("/api/v1/capture-reviews/{}/resolve",captured["reviews"][0]["id"].as_str().unwrap()),serde_json::json!({"version":captured["version"],"fields":{"accountId":account["id"],"amount":"120","currency":"INR","date":"2026-09-10","direction":"debit","merchant":null,"reference":"000999","kind":"payment"}})).await;
            assert!(saved.0.is_success());
            if existing_count == 1 {
                assert_eq!(saved.1["candidates"][0]["status"], "already_recorded");
            } else {
                assert_eq!(saved.1["candidates"][0]["status"], "needs_review");
            }
        }
        if existing_count == 1 {
            let captured = request(app.clone(), "POST", "/api/v1/captures", serde_json::json!({"clientRequestId":"transfer-reference-check","text":"Transfer reference 000999","inputKind":"bank_alert"})).await.1;
            let saved = request(app.clone(), "POST", &format!("/api/v1/capture-reviews/{}/resolve", captured["reviews"][0]["id"].as_str().unwrap()), serde_json::json!({"version":captured["version"],"fields":{"accountId":account["id"],"amount":"120","currency":"INR","date":"2026-09-10","direction":"debit","merchant":null,"reference":"000999","kind":"transfer"}})).await;
            assert!(saved.0.is_success(), "{:?}", saved);
            assert_eq!(saved.1["candidates"][0]["status"], "needs_review");
        }
        assert_eq!(
            request(
                app,
                "POST",
                "/api/v1/activities/search",
                serde_json::json!({"page":0,"pageSize":20})
            )
            .await
            .1["meta"]["totalRowCount"],
            existing_count
        );
    }
}

#[tokio::test]
async fn a_partial_refund_links_to_the_original_without_rewriting_either_amount() {
    let _guard = ENV_LOCK.lock().await;
    let dir = tempdir().unwrap();
    std::env::set_var("WF_DB_PATH", dir.path().join("app.db"));
    std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
    let config = test_config(
        dir.path().join("app.db").display().to_string(),
        dir.path().join("addons").display().to_string(),
    );
    let app = app_router(build_state(&config).await.unwrap(), &config);
    let account=request(app.clone(),"POST","/api/v1/accounts",serde_json::json!({"name":"Refund test bank","accountType":"CASH","currency":"INR","isDefault":false,"isActive":true,"trackingMode":"TRANSACTIONS"})).await.1;
    let mut saved = Vec::new();
    for (index, (kind, direction, amount)) in
        [("payment", "debit", "120"), ("refund", "credit", "40")]
            .iter()
            .enumerate()
    {
        let captured=request(app.clone(),"POST","/api/v1/captures",serde_json::json!({"clientRequestId":format!("refund-{index}"),"text":"Synthetic refund test","inputKind":"bank_alert"})).await.1;
        let receipt=request(app.clone(),"POST",&format!("/api/v1/capture-reviews/{}/resolve",captured["reviews"][0]["id"].as_str().unwrap()),serde_json::json!({"version":captured["version"],"fields":{"accountId":account["id"],"amount":amount,"currency":"INR","date":"2026-09-10","direction":direction,"merchant":"Test shop","reference":null,"kind":kind}})).await;
        assert!(
            receipt.0.is_success(),
            "Refund fixture rejected: {}",
            receipt.0
        );
        saved.push(receipt.1);
    }
    let original_id = saved[0]["candidates"][0]["activityId"].as_str().unwrap();
    let review = saved[1]["reviews"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["reason"] == "original_payment_required")
        .unwrap();
    let path = format!(
        "/api/v1/capture-reviews/{}/resolve",
        review["id"].as_str().unwrap()
    );
    let body = serde_json::json!({"version":saved[1]["version"],"action":"link_refund","relatedActivityId":original_id});
    let linked = request(app.clone(), "POST", &path, body.clone()).await;
    assert!(linked.0.is_success(), "Refund link rejected: {:?}", linked);
    assert!(request(app.clone(), "POST", &path, body)
        .await
        .0
        .is_success());
    let ledger = request(
        app,
        "POST",
        "/api/v1/activities/search",
        serde_json::json!({"page":0,"pageSize":20}),
    )
    .await
    .1;
    assert_eq!(ledger["meta"]["totalRowCount"], 2);
    let rows = ledger["data"].as_array().unwrap();
    let refund = rows.iter().find(|a| a["activityType"] == "CREDIT").unwrap();
    assert_eq!(
        refund["metadata"]["refund"]["original_activity_id"],
        original_id
    );
    assert_eq!(refund["amount"], "40");
    assert_eq!(
        rows.iter().find(|a| a["id"] == original_id).unwrap()["amount"],
        "120"
    );
}

#[tokio::test]
async fn review_pages_filter_unresolved_items_without_repeating_captures() {
    let _guard = ENV_LOCK.lock().await;
    let dir = tempdir().unwrap();
    std::env::set_var("WF_DB_PATH", dir.path().join("app.db"));
    std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
    let config = test_config(
        dir.path().join("app.db").display().to_string(),
        dir.path().join("addons").display().to_string(),
    );
    let app = app_router(build_state(&config).await.unwrap(), &config);
    for index in 0..3 {
        assert!(request(app.clone(),"POST","/api/v1/captures",serde_json::json!({"clientRequestId":format!("page-{index}"),"text":"Unconfigured synthetic capture","inputKind":"unknown"})).await.0.is_success());
    }
    let first = request(
        app.clone(),
        "GET",
        "/api/v1/capture-reviews?page=0&pageSize=2&reason=configuration_required",
        serde_json::Value::Null,
    )
    .await
    .1;
    let second = request(
        app.clone(),
        "GET",
        "/api/v1/capture-reviews?page=1&pageSize=2&reason=configuration_required",
        serde_json::Value::Null,
    )
    .await
    .1;
    assert_eq!(first.as_array().unwrap().len(), 2);
    assert_eq!(second.as_array().unwrap().len(), 1);
    assert!(!first
        .as_array()
        .unwrap()
        .iter()
        .any(|c| c["id"] == second[0]["id"]));
    assert_eq!(
        request(
            app.clone(),
            "GET",
            "/api/v1/capture-reviews?reason=category_required",
            serde_json::Value::Null
        )
        .await
        .1,
        serde_json::json!([])
    );
    assert_eq!(
        request(
            app,
            "GET",
            "/api/v1/capture-reviews?pageSize=1000",
            serde_json::Value::Null
        )
        .await
        .0,
        axum::http::StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn stale_extraction_qualification_cannot_enable_posting() {
    let _guard = ENV_LOCK.lock().await;
    let dir = tempdir().unwrap();
    std::env::set_var("WF_DB_PATH", dir.path().join("app.db"));
    std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
    let config = test_config(
        dir.path().join("app.db").display().to_string(),
        dir.path().join("addons").display().to_string(),
    );
    let app = app_router(build_state(&config).await.unwrap(), &config);
    let result=request(app,"PUT","/api/v1/quick-add/settings",serde_json::json!({"provider":"bedrock","model":"openai.gpt-5.6-luna","monthlyBudgetMicros":1000000,"timezone":"Asia/Kolkata","automaticPosting":true,"evaluationPassed":true,"supervisedTrialCompleted":true,"extractionVersion":"old-version"})).await;
    assert_eq!(result.0, axum::http::StatusCode::CONFLICT);
}

/// Opt-in only. Uses a temporary ledger and synthetic alerts, never production data.
#[tokio::test]
#[ignore = "requires an explicitly supplied local Bedrock key file and network access"]
async fn bedrock_luna_qualification() {
    let _guard = ENV_LOCK.lock().await;
    let key_path = std::env::var("QUICK_ADD_BEDROCK_KEY_FILE").expect("Set the key file path");
    let key = std::fs::read_to_string(key_path).expect("Read local key file");
    let dir = tempdir().unwrap();
    std::env::set_var("WF_DB_PATH", dir.path().join("app.db"));
    std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
    let config = test_config(
        dir.path().join("app.db").display().to_string(),
        dir.path().join("addons").display().to_string(),
    );
    let state = build_state(&config).await.unwrap();
    state
        .secret_store
        .set_secret("ai_bedrock", key.trim())
        .unwrap();
    drop(key);
    let app = app_router(state, &config);
    let provider = request(app.clone(), "PUT", "/api/v1/ai/providers/settings", serde_json::json!({"providerId":"bedrock","enabled":true,"customUrl":"https://bedrock-mantle.us-east-1.api.aws/v1"})).await;
    assert!(provider.0.is_success());
    let account = request(app.clone(), "POST", "/api/v1/accounts", serde_json::json!({"name":"Synthetic qualification bank","accountType":"CASH","currency":"INR","isDefault":false,"isActive":true,"trackingMode":"TRANSACTIONS"})).await.1;
    let configured = request(app.clone(), "PUT", "/api/v1/quick-add/settings", serde_json::json!({"provider":"bedrock","model":"openai.gpt-5.6-luna","monthlyBudgetMicros":250000,"automaticPosting":true,"evaluationPassed":true,"extractionVersion":"bedrock-luna-capture-v3","supervisedTrialCompleted":true,"timezone":"Asia/Kolkata","mappings":[{"alias":"1234","accountId":account["id"]}]})).await;
    assert!(configured.0.is_success());
    let default_cases = [
        ("debit", "INR 120 debited from account 1234 on 2026-09-10 at Test cafe. Ref 000101.", Some(("120", "WITHDRAWAL", "000101"))),
        ("credit", "INR 250 credited to account 1234 on 2026-09-10. Ref 000102.", Some(("250", "DEPOSIT", "000102"))),
        ("balance", "INR 75 debited from account 1234 on 2026-09-10 at Test shop. Ref 000103. Available balance INR 99000.", Some(("75", "WITHDRAWAL", "000103"))),
        ("unknown-account", "INR 120 debited from account 9876 on 2026-09-10. Ref 000104.", None),
        ("declined", "INR 120 payment declined on account 1234 on 2026-09-10. Ref 000105. No debit occurred.", None),
        ("pending", "INR 120 payment from account 1234 is pending on 2026-09-10. Ref 000106.", None),
        ("otp", "OTP 123456 for INR 120 payment using account 1234. Do not share. This is not a payment confirmation.", None),
        ("missing-date", "INR 120 debited from account 1234 at Test cafe. Ref 000108.", None),
        ("ambiguous-dates", "INR 120 debited from account 1234 on 2026-09-10 or 2026-09-11. Ref 000109.", None),
        ("transfer", "INR 500 transferred from account 1234 to my other bank on 2026-09-10. Ref 000110.", None),
        ("typed-uncertain", "Lunch yesterday, maybe 200 or 250. Add it somewhere.", None),
        ("duplicate-reference", "INR 120 debited from account 1234 on 2026-09-10 at Test cafe. Ref 000101.", Some(("120", "WITHDRAWAL", "000101"))),
    ];
    let corpus: Option<serde_json::Value> = std::env::var("QUICK_ADD_QUALIFICATION_CORPUS")
        .ok()
        .map(|path| serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap());
    let cases: Vec<(&str, &str, Option<(&str, &str, &str)>)> = if let Some(corpus) = corpus.as_ref()
    {
        corpus["cases"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| {
                (
                    c["name"].as_str().unwrap(),
                    c["text"].as_str().unwrap(),
                    if c["expected"].is_null() {
                        None
                    } else {
                        Some((
                            c["expected"]["amount"].as_str().unwrap(),
                            c["expected"]["activityType"].as_str().unwrap(),
                            c["expected"]["reference"].as_str().unwrap(),
                        ))
                    },
                )
            })
            .collect()
    } else {
        default_cases.into_iter().collect()
    };
    let expected_ledger_count = corpus
        .as_ref()
        .and_then(|c| c["expectedLedgerCount"].as_u64())
        .unwrap_or(3);
    let mut results = Vec::new();
    let mut input_tokens = 0u64;
    let mut output_tokens = 0u64;
    for (index, (name, text, expected)) in cases.iter().enumerate() {
        let started = std::time::Instant::now();
        let saved = request(app.clone(), "POST", "/api/v1/captures", serde_json::json!({"clientRequestId":format!("qualification-{index}"),"text":text,"inputKind":if name.starts_with("typed") {"typed_note"} else if name.starts_with("card") {"card_alert"} else {"bank_alert"}})).await;
        assert!(saved.0.is_success());
        let path = format!("/api/v1/captures/{}", saved.1["id"].as_str().unwrap());
        let receipt = tokio::time::timeout(Duration::from_secs(65), async {
            loop {
                let receipt = request(app.clone(), "GET", &path, serde_json::Value::Null)
                    .await
                    .1;
                if receipt["status"] != "processing" {
                    break receipt;
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        })
        .await
        .expect("Capture must finish within its bounded extraction timeout");
        input_tokens += receipt["usage"]["inputTokens"].as_u64().unwrap_or(0);
        output_tokens += receipt["usage"]["outputTokens"].as_u64().unwrap_or(0);
        let posted: Vec<_> = receipt["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|c| c["status"] == "posted" || c["status"] == "already_recorded")
            .collect();
        let passed = match expected {
            Some((amount, activity_type, reference)) => {
                posted.len() == 1
                    && posted[0]["fields"]["accountId"] == account["id"]
                    && posted[0]["fields"]["amount"]
                        .as_str()
                        .and_then(|v| v.parse::<rust_decimal::Decimal>().ok())
                        == amount.parse::<rust_decimal::Decimal>().ok()
                    && posted[0]["fields"]["date"] == "2026-09-10"
                    && posted[0]["fields"]["currency"] == "INR"
                    && posted[0]["fields"]["direction"]
                        == if *activity_type == "DEPOSIT" {
                            "credit"
                        } else {
                            "debit"
                        }
                    && posted[0]["fields"]["reference"] == *reference
            }
            None => {
                posted.is_empty()
                    && receipt["reviews"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|r| r["status"] == "open")
            }
        };
        results.push(serde_json::json!({"case":name,"passed":passed,"latencyMs":started.elapsed().as_millis(),"postedCandidates":posted.len(),"reviewReasons":receipt["reviews"].as_array().unwrap().iter().map(|r|r["reason"].clone()).collect::<Vec<_>>(),"syntheticCandidates":receipt["candidates"]}));
    }
    let ledger = request(
        app,
        "POST",
        "/api/v1/activities/search",
        serde_json::json!({"page":0,"pageSize":100}),
    )
    .await
    .1;
    let ledger_count = ledger["meta"]["totalRowCount"].as_u64().unwrap_or(0);
    let passed =
        results.iter().all(|r| r["passed"] == true) && ledger_count == expected_ledger_count;
    let report = serde_json::json!({"model":"openai.gpt-5.6-luna","extractionVersion":wealthfolio_core::captures::extraction::EXTRACTION_VERSION,"provider":"bedrock","region":"us-east-1","passed":passed,"cases":results,"ledgerCount":ledger_count,"inputTokens":input_tokens,"outputTokens":output_tokens,"costMicros":(input_tokens+6*output_tokens).saturating_mul(11).div_ceil(50),"scope":"Synthetic API qualification only; supervised trial and iPhone acceptance remain pending"});
    if let Ok(path) = std::env::var("QUICK_ADD_QUALIFICATION_REPORT") {
        std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
    }
    assert!(
        passed,
        "Synthetic qualification did not pass; inspect the redacted qualification report"
    );
}

#[tokio::test]
async fn pending_alerts_match_completed_payments_in_either_arrival_order() {
    let _guard = ENV_LOCK.lock().await;
    for completed_first in [false, true] {
        let dir = tempdir().unwrap();
        std::env::set_var("WF_DB_PATH", dir.path().join("app.db"));
        std::env::set_var("WF_SECRET_FILE", dir.path().join("secrets.json"));
        let alert = "INR 120 debited from account 1234 on 2026-09-10 at Test cafe. Ref 001234. Completion pending.";
        let extraction = serde_json::json!({"events":[{"accountHint":"1234","amount":"120","currency":"INR","date":"2026-09-10","direction":"debit","merchant":"Test cafe","reference":"001234","kind":"payment","state":"pending","sourceText":alert}]});
        let model = axum::Router::new().route("/v1/chat/completions", axum::routing::post(move |axum::Json(body): axum::Json<serde_json::Value>| {
        let extraction = extraction.clone(); async move {
            assert_eq!(body["model"], "openai.gpt-5.6-luna");
            assert_eq!(body["response_format"]["json_schema"]["strict"], true);
            assert!(body.get("tools").is_none());
            axum::Json(serde_json::json!({"choices":[{"finish_reason":"stop","message":{"content":extraction.to_string()}}],"usage":{"prompt_tokens":500,"completion_tokens":120}}))
        }
    }));
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}/v1", listener.local_addr().unwrap());
        let model_task = tokio::spawn(async move { axum::serve(listener, model).await.unwrap() });
        let config = test_config(
            dir.path().join("app.db").display().to_string(),
            dir.path().join("addons").display().to_string(),
        );
        let state = build_state(&config).await.unwrap();
        state
            .secret_store
            .set_secret("ai_bedrock", "synthetic-key")
            .unwrap();
        let app = app_router(state, &config);
        let provider = request(
            app.clone(),
            "PUT",
            "/api/v1/ai/providers/settings",
            serde_json::json!({"providerId":"bedrock","enabled":true,"customUrl":endpoint}),
        )
        .await;
        assert!(provider.0.is_success(), "{:?}", provider);
        let account = request(app.clone(), "POST", "/api/v1/accounts", serde_json::json!({"name":"Test bank","accountType":"CASH","currency":"INR","isDefault":false,"isActive":true,"trackingMode":"TRANSACTIONS"})).await.1;
        let settings = request(app.clone(), "PUT", "/api/v1/quick-add/settings", serde_json::json!({"provider":"bedrock","model":"openai.gpt-5.6-luna","monthlyBudgetMicros":1000000,"timezone":"Asia/Kolkata","mappings":[{"alias":"1234","accountId":account["id"]}]})).await;
        assert!(settings.0.is_success(), "{:?}", settings);

        let completed_body = serde_json::json!({"accountId":account["id"],"activityType":"WITHDRAWAL","activityDate":"2026-09-10T12:00:00Z","amount":"120","currency":"INR","sourceRecordId":"001234","sourceSystem":"CSV"});
        if completed_first {
            assert!(request(
                app.clone(),
                "POST",
                "/api/v1/activities",
                completed_body.clone()
            )
            .await
            .0
            .is_success());
        }
        let receipt = capture(app.clone(), alert).await;
        assert!(receipt.0.is_success());
        let id = receipt.1["id"].as_str().unwrap();
        let extracted = settled(app.clone(), id).await;
        if !completed_first {
            assert_eq!(extracted["candidates"][0]["status"], "needs_review");
            let ledger = request(
                app.clone(),
                "POST",
                "/api/v1/activities/search",
                serde_json::json!({"page":0,"pageSize":20}),
            )
            .await
            .1;
            assert_eq!(ledger["meta"]["totalRowCount"], 0);
            assert!(
                request(app.clone(), "POST", "/api/v1/activities", completed_body)
                    .await
                    .0
                    .is_success()
            );
        }
        let reconciled = tokio::time::timeout(Duration::from_secs(15), async {
            loop {
                let receipt = request(
                    app.clone(),
                    "GET",
                    &format!("/api/v1/captures/{id}"),
                    serde_json::Value::Null,
                )
                .await
                .1;
                if receipt["status"] == "complete" {
                    break receipt;
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        })
        .await
        .unwrap();
        assert_eq!(reconciled["candidates"][0]["status"], "already_recorded");
        assert_eq!(reconciled["candidates"][0]["sourceState"], "pending");
        let ledger = request(
            app.clone(),
            "POST",
            "/api/v1/activities/search",
            serde_json::json!({"page":0,"pageSize":20}),
        )
        .await
        .1;
        assert_eq!(ledger["meta"]["totalRowCount"], 1);
        model_task.abort();
    }
}
