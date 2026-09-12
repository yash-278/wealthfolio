use crate::{
    db::{get_connection, DbPool, WriteHandle},
    errors::StorageError,
};
use async_trait::async_trait;
use diesel::{
    prelude::*,
    sql_types::{BigInt, Text},
};
use std::sync::Arc;
use wealthfolio_core::{
    captures::{Capture, CaptureRepository},
    Error, Result,
};

#[derive(QueryableByName)]
struct Row {
    #[diesel(sql_type = Text)]
    owner: String,
    #[diesel(sql_type = Text)]
    payload_hash: String,
    #[diesel(sql_type = Text)]
    body: String,
}
pub struct SqliteCaptureRepository {
    pool: Arc<DbPool>,
    writer: WriteHandle,
}
impl SqliteCaptureRepository {
    pub fn new(pool: Arc<DbPool>, writer: WriteHandle) -> Self {
        Self { pool, writer }
    }
}
#[async_trait]
impl CaptureRepository for SqliteCaptureRepository {
    async fn accept(&self, owner: &str, hash: &str, capture: Capture) -> Result<Capture> {
        let owner = owner.to_owned();
        let hash = hash.to_owned();
        self.writer.exec(move |conn| {
            let existing = diesel::sql_query("SELECT owner, payload_hash, body FROM captures WHERE owner = ? AND request_id = ?")
                .bind::<Text,_>(&owner).bind::<Text,_>(&capture.input.client_request_id)
                .get_result::<Row>(conn).optional().map_err(StorageError::from)?;
            if let Some(row) = existing {
                if row.payload_hash != hash { return Err(Error::ConstraintViolation("Request ID already used with different input".into())); }
                return Ok(serde_json::from_str(&row.body)?);
            }
            diesel::sql_query("INSERT INTO captures (id,owner,request_id,payload_hash,body) VALUES (?,?,?,?,?)")
                .bind::<Text,_>(&capture.id).bind::<Text,_>(owner).bind::<Text,_>(&capture.input.client_request_id)
                .bind::<Text,_>(hash).bind::<Text,_>(serde_json::to_string(&capture)?).execute(conn).map_err(StorageError::from)?;
            Ok(capture)
        }).await
    }
    fn get(&self, id: &str) -> Result<Option<(String, Capture)>> {
        let mut conn = get_connection(&self.pool)?;
        let row = diesel::sql_query("SELECT owner, payload_hash, body FROM captures WHERE id = ?")
            .bind::<Text, _>(id)
            .get_result::<Row>(&mut conn)
            .optional()
            .map_err(StorageError::from)?;
        row.map(|r| Ok((r.owner, serde_json::from_str(&r.body)?)))
            .transpose()
    }
    fn list_reviews(&self, page: u32, page_size: u32, reason: &str) -> Result<Vec<Capture>> {
        let mut conn = get_connection(&self.pool)?;
        let rows=diesel::sql_query("SELECT owner,payload_hash,body FROM captures WHERE EXISTS (SELECT 1 FROM json_each(captures.body,'$.reviews') r WHERE json_extract(r.value,'$.status')='open' AND (?='' OR json_extract(r.value,'$.reason')=?)) ORDER BY json_extract(body,'$.submittedAt'),id LIMIT ? OFFSET ?")
            .bind::<Text,_>(reason).bind::<Text,_>(reason).bind::<BigInt,_>(i64::from(page_size)).bind::<BigInt,_>(i64::from(page)*i64::from(page_size)).load::<Row>(&mut conn).map_err(StorageError::from)?;
        rows.into_iter()
            .map(|row| serde_json::from_str(&row.body).map_err(Into::into))
            .collect()
    }
    fn month_usage(&self, month: &str) -> Result<u64> {
        #[derive(QueryableByName)]
        struct Sum {
            #[diesel(sql_type=BigInt)]
            total: i64,
        }
        let mut conn = get_connection(&self.pool)?;
        let used = diesel::sql_query(
            "SELECT COALESCE(SUM(cost_micros),0) AS total FROM capture_ai_calls WHERE month=?",
        )
        .bind::<Text, _>(month)
        .get_result::<Sum>(&mut conn)
        .map_err(StorageError::from)?;
        Ok(used.total as u64)
    }
    fn list(&self) -> Result<Vec<Capture>> {
        let mut conn = get_connection(&self.pool)?;
        let rows =
            diesel::sql_query("SELECT owner, payload_hash, body FROM captures ORDER BY rowid DESC")
                .load::<Row>(&mut conn)
                .map_err(StorageError::from)?;
        rows.into_iter()
            .map(|r| Ok(serde_json::from_str(&r.body)?))
            .collect()
    }
    fn next_cleanup(&self, before: &str) -> Result<Option<Capture>> {
        let mut conn = get_connection(&self.pool)?;
        let row=diesel::sql_query("SELECT owner,payload_hash,body FROM captures WHERE json_extract(body,'$.status')='complete' AND length(json_extract(body,'$.input.text'))>0 AND julianday(json_extract(body,'$.submittedAt'))<julianday(?) ORDER BY rowid LIMIT 1")
            .bind::<Text,_>(before).get_result::<Row>(&mut conn).optional().map_err(StorageError::from)?;
        row.map(|row| Ok(serde_json::from_str(&row.body)?))
            .transpose()
    }
    fn next_pending(&self, now: i64, month: &str, budget: u64) -> Result<Option<Capture>> {
        let mut conn = get_connection(&self.pool)?;
        let row = diesel::sql_query("SELECT owner,payload_hash,body FROM captures WHERE (json_extract(body,'$.status') = 'processing' AND COALESCE(json_extract(body,'$.leaseUntil'),0) <= ?) OR (json_extract(body,'$.status') = 'needs_review' AND json_array_length(body,'$.candidates') = 0 AND EXISTS (SELECT 1 FROM json_each(body,'$.reviews') WHERE json_extract(value,'$.reason') = 'budget_exhausted' AND json_extract(value,'$.status') = 'open') AND (SELECT COALESCE(SUM(cost_micros),0) FROM capture_ai_calls WHERE month = ?) + 20000 <= ?) ORDER BY rowid LIMIT 1")
            .bind::<BigInt,_>(now).bind::<Text,_>(month).bind::<BigInt,_>(i64::try_from(budget).unwrap_or(i64::MAX)).get_result::<Row>(&mut conn).optional().map_err(StorageError::from)?;
        row.map(|row| Ok(serde_json::from_str(&row.body)?))
            .transpose()
    }
    async fn save(&self, mut capture: Capture, version: i64) -> Result<Capture> {
        capture.version = version + 1;
        self.writer
            .exec(move |conn| {
                let changed = diesel::sql_query(
                    "UPDATE captures SET body = ?, version = ? WHERE id = ? AND version = ?",
                )
                .bind::<Text, _>(serde_json::to_string(&capture)?)
                .bind::<BigInt, _>(capture.version)
                .bind::<Text, _>(&capture.id)
                .bind::<BigInt, _>(version)
                .execute(conn)
                .map_err(StorageError::from)?;
                if changed != 1 {
                    return Err(Error::ConstraintViolation(
                        "Review changed; reload it".into(),
                    ));
                }
                Ok(capture)
            })
            .await
    }

    fn settings(&self) -> Result<wealthfolio_core::captures::settings::CaptureSettings> {
        #[derive(QueryableByName)]
        struct Setting {
            #[diesel(sql_type = Text)]
            setting_value: String,
        }
        let mut conn = get_connection(&self.pool)?;
        let value = diesel::sql_query(
            "SELECT setting_value FROM app_settings WHERE setting_key = 'quick_add'",
        )
        .get_result::<Setting>(&mut conn)
        .optional()
        .map_err(StorageError::from)?;
        match value {
            Some(v) => Ok(serde_json::from_str(&v.setting_value)?),
            None => Ok(Default::default()),
        }
    }
    async fn save_settings(
        &self,
        settings: wealthfolio_core::captures::settings::CaptureSettings,
    ) -> Result<()> {
        self.writer.exec(move |conn| {
            diesel::sql_query("INSERT INTO app_settings (setting_key,setting_value) VALUES ('quick_add',?) ON CONFLICT(setting_key) DO UPDATE SET setting_value=excluded.setting_value")
                .bind::<Text,_>(serde_json::to_string(&settings)?).execute(conn).map_err(StorageError::from)?;
            Ok(())
        }).await
    }

    async fn reserve_budget(
        &self,
        call_id: &str,
        month: &str,
        limit: u64,
        reserve: u64,
    ) -> Result<bool> {
        let call_id = call_id.to_owned();
        let month = month.to_owned();
        self.writer.exec(move |conn| {
            #[derive(QueryableByName)] struct Sum { #[diesel(sql_type = BigInt)] total: i64 }
            let used = diesel::sql_query("SELECT COALESCE(SUM(cost_micros),0) AS total FROM capture_ai_calls WHERE month = ?")
                .bind::<Text,_>(&month).get_result::<Sum>(conn).map_err(StorageError::from)?.total as u64;
            if used.saturating_add(reserve) > limit { return Ok(false); }
            diesel::sql_query("INSERT INTO capture_ai_calls (id,month,cost_micros) VALUES (?,?,?)")
                .bind::<Text,_>(call_id).bind::<Text,_>(month).bind::<BigInt,_>(reserve as i64).execute(conn).map_err(StorageError::from)?;
            Ok(true)
        }).await
    }
    async fn settle_budget(&self, call_id: &str, actual: u64) -> Result<()> {
        let call_id = call_id.to_owned();
        self.writer
            .exec(move |conn| {
                diesel::sql_query("UPDATE capture_ai_calls SET cost_micros = ? WHERE id = ?")
                    .bind::<BigInt, _>(i64::try_from(actual).unwrap_or(i64::MAX))
                    .bind::<Text, _>(call_id)
                    .execute(conn)
                    .map_err(StorageError::from)?;
                Ok(())
            })
            .await
    }
}
