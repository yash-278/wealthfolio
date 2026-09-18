//! Owner-operated sync, independent of Wealthfolio Connect.
//! The journal and domain mutation commit together on the existing SQLite writer.
use super::*;
use diesel::sql_types::{BigInt, Text};
use serde::{Deserialize, Serialize};
use wealthfolio_core::errors::ValidationError;

#[derive(Debug, Clone, Serialize, Deserialize, QueryableByName)]
#[serde(rename_all = "camelCase")]
pub struct ServerSyncHead {
    #[diesel(sql_type = Text)]
    pub server_id: String,
    #[diesel(sql_type = BigInt)]
    pub cursor: i64,
    #[diesel(sql_type = BigInt)]
    pub enabled: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServerSyncPush {
    pub server_id: String,
    pub event_id: String,
    pub entity: SyncEntity,
    pub entity_id: String,
    pub op: SyncOperation,
    /// Revision seen by the client; null only for an entity with no revision.
    pub base_event_id: Option<String>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerSyncChange {
    pub seq: i64,
    pub event_id: String,
    pub entity: SyncEntity,
    pub entity_id: String,
    pub op: SyncOperation,
    pub timestamp: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerSyncPage {
    pub server_id: String,
    pub cursor: i64,
    pub has_more: bool,
    pub changes: Vec<ServerSyncChange>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ServerSyncPushResult {
    Applied {
        seq: i64,
    },
    Duplicate {
        seq: i64,
    },
    Conflict {
        #[serde(rename = "currentEventId")]
        current_event_id: Option<String>,
    },
}

#[derive(QueryableByName)]
struct JournalRow {
    #[diesel(sql_type = BigInt)]
    seq: i64,
    #[diesel(sql_type = Text)]
    event_id: String,
    #[diesel(sql_type = Text)]
    entity: String,
    #[diesel(sql_type = Text)]
    entity_id: String,
    #[diesel(sql_type = Text)]
    op: String,
    #[diesel(sql_type = Text)]
    timestamp: String,
    #[diesel(sql_type = Text)]
    payload: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<Text>)]
    request_json: Option<String>,
}

fn invalid(message: &str) -> Error {
    Error::Validation(ValidationError::InvalidInput(message.into()))
}

pub(super) fn head(conn: &mut SqliteConnection) -> Result<ServerSyncHead> {
    diesel::sql_query("SELECT server_id, CAST(enabled AS BIGINT) AS enabled, COALESCE((SELECT MAX(seq) FROM server_sync_events), 0) AS cursor FROM server_sync_state WHERE id = 1")
        .get_result(conn).map_err(|e| StorageError::from(e).into())
}

#[allow(clippy::too_many_arguments)]
pub(super) fn append_event(
    conn: &mut SqliteConnection,
    event_id: &str,
    entity: SyncEntity,
    entity_id: &str,
    op: SyncOperation,
    timestamp: &str,
    payload: &serde_json::Value,
) -> Result<()> {
    diesel::sql_query("INSERT INTO server_sync_events (event_id, entity, entity_id, op, timestamp, payload) SELECT ?, ?, ?, ?, ?, ? WHERE (SELECT enabled FROM server_sync_state WHERE id = 1) = 1")
        .bind::<Text, _>(event_id).bind::<Text, _>(enum_to_db(&entity)?)
        .bind::<Text, _>(entity_id).bind::<Text, _>(enum_to_db(&op)?)
        .bind::<Text, _>(timestamp).bind::<Text, _>(serde_json::to_string(payload)?)
        .execute(conn).map_err(StorageError::from)?;
    diesel::sql_query("INSERT INTO server_sync_revisions (entity, entity_id, last_event_id) SELECT ?, ?, ? WHERE (SELECT enabled FROM server_sync_state WHERE id = 1) = 1 ON CONFLICT(entity, entity_id) DO UPDATE SET last_event_id = excluded.last_event_id")
        .bind::<Text, _>(enum_to_db(&entity)?).bind::<Text, _>(entity_id).bind::<Text, _>(event_id)
        .execute(conn).map_err(StorageError::from)?;
    Ok(())
}

#[derive(QueryableByName)]
struct RevisionCount {
    #[diesel(sql_type = BigInt)]
    count: i64,
}

impl AppSyncRepository {
    /// Older builds omitted posted Quick Add rows from both snapshots and the journal.
    /// Publish only records without a server revision so paired devices recover on
    /// their next pull. Ledger rows and existing revisions are never overwritten.
    pub async fn repair_quick_add_sync(&self) -> Result<usize> {
        self.writer.exec(|conn| {
            use crate::activities::ActivityDB;
            use crate::schema::{activities, activity_taxonomy_assignments, spending_activity_events, spending_activity_splits};
            use crate::spending::{activity_assignments::ActivityTaxonomyAssignmentDB,
                activity_events::ActivityEventDB, activity_splits::ActivitySplitDB};
            use crate::sync::outbox_request_for_model;
            if head(conn)?.enabled != 1 { return Ok(0); }
            let rows = activities::table
                .filter(diesel::dsl::sql::<diesel::sql_types::Bool>("UPPER(TRIM(source_system)) = 'QUICK_ADD'"))
                .select(ActivityDB::as_select()).load::<ActivityDB>(conn).map_err(StorageError::from)?;
            let ids: Vec<_> = rows.iter().map(|row| row.id.clone()).collect();
            let mut requests = Vec::new();
            for row in rows { requests.push(outbox_request_for_model(&row, SyncOperation::Create)?); }
            for row in activity_taxonomy_assignments::table.filter(activity_taxonomy_assignments::activity_id.eq_any(&ids))
                .select(ActivityTaxonomyAssignmentDB::as_select()).load::<ActivityTaxonomyAssignmentDB>(conn).map_err(StorageError::from)? {
                requests.push(outbox_request_for_model(&row, SyncOperation::Create)?);
            }
            for row in spending_activity_splits::table.filter(spending_activity_splits::activity_id.eq_any(&ids))
                .select(ActivitySplitDB::as_select()).load::<ActivitySplitDB>(conn).map_err(StorageError::from)? {
                requests.push(outbox_request_for_model(&row, SyncOperation::Create)?);
            }
            for row in spending_activity_events::table.filter(spending_activity_events::activity_id.eq_any(&ids))
                .select(ActivityEventDB::as_select()).load::<ActivityEventDB>(conn).map_err(StorageError::from)? {
                requests.push(outbox_request_for_model(&row, SyncOperation::Create)?);
            }
            let mut repaired = 0;
            for request in requests {
                let existing = diesel::sql_query("SELECT COUNT(*) AS count FROM server_sync_revisions WHERE entity = ? AND entity_id = ?")
                    .bind::<Text,_>(enum_to_db(&request.entity)?).bind::<Text,_>(&request.entity_id)
                    .get_result::<RevisionCount>(conn).map_err(StorageError::from)?;
                if existing.count == 0 {
                    insert_outbox_event(conn, request)?;
                    repaired += 1;
                }
            }
            Ok(repaired)
        }).await
    }

    /// Explicit opt-in. Existing data is supplied by the initial snapshot.
    pub async fn enable_server_sync(&self) -> Result<ServerSyncHead> {
        self.writer
            .exec(|conn| {
                if sync_device_config::table.count().get_result::<i64>(conn).map_err(StorageError::from)? > 0 {
                    return Err(invalid("Disconnect Wealthfolio Connect device sync before enabling server sync"));
                }
                diesel::sql_query("INSERT INTO server_sync_revisions (entity, entity_id, last_event_id) SELECT entity, entity_id, last_event_id FROM sync_entity_metadata WHERE (SELECT enabled FROM server_sync_state WHERE id = 1) = 0 ON CONFLICT(entity, entity_id) DO NOTHING")
                .execute(conn).map_err(StorageError::from)?;
            diesel::sql_query("UPDATE server_sync_state SET enabled = 1 WHERE id = 1")
                    .execute(conn)
                    .map_err(StorageError::from)?;
                head(conn)
            })
            .await
    }

    pub fn server_sync_head(&self) -> Result<ServerSyncHead> {
        let mut conn = get_connection(&self.pool)?;
        head(&mut conn)
    }

    pub fn pull_server_sync(
        &self,
        server_id: &str,
        cursor: i64,
        limit: i64,
    ) -> Result<ServerSyncPage> {
        if cursor < 0 || !(1..=500).contains(&limit) {
            return Err(invalid("Invalid sync cursor or page size"));
        }
        let mut conn = get_connection(&self.pool)?;
        conn.transaction::<_, StorageError, _>(|conn| {
            let current = head(conn)?;
            if current.enabled != 1 || current.server_id != server_id || cursor > current.cursor {
                return Err(invalid(
                    "Sync server changed, is disabled, or cursor is ahead; bootstrap required",
                )
                .into());
            }
            let rows = diesel::sql_query(
                "SELECT * FROM server_sync_events WHERE seq > ? ORDER BY seq LIMIT ?",
            )
            .bind::<BigInt, _>(cursor)
            .bind::<BigInt, _>(limit + 1)
            .load::<JournalRow>(conn)
            .map_err(StorageError::from)?;
            let has_more = rows.len() > limit as usize;
            let changes = rows
                .into_iter()
                .take(limit as usize)
                .map(|row| -> Result<_> {
                    Ok(ServerSyncChange {
                        seq: row.seq,
                        event_id: row.event_id,
                        entity: enum_from_db(&row.entity)?,
                        entity_id: row.entity_id,
                        op: enum_from_db(&row.op)?,
                        timestamp: row.timestamp,
                        payload: serde_json::from_str(&row.payload)?,
                    })
                })
                .collect::<Result<Vec<_>>>()?;
            Ok(ServerSyncPage {
                server_id: current.server_id,
                cursor: changes.last().map_or(cursor, |change| change.seq),
                has_more,
                changes,
            })
        })
        .map_err(Error::from)
    }

    /// One mutation per request. Clients upload dependent changes in order and stop on conflict.
    pub async fn push_server_sync(&self, request: ServerSyncPush) -> Result<ServerSyncPushResult> {
        if Uuid::parse_str(&request.event_id).is_err()
            || request.entity_id.is_empty()
            || request.entity_id.len() > 256
            || !request
                .entity_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b':' | b'-'))
        {
            return Err(invalid("Invalid sync event or entity identifier"));
        }
        let (table, pk) = entity_storage_mapping(&request.entity)
            .ok_or_else(|| invalid("This entity does not yet support server sync uploads"))?;
        let request_json = serde_json::to_string(&request)?;
        if request_json.len() > 1_000_000 || !request.payload.is_object() {
            return Err(invalid("Sync payload must be an object smaller than 1 MB"));
        }
        self.writer
            .exec(move |conn| {
                let current = head(conn)?;
                if current.enabled != 1 || current.server_id != request.server_id {
                    return Err(invalid(
                        "Sync server changed or is disabled; bootstrap required",
                    ));
                }
                let previous =
                    diesel::sql_query("SELECT * FROM server_sync_events WHERE event_id = ?")
                        .bind::<Text, _>(&request.event_id)
                        .get_result::<JournalRow>(conn)
                        .optional()
                        .map_err(StorageError::from)?;
                if let Some(previous) = previous {
                    if previous.request_json.as_deref() != Some(&request_json) {
                        return Err(invalid(
                            "Sync event ID cannot be reused with different content",
                        ));
                    }
                    return Ok(ServerSyncPushResult::Duplicate { seq: previous.seq });
                }
                #[derive(QueryableByName)]
            struct Revision { #[diesel(sql_type = Text)] last_event_id: String }
            let metadata = diesel::sql_query("SELECT last_event_id FROM server_sync_revisions WHERE entity = ? AND entity_id = ?")
                .bind::<Text, _>(enum_to_db(&request.entity)?).bind::<Text, _>(&request.entity_id)
                .get_result::<Revision>(conn).optional().map_err(StorageError::from)?;
            let current_event_id = metadata.as_ref().map(|m| m.last_event_id.clone());
            if current_event_id != request.base_event_id {
                    return Ok(ServerSyncPushResult::Conflict { current_event_id });
                }
                #[derive(QueryableByName)]
                struct Exists {
                    #[diesel(sql_type = BigInt)]
                    present: i64,
                }
                let exists = diesel::sql_query(format!(
                    "SELECT EXISTS(SELECT 1 FROM {} WHERE {} = ?) AS present",
                    quote_identifier(table),
                    quote_identifier(pk)
                ))
                .bind::<Text, _>(&request.entity_id)
                .get_result::<Exists>(conn)
                .map_err(StorageError::from)?
                .present
                    != 0;
                if (request.op == SyncOperation::Create && (exists || metadata.is_some()))
                    || (request.op != SyncOperation::Create && !exists)
                {
                    return Ok(ServerSyncPushResult::Conflict { current_event_id });
                }
                // The server revision check above decides ordering, never the phone's clock.
                let applied = apply_remote_event_tx(
                    conn,
                    request.entity,
                    request.entity_id.clone(),
                    request.op,
                    request.event_id.clone(),
                    Utc::now().to_rfc3339(),
                    0,
                    request.payload.clone(),
                    false,
                )?;
                if !applied {
                    return Err(invalid("Sync mutation was not applied"));
                }
                diesel::sql_query(
                    "UPDATE server_sync_events SET request_json = ? WHERE event_id = ?",
                )
                .bind::<Text, _>(&request_json)
                .bind::<Text, _>(&request.event_id)
                .execute(conn)
                .map_err(StorageError::from)?;
                let row = diesel::sql_query("SELECT * FROM server_sync_events WHERE event_id = ?")
                    .bind::<Text, _>(&request.event_id)
                    .get_result::<JournalRow>(conn)
                    .map_err(StorageError::from)?;
                Ok(ServerSyncPushResult::Applied { seq: row.seq })
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{create_pool, run_migrations, write_actor::spawn_writer};
    use serde_json::json;

    fn setup() -> (tempfile::TempDir, AppSyncRepository) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("sync.db");
        run_migrations(path.to_str().unwrap()).unwrap();
        let pool = create_pool(path.to_str().unwrap()).unwrap();
        let writer = spawn_writer(pool.as_ref().clone()).unwrap();
        (dir, AppSyncRepository::new(pool, writer))
    }

    fn create(server: &str, id: &str) -> ServerSyncPush {
        ServerSyncPush {
            server_id: server.into(),
            event_id: Uuid::now_v7().to_string(),
            entity: SyncEntity::Goal,
            entity_id: id.into(),
            op: SyncOperation::Create,
            base_event_id: None,
            payload: json!({"id":id,"title":"Test goal","target_amount":100}),
        }
    }

    #[tokio::test]
    async fn quick_add_repair_reaches_existing_clients_once_without_changing_ledger() {
        let (dir, server) = setup();
        server.writer.exec(|conn| {
            diesel::sql_query("INSERT INTO accounts (id,name,account_type,currency,is_default,is_active) VALUES ('cash','Synthetic cash','cash','USD',0,1)")
                .execute(conn).map_err(StorageError::from)?;
            Ok(())
        }).await.unwrap();
        let head = server.enable_server_sync().await.unwrap();
        let (snapshot, snapshot_head) = server.export_server_sync_snapshot().await.unwrap();
        let path = dir.path().join("initial.db");
        std::fs::write(&path, snapshot).unwrap();
        let (_phone_dir, phone) = setup();
        phone
            .bootstrap_server_client(
                path.to_string_lossy().into_owned(),
                "https://example.com".into(),
                snapshot_head.server_id,
                snapshot_head.cursor,
            )
            .await
            .unwrap();
        // Reproduce the old build's ledger write with no corresponding sync event.
        server.writer.exec(|conn| {
            diesel::sql_query("INSERT INTO activities (id,account_id,activity_type,status,activity_date,amount,currency,source_system,source_record_id,needs_review,created_at,updated_at) VALUES ('missed','cash','WITHDRAWAL','POSTED','2026-09-10T12:00:00Z','25','USD','QUICK_ADD','synthetic-reference',0,'2026-09-10T12:00:00Z','2026-09-10T12:00:00Z')")
                .execute(conn).map_err(StorageError::from)?;
            Ok(())
        }).await.unwrap();
        assert_eq!(server.server_sync_head().unwrap().cursor, head.cursor);
        assert_eq!(server.repair_quick_add_sync().await.unwrap(), 1);
        let repaired_head = server.server_sync_head().unwrap();
        assert_eq!(server.repair_quick_add_sync().await.unwrap(), 0);
        assert_eq!(
            server.server_sync_head().unwrap().cursor,
            repaired_head.cursor
        );
        let page = server
            .pull_server_sync(&head.server_id, head.cursor, 100)
            .unwrap();
        assert_eq!(page.changes.len(), 1);
        phone.apply_server_page(head.cursor, page).await.unwrap();
        use crate::activities::ActivityDB;
        let read = |repo: &AppSyncRepository| {
            let mut conn = get_connection(&repo.pool).unwrap();
            crate::schema::activities::table
                .find("missed")
                .select(ActivityDB::as_select())
                .first::<ActivityDB>(&mut conn)
                .unwrap()
        };
        assert_eq!(
            serde_json::to_value(read(&phone)).unwrap(),
            serde_json::to_value(read(&server)).unwrap()
        );
        assert_eq!(read(&server).amount.as_deref(), Some("25"));
    }

    #[tokio::test]
    async fn server_sync_retry_is_durable_and_identity_is_immutable() {
        let (_dir, repo) = setup();
        let head = repo.enable_server_sync().await.unwrap();
        let request = create(&head.server_id, "goal-a");
        assert!(matches!(
            repo.push_server_sync(request.clone()).await.unwrap(),
            ServerSyncPushResult::Applied { seq: 1 }
        ));
        let reopened = AppSyncRepository::new(repo.pool.clone(), repo.writer.clone());
        assert!(matches!(
            reopened.push_server_sync(request.clone()).await.unwrap(),
            ServerSyncPushResult::Duplicate { seq: 1 }
        ));
        let mut changed = request;
        changed.payload["title"] = json!("Different");
        assert!(reopened.push_server_sync(changed).await.is_err());
        assert_eq!(
            reopened
                .pull_server_sync(&head.server_id, 0, 100)
                .unwrap()
                .changes
                .len(),
            1
        );
    }

    #[tokio::test]
    async fn server_sync_concurrent_edits_conflict_and_delete_cannot_resurrect() {
        let (_dir, repo) = setup();
        let head = repo.enable_server_sync().await.unwrap();
        let first = create(&head.server_id, "goal-a");
        repo.push_server_sync(first.clone()).await.unwrap();
        let mut a = first.clone();
        a.event_id = Uuid::now_v7().to_string();
        a.op = SyncOperation::Update;
        a.base_event_id = Some(first.event_id.clone());
        a.payload["title"] = json!("Phone A");
        let mut b = a.clone();
        b.event_id = Uuid::now_v7().to_string();
        b.payload["title"] = json!("Phone B");
        let (a_result, b_result) = tokio::join!(
            repo.push_server_sync(a.clone()),
            repo.push_server_sync(b.clone())
        );
        let winner = match (a_result.unwrap(), b_result.unwrap()) {
            (ServerSyncPushResult::Applied { .. }, ServerSyncPushResult::Conflict { .. }) => a,
            (ServerSyncPushResult::Conflict { .. }, ServerSyncPushResult::Applied { .. }) => b,
            pair => panic!("Expected one accepted edit and one conflict: {pair:?}"),
        };
        let mut delete = winner.clone();
        delete.op = SyncOperation::Delete;
        delete.event_id = Uuid::now_v7().to_string();
        delete.base_event_id = Some(winner.event_id);
        delete.payload = json!({});
        repo.push_server_sync(delete.clone()).await.unwrap();
        let mut resurrect = create(&head.server_id, "goal-a");
        resurrect.base_event_id = Some(delete.event_id);
        assert!(matches!(
            repo.push_server_sync(resurrect).await.unwrap(),
            ServerSyncPushResult::Conflict { .. }
        ));
        assert_eq!(repo.server_sync_head().unwrap().cursor, 3);
    }

    #[tokio::test]
    async fn server_sync_local_edits_are_pulled_and_invalidate_phone_revision() {
        let (_dir, repo) = setup();
        let head = repo.enable_server_sync().await.unwrap();
        let first = create(&head.server_id, "goal-a");
        repo.push_server_sync(first.clone()).await.unwrap();
        let local_id = repo
            .writer
            .exec(|conn| {
                diesel::sql_query("UPDATE goals SET title = 'Web edit' WHERE id = 'goal-a'")
                    .execute(conn)
                    .map_err(StorageError::from)?;
                insert_outbox_event(
                    conn,
                    OutboxWriteRequest::new(
                        SyncEntity::Goal,
                        "goal-a",
                        SyncOperation::Update,
                        json!({"id":"goal-a","title":"Web edit","target_amount":100}),
                    ),
                )
            })
            .await
            .unwrap();
        let mut edit = first.clone();
        edit.event_id = Uuid::now_v7().to_string();
        edit.op = SyncOperation::Update;
        edit.base_event_id = Some(first.event_id);
        match repo.push_server_sync(edit).await.unwrap() {
            ServerSyncPushResult::Conflict { current_event_id } => {
                assert_eq!(current_event_id, Some(local_id))
            }
            other => panic!("Expected conflict: {other:?}"),
        }
        let page = repo.pull_server_sync(&head.server_id, 1, 1).unwrap();
        assert_eq!(page.changes[0].payload["title"], "Web edit");
    }

    #[tokio::test]
    async fn server_sync_pagination_identity_and_disabled_guards() {
        let (_dir, repo) = setup();
        let before = repo.server_sync_head().unwrap();
        assert!(repo
            .push_server_sync(create(&before.server_id, "disabled"))
            .await
            .is_err());
        let head = repo.enable_server_sync().await.unwrap();
        for id in ["a", "b", "c"] {
            repo.push_server_sync(create(&head.server_id, id))
                .await
                .unwrap();
        }
        let first = repo.pull_server_sync(&head.server_id, 0, 2).unwrap();
        assert!(first.has_more);
        assert_eq!(first.cursor, 2);
        let second = repo
            .pull_server_sync(&head.server_id, first.cursor, 2)
            .unwrap();
        assert!(!second.has_more);
        assert_eq!(second.cursor, 3);
        assert_eq!(second.changes.len(), 1);
        assert!(repo.pull_server_sync("wrong-server", 0, 2).is_err());
        assert!(repo.pull_server_sync(&head.server_id, 4, 2).is_err());
        assert!(repo.pull_server_sync(&head.server_id, -1, 2).is_err());
        assert!(repo.pull_server_sync(&head.server_id, 0, 501).is_err());
    }

    #[tokio::test]
    async fn server_sync_invalid_payload_rolls_back_mutation_and_journal() {
        let (_dir, repo) = setup();
        let head = repo.enable_server_sync().await.unwrap();
        let mut bad = create(&head.server_id, "a");
        bad.payload["id"] = json!("b");
        assert!(repo.push_server_sync(bad).await.is_err());
        assert_eq!(repo.server_sync_head().unwrap().cursor, 0);
        assert!(repo
            .get_entity_metadata(SyncEntity::Goal, "a")
            .unwrap()
            .is_none());
        repo.push_server_sync(create(&head.server_id, "a"))
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn server_sync_bootstrap_contains_data_revisions_and_matching_cursor_without_secrets() {
        let (_dir, repo) = setup();
        let head = repo.enable_server_sync().await.unwrap();
        let request = create(&head.server_id, "a");
        repo.push_server_sync(request.clone()).await.unwrap();
        let (bytes, manifest) = repo.export_server_sync_snapshot().await.unwrap();
        assert_eq!(manifest.cursor, 1);
        assert_eq!(manifest.server_id, head.server_id);
        let snapshot_dir = tempfile::tempdir().unwrap();
        let path = snapshot_dir.path().join("snapshot.db");
        std::fs::write(&path, bytes).unwrap();
        let snapshot = rusqlite::Connection::open(path).unwrap();
        assert_eq!(
            snapshot
                .query_row("SELECT title FROM goals WHERE id = 'a'", [], |r| r
                    .get::<_, String>(0))
                .unwrap(),
            "Test goal"
        );
        assert_eq!(
            snapshot
                .query_row(
                    "SELECT last_event_id FROM server_sync_revisions WHERE entity_id = 'a'",
                    [],
                    |r| r.get::<_, String>(0)
                )
                .unwrap(),
            request.event_id
        );
        for table in [
            "secrets",
            "capture_tokens",
            "server_sync_events",
            "sync_outbox",
        ] {
            assert_eq!(
                snapshot
                    .query_row(
                        "SELECT COUNT(*) FROM sqlite_master WHERE name = ?",
                        [table],
                        |r| r.get::<_, i64>(0)
                    )
                    .unwrap(),
                0
            );
        }
    }
    #[tokio::test]
    async fn server_sync_enable_is_idempotent_and_revisions_survive_connect_reset() {
        let (_dir, repo) = setup();
        let head = repo.enable_server_sync().await.unwrap();
        let initial = create(&head.server_id, "a");
        repo.push_server_sync(initial.clone()).await.unwrap();
        repo.reset_local_sync_session().await.unwrap();
        repo.enable_server_sync().await.unwrap();
        let mut stale = initial.clone();
        stale.event_id = Uuid::now_v7().to_string();
        stale.op = SyncOperation::Update;
        assert!(matches!(
            repo.push_server_sync(stale).await.unwrap(),
            ServerSyncPushResult::Conflict { .. }
        ));
        let mut valid = initial.clone();
        valid.event_id = Uuid::now_v7().to_string();
        valid.op = SyncOperation::Update;
        valid.base_event_id = Some(initial.event_id);
        assert!(matches!(
            repo.push_server_sync(valid).await.unwrap(),
            ServerSyncPushResult::Applied { .. }
        ));
        assert_eq!(repo.server_sync_head().unwrap().server_id, head.server_id);
    }

    #[tokio::test]
    async fn server_sync_rejects_enable_when_connect_device_is_registered() {
        let (_dir, repo) = setup();
        repo.writer.exec(|conn| {
            diesel::sql_query("INSERT INTO sync_device_config (device_id, trust_state) VALUES ('cloud-device', 'trusted')")
                .execute(conn).map_err(StorageError::from)?;
            Ok(())
        }).await.unwrap();
        assert!(repo.enable_server_sync().await.is_err());
        assert_eq!(repo.server_sync_head().unwrap().enabled, 0);
    }
    #[tokio::test]
    async fn server_sync_journal_failure_rolls_back_the_domain_write() {
        let (_dir, repo) = setup();
        let head = repo.enable_server_sync().await.unwrap();
        repo.writer.exec(|conn| {
            diesel::sql_query("CREATE TRIGGER fail_sync_journal BEFORE INSERT ON server_sync_events BEGIN SELECT RAISE(ABORT, 'injected journal failure'); END")
                .execute(conn).map_err(StorageError::from)?;
            Ok(())
        }).await.unwrap();
        let request = create(&head.server_id, "rollback-goal");
        assert!(repo.push_server_sync(request.clone()).await.is_err());
        let mut conn = get_connection(&repo.pool).unwrap();
        assert_eq!(
            crate::schema::goals::table
                .count()
                .get_result::<i64>(&mut conn)
                .unwrap(),
            0
        );
        assert_eq!(repo.server_sync_head().unwrap().cursor, 0);
        assert!(repo
            .get_entity_metadata(SyncEntity::Goal, "rollback-goal")
            .unwrap()
            .is_none());
        repo.writer
            .exec(|conn| {
                diesel::sql_query("DROP TRIGGER fail_sync_journal")
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await
            .unwrap();
        assert!(matches!(
            repo.push_server_sync(request).await.unwrap(),
            ServerSyncPushResult::Applied { .. }
        ));
    }

    #[tokio::test]
    async fn server_sync_enable_seeds_revisions_for_existing_local_data() {
        let (_dir, repo) = setup();
        let revision = repo.writer.exec(|conn| {
            diesel::sql_query("INSERT INTO goals (id, title, target_amount) VALUES ('existing', 'Existing goal', 100)")
                .execute(conn).map_err(StorageError::from)?;
            insert_outbox_event(conn, OutboxWriteRequest::new(SyncEntity::Goal, "existing", SyncOperation::Create,
                json!({"id":"existing","title":"Existing goal","target_amount":100})))
        }).await.unwrap();
        assert_eq!(repo.server_sync_head().unwrap().cursor, 0);
        let head = repo.enable_server_sync().await.unwrap();
        let mut request = create(&head.server_id, "existing");
        request.op = SyncOperation::Update;
        assert!(matches!(
            repo.push_server_sync(request.clone()).await.unwrap(),
            ServerSyncPushResult::Conflict { .. }
        ));
        request.base_event_id = Some(revision);
        assert!(matches!(
            repo.push_server_sync(request).await.unwrap(),
            ServerSyncPushResult::Applied { seq: 1 }
        ));
    }
}
