use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use wealthfolio_core::secrets::SecretStore;
use wealthfolio_native::{Command, Engine, Reply};

#[derive(Default)]
struct TestSecrets(Mutex<HashMap<String, String>>);
impl SecretStore for TestSecrets {
    fn get_secret(&self, key: &str) -> wealthfolio_core::Result<Option<String>> {
        Ok(self.0.lock().unwrap().get(key).cloned())
    }
    fn set_secret(&self, key: &str, value: &str) -> wealthfolio_core::Result<()> {
        self.0.lock().unwrap().insert(key.into(), value.into());
        Ok(())
    }
    fn delete_secret(&self, key: &str) -> wealthfolio_core::Result<()> {
        self.0.lock().unwrap().remove(key);
        Ok(())
    }
}
async fn request(engine: &Engine, method: &str, path: &str, body: Value) -> Reply {
    engine
        .request(Command {
            method: method.into(),
            path: path.into(),
            body,
        })
        .await
        .unwrap()
}
#[tokio::test]
async fn native_client_uses_the_existing_offline_services_and_persists_edits() {
    let directory = tempfile::tempdir().unwrap();
    let secrets = Arc::new(TestSecrets::default());
    let engine = Engine::open(directory.path().to_str().unwrap(), secrets.clone())
        .await
        .unwrap();
    let account = request(
        &engine,
        "POST",
        "/api/v1/accounts",
        json!({
            "name":"Native contract account", "accountType":"CASH", "currency":"USD",
            "isDefault":true, "isActive":true, "trackingMode":"TRANSACTIONS"
        }),
    )
    .await;
    assert_eq!(account.status, 200, "{}", account.body);
    let account_id = account.body["id"].as_str().unwrap();
    let created = request(
        &engine,
        "POST",
        "/api/v1/activities",
        json!({
            "accountId":account_id, "activityType":"DEPOSIT", "activityDate":"2026-01-15T12:00:00Z",
            "currency":"USD", "amount":"123.45", "sourceSystem":"MANUAL"
        }),
    )
    .await;
    assert_eq!(created.status, 200, "{}", created.body);
    let invalid = request(
        &engine,
        "POST",
        "/api/v1/activities",
        json!({
            "accountId":account_id,"activityType":"NOT_A_TRANSACTION","activityDate":"invalid",
            "currency":"USD","amount":"123.45"
        }),
    )
    .await;
    assert!(invalid.status >= 400);
    for (method, path, body) in [
        ("GET", "/api/v1/settings", Value::Null),
        ("GET", "/api/v1/accounts", Value::Null),
        ("GET", "/native/sync/status", Value::Null),
        ("GET", "/api/v1/goals", Value::Null),
        ("GET", "/api/v1/net-worth", Value::Null),
        (
            "POST",
            "/api/v1/allocations/query",
            json!({"filter":{"type":"all"}}),
        ),
        (
            "POST",
            "/api/v1/performance/history",
            json!({"itemType":"account","itemId":"TOTAL","filter":{"type":"all"}}),
        ),
        (
            "GET",
            "/api/v1/utilities/export/activities/csv",
            Value::Null,
        ),
        (
            "POST",
            "/api/v1/holdings/list/query",
            json!({"filter":{"type":"all"}}),
        ),
        (
            "POST",
            "/api/v1/valuations/current/query",
            json!({"filter":{"type":"all"},"includeAccounts":true}),
        ),
        (
            "POST",
            "/api/v1/valuations/history/query",
            json!({"filter":{"type":"all"}}),
        ),
    ] {
        let result = request(&engine, method, path, body).await;
        assert_eq!(result.status, 200, "{path}: {}", result.body);
    }
    let reopened = Engine::open(directory.path().to_str().unwrap(), secrets)
        .await
        .unwrap();
    let found = request(
        &reopened,
        "POST",
        "/api/v1/activities/search",
        json!({"page":0,"pageSize":50}),
    )
    .await;
    assert_eq!(found.status, 200, "{}", found.body);
    assert_eq!(found.body["data"].as_array().unwrap().len(), 1);
    assert_eq!(found.body["data"][0]["amount"], "123.45");
    assert!(!directory.path().join("secrets.json").exists());
}
