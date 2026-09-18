//! Durable state for a device connected to an owner-operated server.
use super::server_sync::{ServerSyncChange, ServerSyncPage, ServerSyncPush};
use super::*;
use diesel::sql_types::{BigInt, Bool, Nullable, Text};
use serde::{Deserialize, Serialize};
use wealthfolio_core::errors::ValidationError;

#[derive(Debug, Clone, Serialize, Deserialize, QueryableByName)]
#[serde(rename_all = "camelCase")]
pub struct ServerClientStatus {
    #[diesel(sql_type = Text)]
    pub endpoint: String,
    #[diesel(sql_type = Text)]
    pub server_id: String,
    #[diesel(sql_type = BigInt)]
    pub cursor: i64,
    #[diesel(sql_type = Bool)]
    pub paused: bool,
    #[diesel(sql_type = Nullable<Text>)]
    pub last_sync: Option<String>,
    #[diesel(sql_type = BigInt)]
    pub pending: i64,
    #[diesel(sql_type = BigInt)]
    pub conflicts: i64,
}

#[derive(Debug, Clone)]
pub struct QueuedServerChange {
    pub request: ServerSyncPush,
    pub conflict: Option<ServerSyncChange>,
}
#[derive(QueryableByName)]
struct QueueRow {
    #[diesel(sql_type = Text)]
    request_json: String,
    #[diesel(sql_type = Nullable<Text>)]
    conflict_json: Option<String>,
}
fn invalid(message: &str) -> Error {
    Error::Validation(ValidationError::InvalidInput(message.into()))
}
fn status(conn: &mut SqliteConnection) -> Result<Option<ServerClientStatus>> {
    diesel::sql_query("SELECT *, (SELECT COUNT(*) FROM server_sync_client_queue) AS pending, (SELECT COUNT(*) FROM server_sync_client_queue WHERE conflict_json IS NOT NULL) AS conflicts FROM server_sync_client WHERE id = 1")
        .get_result(conn).optional().map_err(|e| StorageError::from(e).into())
}

pub(super) fn is_paired(conn: &mut SqliteConnection) -> Result<bool> {
    Ok(status(conn)?.is_some())
}

pub(super) fn require_empty_client(conn: &mut SqliteConnection) -> Result<()> {
    if status(conn)?.is_some()
        || super::server_sync::head(conn)?.enabled == 1
        || sync_device_config::table
            .count()
            .get_result::<i64>(conn)
            .map_err(StorageError::from)?
            > 0
    {
        return Err(invalid("This installation is already configured for sync"));
    }
    for table in OVERWRITE_RISK_UNFILTERED_TABLES {
        let count = diesel::sql_query(format!(
            "SELECT COUNT(*) AS count FROM {}",
            quote_identifier(table)
        ))
        .get_result::<TableRowCountResult>(conn)
        .map_err(StorageError::from)?
        .count;
        if count > 0 {
            return Err(invalid(
                "Connect a fresh installation. Existing local records will not be overwritten",
            ));
        }
    }
    for spec in OVERWRITE_RISK_FILTERED_TABLES {
        let count = diesel::sql_query(format!(
            "SELECT COUNT(*) AS count FROM {} WHERE {}",
            quote_identifier(spec.table),
            spec.filter.sql()
        ))
        .get_result::<TableRowCountResult>(conn)
        .map_err(StorageError::from)?
        .count;
        if count > 0 {
            return Err(invalid(
                "Connect a fresh installation. Existing local records will not be overwritten",
            ));
        }
    }
    Ok(())
}

pub(super) fn finish_bootstrap(
    conn: &mut SqliteConnection,
    alias: &str,
    endpoint: &str,
    server_id: &str,
    cursor: i64,
) -> Result<()> {
    diesel::sql_query("DELETE FROM server_sync_revisions")
        .execute(conn)
        .map_err(StorageError::from)?;
    diesel::sql_query(format!("INSERT INTO server_sync_revisions SELECT entity, entity_id, last_event_id FROM {}.server_sync_revisions", quote_identifier(alias)))
        .execute(conn).map_err(StorageError::from)?;
    diesel::delete(sync_device_config::table)
        .execute(conn)
        .map_err(StorageError::from)?;
    diesel::sql_query(
        "INSERT INTO server_sync_client (id, endpoint, server_id, cursor) VALUES (1, ?, ?, ?)",
    )
    .bind::<Text, _>(endpoint)
    .bind::<Text, _>(server_id)
    .bind::<BigInt, _>(cursor)
    .execute(conn)
    .map_err(StorageError::from)?;
    Ok(())
}

pub(super) fn capture_local(conn: &mut SqliteConnection, event: &SyncOutboxEventDB) -> Result<()> {
    let Some(state) = status(conn)? else {
        return Ok(());
    };
    #[derive(QueryableByName)]
    struct Revision {
        #[diesel(sql_type = Nullable<Text>)]
        revision: Option<String>,
    }
    let base = diesel::sql_query("SELECT COALESCE((SELECT event_id FROM server_sync_client_queue WHERE entity = ? AND entity_id = ? ORDER BY seq DESC LIMIT 1), (SELECT last_event_id FROM server_sync_revisions WHERE entity = ? AND entity_id = ?)) AS revision")
        .bind::<Text,_>(&event.entity).bind::<Text,_>(&event.entity_id)
        .bind::<Text,_>(&event.entity).bind::<Text,_>(&event.entity_id).get_result::<Revision>(conn).map_err(StorageError::from)?.revision;
    let request = ServerSyncPush {
        server_id: state.server_id,
        event_id: event.event_id.clone(),
        entity: enum_from_db(&event.entity)?,
        entity_id: event.entity_id.clone(),
        op: enum_from_db(&event.op)?,
        base_event_id: base,
        payload: serde_json::from_str(&event.payload)?,
    };
    diesel::sql_query("INSERT INTO server_sync_client_queue (event_id, entity, entity_id, request_json) VALUES (?, ?, ?, ?)")
        .bind::<Text,_>(&event.event_id).bind::<Text,_>(&event.entity).bind::<Text,_>(&event.entity_id)
        .bind::<Text,_>(serde_json::to_string(&request)?).execute(conn).map_err(StorageError::from)?;
    Ok(())
}

impl AppSyncRepository {
    pub fn require_empty_server_client(&self) -> Result<()> {
        let mut conn = get_connection(&self.pool)?;
        require_empty_client(&mut conn)
    }

    pub fn server_client_status(&self) -> Result<Option<ServerClientStatus>> {
        let mut conn = get_connection(&self.pool)?;
        status(&mut conn)
    }
    pub fn next_server_change(&self) -> Result<Option<QueuedServerChange>> {
        let mut conn = get_connection(&self.pool)?;
        let row = diesel::sql_query(
            "SELECT request_json, conflict_json FROM server_sync_client_queue ORDER BY seq LIMIT 1",
        )
        .get_result::<QueueRow>(&mut conn)
        .optional()
        .map_err(StorageError::from)?;
        row.map(|r| {
            Ok(QueuedServerChange {
                request: serde_json::from_str(&r.request_json)?,
                conflict: r
                    .conflict_json
                    .map(|s| serde_json::from_str(&s))
                    .transpose()?,
            })
        })
        .transpose()
    }
    pub async fn pause_server_client(&self, paused: bool) -> Result<()> {
        self.writer
            .exec(move |conn| {
                diesel::sql_query("UPDATE server_sync_client SET paused = ? WHERE id = 1")
                    .bind::<Bool, _>(paused)
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await
    }
    pub async fn acknowledge_server_change(&self, event_id: String) -> Result<()> {
        self.writer.exec(move |conn| {
            diesel::sql_query("INSERT INTO server_sync_revisions (entity, entity_id, last_event_id) SELECT entity, entity_id, event_id FROM server_sync_client_queue WHERE event_id = ? ON CONFLICT(entity, entity_id) DO UPDATE SET last_event_id = excluded.last_event_id")
                .bind::<Text,_>(&event_id).execute(conn).map_err(StorageError::from)?;
            diesel::sql_query("DELETE FROM server_sync_client_queue WHERE event_id = ?")
                .bind::<Text,_>(&event_id).execute(conn).map_err(StorageError::from)?;
            diesel::update(sync_outbox::table.filter(sync_outbox::event_id.eq(event_id)))
                .set((sync_outbox::sent.eq(1), sync_outbox::status.eq("sent")))
                .execute(conn).map_err(StorageError::from)?; Ok(())
        }).await
    }
    pub async fn apply_server_page(&self, from_cursor: i64, page: ServerSyncPage) -> Result<()> {
        self.writer.exec(move |conn| {
            let state = status(conn)?.ok_or_else(|| invalid("Connect a server first"))?;
            if state.server_id != page.server_id || state.cursor != from_cursor || page.cursor < from_cursor
                || page.changes.last().map_or(from_cursor, |e| e.seq) != page.cursor {
                return Err(invalid("Server sync cursor mismatch"));
            }
            let mut previous = from_cursor;
            for change in page.changes {
                if change.seq <= previous { return Err(invalid("Server changes arrived out of order")); }
                previous = change.seq;
                let entity = enum_to_db(&change.entity)?;
                let pending = diesel::sql_query("SELECT request_json, conflict_json FROM server_sync_client_queue WHERE entity = ? AND entity_id = ? ORDER BY seq LIMIT 1")
                    .bind::<Text,_>(&entity).bind::<Text,_>(&change.entity_id)
                    .get_result::<QueueRow>(conn).optional().map_err(StorageError::from)?;
                if let Some(pending) = pending {
                    let request: ServerSyncPush = serde_json::from_str(&pending.request_json)?;
                    if request.base_event_id.as_deref() == Some(&change.event_id) {
                        diesel::sql_query("UPDATE server_sync_client_queue SET conflict_json = NULL WHERE entity = ? AND entity_id = ?")
                            .bind::<Text,_>(&entity).bind::<Text,_>(&change.entity_id).execute(conn).map_err(StorageError::from)?;
                    } else if request.event_id != change.event_id {
                        diesel::sql_query("UPDATE server_sync_client_queue SET conflict_json = ? WHERE entity = ? AND entity_id = ?")
                            .bind::<Text,_>(serde_json::to_string(&change)?).bind::<Text,_>(&entity)
                            .bind::<Text,_>(&change.entity_id).execute(conn).map_err(StorageError::from)?;
                    }
                } else {
                    // Own-server revisions, not wall-clock timestamps, establish order.
                    if !apply_remote_event_tx(conn, change.entity, change.entity_id.clone(), change.op,
                        change.event_id.clone(), change.timestamp.clone(), change.seq, change.payload.clone(), false)? {
                        // Replays of already-applied events are safe. Other skips cannot advance the cursor.
                        let applied = sync_applied_events::table.find(&change.event_id).count().get_result::<i64>(conn).map_err(StorageError::from)?;
                        if applied == 0 { return Err(invalid("A server change could not be applied")); }
                    }
                }
                diesel::sql_query("INSERT INTO server_sync_revisions (entity, entity_id, last_event_id) VALUES (?, ?, ?) ON CONFLICT(entity, entity_id) DO UPDATE SET last_event_id = excluded.last_event_id")
                    .bind::<Text,_>(&entity).bind::<Text,_>(&change.entity_id).bind::<Text,_>(&change.event_id)
                    .execute(conn).map_err(StorageError::from)?;
            }
            diesel::sql_query("UPDATE server_sync_client SET cursor = ?, last_sync = ? WHERE id = 1")
                .bind::<BigInt,_>(page.cursor).bind::<Text,_>(Utc::now().to_rfc3339())
                .execute(conn).map_err(StorageError::from)?; Ok(())
        }).await
    }
    pub async fn accept_server_conflict(
        &self,
        event_id: String,
        expected_server_event: String,
    ) -> Result<()> {
        self.writer.exec(move |conn| {
            let row = diesel::sql_query("SELECT request_json, conflict_json FROM server_sync_client_queue WHERE event_id = ?")
                .bind::<Text,_>(&event_id).get_result::<QueueRow>(conn).map_err(StorageError::from)?;
            let request: ServerSyncPush = serde_json::from_str(&row.request_json)?;
            let remote: ServerSyncChange = serde_json::from_str(&row.conflict_json.ok_or_else(|| invalid("No server conflict to resolve"))?)?;
            if remote.event_id != expected_server_event { return Err(invalid("The server version changed. Review it again")); }
            // Explicit user choice discards every queued edit to this same record.
            diesel::sql_query("UPDATE sync_outbox SET sent = 1, status = 'sent' WHERE event_id IN (SELECT event_id FROM server_sync_client_queue WHERE entity = ? AND entity_id = ?)")
                .bind::<Text,_>(enum_to_db(&request.entity)?).bind::<Text,_>(&request.entity_id).execute(conn).map_err(StorageError::from)?;
            diesel::sql_query("DELETE FROM server_sync_client_queue WHERE entity = ? AND entity_id = ?")
                .bind::<Text,_>(enum_to_db(&request.entity)?).bind::<Text,_>(&request.entity_id).execute(conn).map_err(StorageError::from)?;
            if !apply_remote_event_tx(conn, remote.entity, remote.entity_id, remote.op, remote.event_id,
                remote.timestamp, remote.seq, remote.payload, false)? {
                return Err(invalid("Server conflict could not be applied"));
            }
            Ok(())
        }).await
    }
}
