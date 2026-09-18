mod eligibility;
pub mod extraction;
mod relationships;
pub mod settings;
use extraction::CaptureExtractor;
use settings::CaptureSettings;
// Durable text intake shared by the web and desktop hosts.
use crate::{errors::ValidationError, Result};
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum InputKind {
    BankAlert,
    CardAlert,
    TypedNote,
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaptureInput {
    pub client_request_id: String,
    pub text: String,
    pub input_kind: InputKind,
    pub sender: Option<String>,
    pub source_timestamp: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Capture {
    pub id: String,
    pub input: CaptureInput,
    pub submitted_at: String,
    #[serde(default)]
    pub timezone: String,
    #[serde(default)]
    pub extraction_attempts: u32,
    pub status: String,
    pub version: i64,
    pub reviews: Vec<Review>,
    #[serde(default)]
    pub candidates: Vec<Candidate>,
    #[serde(default)]
    pub usage: Option<extraction::TokenUsage>,
    #[serde(default)]
    pub processed_model: Option<String>,
    #[serde(default)]
    pub lease_until: Option<i64>,
    #[serde(default)]
    pub automatic_batch: bool,
    #[serde(default)]
    pub pending_resolution: Option<ResolutionRecord>,
    #[serde(default)]
    pub resolution_history: Vec<ResolutionRecord>,
    #[serde(default)]
    pub pending_category: Option<CategoryResolution>,
    #[serde(default)]
    pub pending_link: Option<relationships::PendingLink>,
    #[serde(default)]
    pub pending_refund: Option<relationships::PendingRefund>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Review {
    pub id: String,
    pub reason: String,
    pub status: String,
    pub activity_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Fields {
    pub account_id: String,
    pub amount: String,
    pub currency: String,
    pub date: String,
    pub direction: String,
    pub merchant: Option<String>,
    pub reference: Option<String>,
    pub kind: String,
}
impl Fields {
    pub fn activity_type(&self) -> &'static str {
        if ["transfer", "card_payment"].contains(&self.kind.as_str()) {
            if self.direction == "debit" {
                "TRANSFER_OUT"
            } else {
                "TRANSFER_IN"
            }
        } else if self.kind == "refund" {
            "CREDIT"
        } else if self.direction == "debit" {
            "WITHDRAWAL"
        } else {
            "DEPOSIT"
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub id: String,
    pub fields: Fields,
    pub status: String,
    pub activity_id: String,
    #[serde(default)]
    pub source_text: String,
    #[serde(default)]
    pub posting_key: String,
    #[serde(default)]
    pub source_state: String,
    #[serde(default)]
    pub source_dates: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub field_origins: std::collections::BTreeMap<String, String>,
    #[serde(default)]
    pub extraction_version: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryResolution {
    #[serde(default)]
    pub learn: bool,
    pub review_id: String,
    pub activity_id: String,
    pub taxonomy_id: String,
    pub category_id: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolveReview {
    pub version: i64,
    pub fields: Option<Fields>,
    #[serde(default)]
    pub action: Option<String>,
    pub taxonomy_id: Option<String>,
    pub category_id: Option<String>,
    pub related_activity_id: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolutionRecord {
    pub review_id: String,
    pub request_hash: String,
    pub request: ResolveReview,
    pub resolved_at: String,
}
#[async_trait]
pub trait CaptureCategorizer: Send + Sync {
    async fn assign(&self, activity_id: &str, taxonomy_id: &str, category_id: &str) -> Result<()>;
    async fn learn(&self, fields: &Fields, taxonomy: &str, category: &str) -> Result<()>;
    async fn suggest(&self, fields: &Fields) -> Result<Option<(String, String)>>;
}

#[async_trait]
pub trait CaptureRepository: Send + Sync {
    /// The owner/request pair is immutable. An identical payload replays the stored receipt.
    async fn accept(&self, owner: &str, hash: &str, capture: Capture) -> Result<Capture>;
    fn get(&self, id: &str) -> Result<Option<(String, Capture)>>;
    fn list(&self) -> Result<Vec<Capture>>;
    fn month_usage(&self, month: &str) -> Result<u64>;
    fn list_reviews(&self, page: u32, page_size: u32, reason: &str) -> Result<Vec<Capture>>;
    fn next_pending(&self, now: i64, month: &str, budget: u64) -> Result<Option<Capture>>;
    fn next_cleanup(&self, before: &str) -> Result<Option<Capture>>;
    async fn save(&self, capture: Capture, version: i64) -> Result<Capture>;
    fn settings(&self) -> Result<CaptureSettings>;
    async fn save_settings(&self, settings: CaptureSettings) -> Result<()>;
    async fn reserve_budget(
        &self,
        call_id: &str,
        month: &str,
        limit: u64,
        reserve: u64,
    ) -> Result<bool>;
    async fn settle_budget(&self, call_id: &str, actual: u64) -> Result<()>;
}

pub fn merchant_rule_id(fields: &Fields) -> String {
    let merchant = fields
        .merchant
        .as_deref()
        .unwrap_or("")
        .trim()
        .to_uppercase();
    format!(
        "quick-add-learned:{:x}",
        Sha256::digest(
            serde_json::to_vec(&(&fields.account_id, fields.activity_type(), merchant))
                .expect("string tuple")
        )
    )
}

pub struct CaptureService {
    repository: Arc<dyn CaptureRepository>,
    extractor: Arc<dyn CaptureExtractor>,
    categorizer: Arc<dyn CaptureCategorizer>,
    #[cfg(feature = "test-utils")]
    test_clock_offset: std::sync::atomic::AtomicI64,
    #[cfg(feature = "test-utils")]
    write_pause: std::sync::Mutex<Option<(Arc<tokio::sync::Notify>, Arc<tokio::sync::Notify>)>>,
    accounts: Arc<dyn crate::accounts::AccountServiceTrait>,
    activities: Arc<dyn crate::activities::ActivityServiceTrait>,
}
impl CaptureService {
    pub fn new(
        repository: Arc<dyn CaptureRepository>,
        extractor: Arc<dyn CaptureExtractor>,
        categorizer: Arc<dyn CaptureCategorizer>,
        accounts: Arc<dyn crate::accounts::AccountServiceTrait>,
        activities: Arc<dyn crate::activities::ActivityServiceTrait>,
    ) -> Self {
        Self {
            repository,
            extractor,
            categorizer,
            accounts,
            activities,
            #[cfg(feature = "test-utils")]
            test_clock_offset: std::sync::atomic::AtomicI64::new(0),
            #[cfg(feature = "test-utils")]
            write_pause: std::sync::Mutex::new(None),
        }
    }
    fn now(&self) -> i64 {
        #[cfg(feature = "test-utils")]
        {
            Utc::now().timestamp()
                + self
                    .test_clock_offset
                    .load(std::sync::atomic::Ordering::Relaxed)
        }
        #[cfg(not(feature = "test-utils"))]
        {
            Utc::now().timestamp()
        }
    }
    #[cfg(feature = "test-utils")]
    pub fn advance_lease_clock_for_test(&self, seconds: i64) {
        self.test_clock_offset
            .fetch_add(seconds, std::sync::atomic::Ordering::Relaxed);
    }
    #[cfg(feature = "test-utils")]
    pub fn pause_after_write_for_test(
        &self,
    ) -> (Arc<tokio::sync::Notify>, Arc<tokio::sync::Notify>) {
        let pair = (
            Arc::new(tokio::sync::Notify::new()),
            Arc::new(tokio::sync::Notify::new()),
        );
        *self.write_pause.lock().expect("write pause") = Some(pair.clone());
        pair
    }
    pub async fn accept(&self, owner: &str, input: CaptureInput) -> Result<Capture> {
        if input.text.trim().is_empty() || input.text.chars().count() > 16_000 {
            return Err(ValidationError::InvalidInput(
                "Enter between 1 and 16000 characters".into(),
            )
            .into());
        }
        if input.client_request_id.trim().is_empty() || input.client_request_id.len() > 128 {
            return Err(ValidationError::InvalidInput(
                "A request ID of at most 128 bytes is required".into(),
            )
            .into());
        }
        let hash = format!("{:x}", Sha256::digest(serde_json::to_vec(&input)?));
        let settings = self.repository.settings()?;
        let configured = !settings.model.is_empty();
        let capture = Capture {
            id: uuid::Uuid::new_v4().to_string(),
            input,
            timezone: settings.timezone,
            extraction_attempts: 0,
            submitted_at: Utc::now().to_rfc3339(),
            status: if configured {
                "processing"
            } else {
                "needs_review"
            }
            .into(),
            version: 1,
            lease_until: None,
            usage: None,
            processed_model: None,
            candidates: vec![],
            pending_category: None,
            pending_link: None,
            pending_refund: None,
            automatic_batch: false,
            pending_resolution: None,
            resolution_history: vec![],
            reviews: if configured {
                vec![]
            } else {
                vec![Review {
                    id: uuid::Uuid::new_v4().to_string(),
                    reason: "configuration_required".into(),
                    status: "open".into(),
                    activity_id: None,
                }]
            },
        };
        self.repository.accept(owner, &hash, capture).await
    }
    pub fn usage(&self) -> Result<settings::CaptureUsage> {
        let settings = self.settings()?;
        let month = Utc::now()
            .with_timezone(
                &settings
                    .timezone
                    .parse::<chrono_tz::Tz>()
                    .unwrap_or(chrono_tz::UTC),
            )
            .format("%Y-%m")
            .to_string();
        Ok(settings::CaptureUsage {
            reserved_or_used_micros: self.repository.month_usage(&month)?,
            month,
            monthly_budget_micros: settings.monthly_budget_micros,
        })
    }
    pub fn settings(&self) -> Result<CaptureSettings> {
        let mut settings = self.repository.settings()?;
        if settings.qualification_version.as_deref() != Some(extraction::EXTRACTION_VERSION) {
            settings.evaluation_passed = false;
            settings.supervised_trial_completed = false;
            settings.automatic_posting = false;
        }
        settings.extraction_version = extraction::EXTRACTION_VERSION.into();
        Ok(settings)
    }
    pub async fn configure(&self, mut settings: CaptureSettings) -> Result<()> {
        if settings.evaluation_passed
            && settings.extraction_version != extraction::EXTRACTION_VERSION
        {
            return Err(crate::Error::ConstraintViolation("Extraction changed; reload settings and qualify the current version before enabling automation".into()));
        }
        if settings.provider != "bedrock" || settings.model != "openai.gpt-5.6-luna" {
            return Err(ValidationError::InvalidInput(
                "Quick Add currently supports AWS Bedrock openai.gpt-5.6-luna".into(),
            )
            .into());
        }
        if settings
            .monthly_budget_micros
            .is_none_or(|v| v == 0 || v > i64::MAX as u64)
            || settings.timezone.parse::<chrono_tz::Tz>().is_err()
        {
            return Err(ValidationError::InvalidInput(
                "Set a positive monthly AI budget and valid timezone".into(),
            )
            .into());
        }
        if settings.automatic_posting
            && !(settings.evaluation_passed && settings.supervised_trial_completed)
        {
            return Err(ValidationError::InvalidInput(
                "Complete qualification and supervised trial before automatic posting".into(),
            )
            .into());
        }
        if settings
            .source_retention_days
            .is_some_and(|days| days == 0 || days > 3650)
        {
            return Err(ValidationError::InvalidInput(
                "Retention must be between 1 and 3650 days".into(),
            )
            .into());
        }
        for mapping in &settings.mappings {
            if mapping.alias.trim().is_empty() {
                return Err(ValidationError::InvalidInput(
                    "Account aliases cannot be empty".into(),
                )
                .into());
            }
            self.accounts.get_account(&mapping.account_id)?;
        }
        settings.qualification_version = if settings.evaluation_passed {
            Some(extraction::EXTRACTION_VERSION.into())
        } else {
            None
        };
        self.repository.save_settings(settings).await
    }
    /// The weak reference lets the worker stop when its host shuts down. Capture
    /// leases and candidates live in SQLite, so a replacement worker can resume.
    pub fn start_worker(service: &Arc<Self>) {
        let weak = Arc::downgrade(service);
        tokio::spawn(async move {
            let mut pending_page = 0;
            let mut next_reconciliation = tokio::time::Instant::now();
            loop {
                let Some(service) = weak.upgrade() else { break };
                let _ = service.process_next().await;
                if tokio::time::Instant::now() >= next_reconciliation {
                    if let Ok(page) = service.reconcile_pending_reviews(pending_page).await {
                        pending_page = page;
                    }
                    next_reconciliation =
                        tokio::time::Instant::now() + std::time::Duration::from_secs(5);
                }
                drop(service);
                tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            }
        });
    }
    /// Reconcile one page without writing money. Rechecking is safe after a crash
    /// because evidence is keyed by source and receipt updates use CAS.
    async fn reconcile_pending_reviews(&self, page: u32) -> Result<u32> {
        let captures = self
            .repository
            .list_reviews(page, 100, "unconfirmed_event")?;
        if captures.is_empty() {
            return Ok(0);
        }
        let settings = self.settings()?;
        let mut changed_page = false;
        for mut capture in captures {
            if capture.status != "needs_review" {
                continue;
            }
            let mut changed = false;
            for index in 0..capture.candidates.len() {
                let candidate = &capture.candidates[index];
                if candidate.source_state != "pending"
                    || candidate.status != "needs_review"
                    || !capture.reviews.iter().any(|r| {
                        r.id == candidate.id
                            && r.status == "open"
                            && r.reason == "unconfirmed_event"
                    })
                    || !eligibility::evidenced_payment(&capture, candidate, &settings)
                {
                    continue;
                }
                let Some(key) = crate::activities::bank_reference::key(
                    &candidate.fields.account_id,
                    candidate.fields.activity_type(),
                    candidate.fields.reference.as_deref(),
                ) else {
                    continue;
                };
                let (existing, conflict) = self.lookup_event(&candidate.fields, &key)?;
                if conflict {
                    continue;
                }
                if let Some(activity) = existing {
                    self.attach_evidence(&capture, index, &activity).await?;
                    Self::finish_posting(&mut capture, index, &activity);
                    changed = true;
                }
            }
            if changed {
                capture.status = if capture.reviews.iter().any(|r| r.status == "open") {
                    "needs_review"
                } else {
                    "complete"
                }
                .into();
                let version = capture.version;
                self.repository.save(capture, version).await?;
                changed_page = true;
            }
        }
        Ok(if changed_page {
            page
        } else {
            page.saturating_add(1)
        })
    }
    async fn process_next(&self) -> Result<()> {
        let now = self.now();
        let settings = self.settings()?;
        let month = Utc::now()
            .with_timezone(
                &settings
                    .timezone
                    .parse::<chrono_tz::Tz>()
                    .unwrap_or(chrono_tz::UTC),
            )
            .format("%Y-%m")
            .to_string();
        let Some(mut capture) = self.repository.next_pending(
            now,
            &month,
            settings.monthly_budget_micros.unwrap_or(0),
        )?
        else {
            if let Some(days) = settings.source_retention_days {
                let before = chrono::DateTime::from_timestamp(now - i64::from(days) * 86400, 0)
                    .ok_or_else(|| ValidationError::InvalidInput("Invalid cleanup date".into()))?
                    .to_rfc3339();
                if let Some(capture) = self.repository.next_cleanup(&before)? {
                    self.clear_source(&capture.id, capture.version).await?;
                }
            }
            return Ok(());
        };
        if capture.status == "needs_review" {
            capture.reviews.retain(|r| r.reason != "budget_exhausted");
            capture.status = "processing".into();
        }
        capture.lease_until = Some(now + 120);
        let version = capture.version;
        let mut capture = self.repository.save(capture, version).await?;
        if capture.pending_refund.is_some() {
            self.finish_refund(capture).await?;
            return Ok(());
        }
        if capture.pending_link.is_some() {
            let capture = self.finish_link(capture).await?;
            if capture.automatic_batch {
                self.run_automatic_batch(capture).await?;
            }
            return Ok(());
        }
        if capture.pending_category.is_some() {
            let capture = self.finish_category(capture).await?;
            if capture.automatic_batch {
                self.run_automatic_batch(capture).await?;
            }
            return Ok(());
        }
        if capture.candidates.iter().any(|c| c.status == "posting") {
            for index in 0..capture.candidates.len() {
                let candidate = &capture.candidates[index];
                if candidate.status != "posting" {
                    continue;
                }
                let key = if candidate.posting_key.is_empty() {
                    format!("capture:{}", candidate.id)
                } else {
                    candidate.posting_key.clone()
                };
                let (activity, conflict) = self.lookup_event(&candidate.fields, &key)?;
                if let Some(activity) = activity {
                    self.attach_evidence(&capture, index, &activity).await?;
                    Self::finish_posting(&mut capture, index, &activity);
                    self.prepare_category(&mut capture, index).await?;
                    self.prepare_transfer_link(&mut capture, index)?;
                } else {
                    let candidate = &mut capture.candidates[index];
                    candidate.status = "needs_review".into();
                    for review in &mut capture.reviews {
                        if review.id == candidate.id {
                            review.reason = if conflict {
                                "reference_conflict"
                            } else {
                                "write_interrupted"
                            }
                            .into();
                        }
                    }
                }
            }
            capture.status = if capture.automatic_batch {
                "processing"
            } else {
                "needs_review"
            }
            .into();
            if !capture.automatic_batch {
                capture.lease_until = None;
            }
            let version = capture.version;
            if capture.pending_category.is_some() || capture.pending_link.is_some() {
                capture.status = "processing".into();
                capture.lease_until = Some(self.now() + 120);
            }
            let mut capture = self.repository.save(capture, version).await?;
            if capture.pending_category.is_some() {
                capture = self.finish_category(capture).await?;
            }
            if capture.pending_link.is_some() {
                capture = self.finish_link(capture).await?;
            }
            if capture.automatic_batch {
                self.run_automatic_batch(capture).await?;
            }
            return Ok(());
        }
        if capture.automatic_batch {
            self.run_automatic_batch(capture).await?;
            return Ok(());
        }
        if capture.extraction_attempts >= 2 {
            capture.status = "needs_review".into();
            capture.lease_until = None;
            capture.reviews.push(Review {
                id: uuid::Uuid::new_v4().to_string(),
                reason: "extraction_failed".into(),
                status: "open".into(),
                activity_id: None,
            });
            let version = capture.version;
            self.repository.save(capture, version).await?;
            return Ok(());
        }
        let call_id = uuid::Uuid::new_v4().to_string();
        // $0.02 reserves the maximum bounded input and output at Luna's published
        // rates. Missing usage keeps the entire allowance charged to this month.
        if !self
            .repository
            .reserve_budget(
                &call_id,
                &month,
                settings.monthly_budget_micros.unwrap_or(0),
                20_000,
            )
            .await?
        {
            capture.status = "needs_review".into();
            capture.lease_until = None;
            capture.reviews.push(Review {
                id: uuid::Uuid::new_v4().to_string(),
                reason: "budget_exhausted".into(),
                status: "open".into(),
                activity_id: None,
            });
            let version = capture.version;
            self.repository.save(capture, version).await?;
            return Ok(());
        }
        capture.extraction_attempts += 1;
        let version = capture.version;
        let mut capture = self.repository.save(capture, version).await?;
        let result = self
            .extractor
            .extract(&capture.input, &settings.provider, &settings.model)
            .await;
        if let Ok(ref extracted) = result {
            if let Some(ref usage) = extracted.usage {
                let cost = usage
                    .input_tokens
                    .saturating_add(usage.output_tokens.saturating_mul(6))
                    .saturating_mul(11)
                    .div_ceil(50);
                self.repository.settle_budget(&call_id, cost).await?;
                capture.usage = Some(usage.clone());
            }
        }
        capture.processed_model = Some(settings.model.clone());
        match result {
            Ok(extraction) if !extraction.events.is_empty() && extraction.events.len() <= 20 => {
                for event in extraction.events {
                    let accounts: std::collections::HashSet<_> = settings
                        .mappings
                        .iter()
                        .filter(|m| {
                            event
                                .account_hint
                                .as_ref()
                                .is_some_and(|h| eligibility::account_hint_matches(&m.alias, h))
                        })
                        .map(|m| m.account_id.clone())
                        .collect();
                    let mut account_id = if accounts.len() == 1 {
                        accounts.into_iter().next().unwrap_or_default()
                    } else {
                        String::new()
                    };
                    let default_account = event.account_hint.is_none()
                        && capture.input.input_kind == InputKind::TypedNote
                        && settings.typed_note_account_id.is_some();
                    let default_date = event.date.is_none()
                        && capture.input.input_kind == InputKind::TypedNote
                        && settings.typed_note_today;
                    let mut date = event.date.unwrap_or_default();
                    if capture.input.input_kind == InputKind::TypedNote {
                        if event.account_hint.is_none() {
                            account_id = settings.typed_note_account_id.clone().unwrap_or_default();
                        }
                        if date.is_empty() && settings.typed_note_today {
                            date = chrono::DateTime::parse_from_rfc3339(&capture.submitted_at)?
                                .with_timezone(
                                    &capture
                                        .timezone
                                        .parse::<chrono_tz::Tz>()
                                        .unwrap_or(chrono_tz::UTC),
                                )
                                .date_naive()
                                .to_string();
                        }
                    }
                    let reason = if !capture.input.text.contains(&event.source_text)
                        || event.source_text.is_empty()
                    {
                        "invalid_evidence"
                    } else if event.state != "completed" {
                        "unconfirmed_event"
                    } else if account_id.is_empty() {
                        "account_required"
                    } else if date.is_empty() {
                        "date_required"
                    } else {
                        "supervised_review"
                    };
                    let candidate = Candidate {
                        id: uuid::Uuid::new_v4().to_string(),
                        status: "needs_review".into(),
                        activity_id: uuid::Uuid::new_v4().to_string(),
                        source_text: event.source_text,
                        posting_key: String::new(),
                        source_state: event.state,
                        source_dates: [
                            ("transactionDate", event.transaction_date),
                            ("bookingDate", event.booking_date),
                            ("valueDate", event.value_date),
                        ]
                        .into_iter()
                        .filter_map(|(k, v)| v.map(|v| (k.into(), v)))
                        .collect(),
                        extraction_version: extraction::EXTRACTION_VERSION.into(),
                        field_origins: [
                            (
                                "accountId",
                                if account_id.is_empty() {
                                    "unresolved"
                                } else if default_account {
                                    "configured_default"
                                } else {
                                    "mapped_source"
                                },
                            ),
                            (
                                "date",
                                if date.is_empty() {
                                    "unresolved"
                                } else if default_date {
                                    "configured_default"
                                } else {
                                    "model_proposal"
                                },
                            ),
                            ("amount", "model_proposal"),
                            ("currency", "model_proposal"),
                            ("direction", "model_proposal"),
                            ("kind", "model_proposal"),
                            ("merchant", "model_proposal"),
                            ("reference", "model_proposal"),
                        ]
                        .into_iter()
                        .map(|(k, v)| (k.into(), v.into()))
                        .collect(),
                        fields: Fields {
                            account_id,
                            amount: event.amount.unwrap_or_default(),
                            currency: event.currency.unwrap_or_default(),
                            date,
                            direction: event.direction.unwrap_or_default(),
                            merchant: event.merchant,
                            reference: event.reference,
                            kind: event.kind,
                        },
                    };
                    capture.reviews.push(Review {
                        id: candidate.id.clone(),
                        reason: reason.into(),
                        status: "open".into(),
                        activity_id: None,
                    });
                    capture.candidates.push(candidate);
                }
            }
            _ => capture.reviews.push(Review {
                id: uuid::Uuid::new_v4().to_string(),
                reason: "extraction_failed".into(),
                status: "open".into(),
                activity_id: None,
            }),
        }
        capture.automatic_batch = settings.automatic_posting
            && settings.evaluation_passed
            && settings.supervised_trial_completed;
        capture.status = if capture.automatic_batch {
            "processing"
        } else {
            "needs_review"
        }
        .into();
        if !capture.automatic_batch {
            capture.lease_until = None;
        }
        let version = capture.version;
        let capture = self.repository.save(capture, version).await?;
        if capture.automatic_batch {
            self.run_automatic_batch(capture).await?;
        }
        Ok(())
    }
    async fn run_automatic_batch(&self, capture: Capture) -> Result<()> {
        for candidate in &capture.candidates {
            let current_settings = self.settings()?;
            if !(current_settings.automatic_posting
                && current_settings.evaluation_passed
                && current_settings.supervised_trial_completed)
            {
                break;
            }
            let current = self
                .get(&capture.id, None)?
                .ok_or_else(|| ValidationError::InvalidInput("Capture missing".into()))?;
            if current.reviews.iter().any(|r| {
                r.id == candidate.id && r.status == "open" && r.reason == "supervised_review"
            }) && eligibility::evidenced_payment(&current, candidate, &current_settings)
            {
                let key = crate::activities::bank_reference::key(
                    &candidate.fields.account_id,
                    candidate.fields.activity_type(),
                    candidate.fields.reference.as_deref(),
                );
                let conflicting_block = current.candidates.iter().any(|other| {
                    other.id != candidate.id
                        && key.is_some()
                        && crate::activities::bank_reference::key(
                            &other.fields.account_id,
                            other.fields.activity_type(),
                            other.fields.reference.as_deref(),
                        ) == key
                        && (other.fields.amount != candidate.fields.amount
                            || other.fields.currency != candidate.fields.currency
                            || other.fields.date != candidate.fields.date)
                });
                let similar = key.is_none()
                    && self
                        .activities
                        .get_activities_by_account_id(&candidate.fields.account_id)?
                        .iter()
                        .any(|activity| {
                            Self::compatible(activity, &candidate.fields)
                                && activity
                                    .notes
                                    .as_deref()
                                    .map(str::trim)
                                    .map(str::to_lowercase)
                                    == candidate
                                        .fields
                                        .merchant
                                        .as_deref()
                                        .map(str::trim)
                                        .map(str::to_lowercase)
                        });
                if conflicting_block || similar {
                    let mut current = current;
                    if let Some(review) = current.reviews.iter_mut().find(|r| r.id == candidate.id)
                    {
                        review.reason = if conflicting_block {
                            "reference_conflict"
                        } else {
                            "possible_duplicate"
                        }
                        .into();
                    }
                    let version = current.version;
                    self.repository.save(current, version).await?;
                    continue;
                }
                let _ = self
                    .resolve_inner(
                        &candidate.id,
                        ResolveReview {
                            version: current.version,
                            fields: Some(candidate.fields.clone()),
                            action: None,
                            taxonomy_id: None,
                            category_id: None,
                            related_activity_id: None,
                        },
                        true,
                    )
                    .await;
            }
        }
        let mut current = self
            .get(&capture.id, None)?
            .ok_or_else(|| ValidationError::InvalidInput("Capture missing".into()))?;
        if current.candidates.iter().any(|c| c.status == "posting") {
            return Ok(());
        }
        current.automatic_batch = false;
        current.lease_until = None;
        current.status = if current.reviews.iter().any(|r| r.status == "open") {
            "needs_review"
        } else {
            "complete"
        }
        .into();
        let version = current.version;
        self.repository.save(current, version).await?;
        Ok(())
    }
    pub async fn resolve(&self, review_id: &str, resolution: ResolveReview) -> Result<Capture> {
        self.resolve_inner(review_id, resolution, false).await
    }
    async fn resolve_inner(
        &self,
        review_id: &str,
        resolution: ResolveReview,
        automatic: bool,
    ) -> Result<Capture> {
        use crate::activities::NewActivity;
        use rust_decimal::Decimal;
        use std::str::FromStr;
        let mut capture = self
            .repository
            .list()?
            .into_iter()
            .find(|c| c.reviews.iter().any(|r| r.id == review_id))
            .ok_or_else(|| ValidationError::InvalidInput("Review not found".into()))?;
        let request_hash = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&(review_id, &resolution))?)
        );
        if capture
            .resolution_history
            .iter()
            .any(|record| record.request_hash == request_hash)
        {
            return Ok(capture);
        }
        let record = ResolutionRecord {
            review_id: review_id.into(),
            request_hash,
            request: resolution.clone(),
            resolved_at: Utc::now().to_rfc3339(),
        };
        if (!automatic && capture.automatic_batch)
            || capture.pending_category.is_some()
            || capture.pending_link.is_some()
            || capture.pending_refund.is_some()
            || capture
                .candidates
                .iter()
                .any(|candidate| candidate.status == "posting")
        {
            return Err(crate::Error::ConstraintViolation(
                "Payment is being saved; wait for reconciliation".into(),
            ));
        }
        if resolution.action.as_deref() == Some("link_refund") {
            if capture.version != resolution.version {
                return Err(crate::Error::ConstraintViolation(
                    "Review changed; reload it".into(),
                ));
            }
            let review = capture
                .reviews
                .iter()
                .find(|r| {
                    r.id == review_id
                        && r.status == "open"
                        && r.reason == "original_payment_required"
                })
                .ok_or_else(|| ValidationError::InvalidInput("Refund review not found".into()))?;
            let refund_id = review
                .activity_id
                .clone()
                .ok_or_else(|| ValidationError::InvalidInput("Save the refund first".into()))?;
            let original_id = resolution
                .related_activity_id
                .ok_or_else(|| ValidationError::MissingField("relatedActivityId".into()))?;
            self.validate_refund_link(&refund_id, &original_id)?;
            capture.pending_refund = Some(relationships::PendingRefund {
                review_id: review_id.into(),
                refund_id,
                original_id,
            });
            capture.pending_resolution = Some(record);
            capture.status = "processing".into();
            capture.lease_until = Some(self.now() + 120);
            let version = capture.version;
            let capture = self.repository.save(capture, version).await?;
            return self.finish_refund(capture).await;
        }
        if resolution.action.as_deref() == Some("categorize") {
            if capture.version != resolution.version {
                return Err(crate::Error::ConstraintViolation(
                    "Review changed; reload it".into(),
                ));
            }
            let review = capture
                .reviews
                .iter()
                .find(|r| {
                    r.id == review_id && r.status == "open" && r.reason == "category_required"
                })
                .ok_or_else(|| ValidationError::InvalidInput("Category review not found".into()))?;
            let activity_id = review
                .activity_id
                .as_deref()
                .ok_or_else(|| {
                    ValidationError::InvalidInput(
                        "Payment must be saved before categorization".into(),
                    )
                })?
                .to_string();
            let taxonomy = resolution
                .taxonomy_id
                .ok_or_else(|| ValidationError::MissingField("taxonomyId".into()))?;
            let category = resolution
                .category_id
                .ok_or_else(|| ValidationError::MissingField("categoryId".into()))?;
            capture.pending_resolution = Some(record);
            capture.pending_category = Some(CategoryResolution {
                learn: true,
                review_id: review_id.into(),
                activity_id,
                taxonomy_id: taxonomy,
                category_id: category,
            });
            capture.status = "processing".into();
            capture.lease_until = Some(self.now() + 120);
            let version = capture.version;
            let capture = self.repository.save(capture, version).await?;
            return self.finish_category(capture).await;
        }
        if resolution.action.as_deref() == Some("complete") {
            if capture.version != resolution.version {
                return Err(crate::Error::ConstraintViolation(
                    "Review changed; reload it".into(),
                ));
            }
            let review = capture
                .reviews
                .iter_mut()
                .find(|r| {
                    r.id == review_id
                        && r.status == "open"
                        && ["balance_update_required", "counterpart_required"]
                            .contains(&r.reason.as_str())
                        && r.activity_id.is_some()
                })
                .ok_or_else(|| {
                    ValidationError::InvalidInput(
                        "Only saved-payment balance follow-up can be completed here".into(),
                    )
                })?;
            if review.reason == "counterpart_required" {
                let activity = self
                    .activities
                    .get_activity(review.activity_id.as_deref().expect("saved activity"))?;
                if activity.source_group_id.is_none()
                    || !self.activities.get_activities()?.iter().any(|other| {
                        other.id != activity.id
                            && other.source_group_id == activity.source_group_id
                            && other.activity_type != activity.activity_type
                            && ["TRANSFER_IN", "TRANSFER_OUT"]
                                .contains(&other.activity_type.as_str())
                    })
                {
                    return Err(ValidationError::InvalidInput(
                        "Link the confirmed counterpart before completing review".into(),
                    )
                    .into());
                }
            }
            review.status = "resolved".into();
            capture.resolution_history.push(record);
            if capture.reviews.iter().all(|r| r.status != "open") {
                capture.status = "complete".into();
            }
            let version = capture.version;
            return self.repository.save(capture, version).await;
        }
        if capture.version != resolution.version
            || !capture
                .reviews
                .iter()
                .any(|r| r.id == review_id && r.status == "open" && r.activity_id.is_none())
        {
            return Err(crate::Error::ConstraintViolation(
                "Review changed; reload it".into(),
            ));
        }
        let fields = resolution
            .fields
            .ok_or_else(|| ValidationError::InvalidInput("Payment fields are required".into()))?;
        let account = self.accounts.get_account(&fields.account_id)?;
        if !account.is_active || account.is_archived || account.currency != fields.currency {
            return Err(ValidationError::InvalidInput(
                "Select an active account with matching currency".into(),
            )
            .into());
        }
        let amount = Decimal::from_str(&fields.amount)?;
        if amount <= Decimal::ZERO
            || !["debit", "credit"].contains(&fields.direction.as_str())
            || ![
                "payment",
                "loan_payment",
                "investment_payment",
                "epf_payment",
                "transfer",
                "card_payment",
                "refund",
            ]
            .contains(&fields.kind.as_str())
            || (fields.kind == "refund" && fields.direction != "credit")
        {
            return Err(ValidationError::InvalidInput(
                "Enter a positive payment amount and direction".into(),
            )
            .into());
        }
        let date = chrono::NaiveDate::parse_from_str(&fields.date, "%Y-%m-%d")?;
        let previous = capture.candidates.iter().find(|c| c.id == review_id);
        let posting_key = crate::activities::bank_reference::key(
            &fields.account_id,
            fields.activity_type(),
            fields.reference.as_deref(),
        )
        .unwrap_or_else(|| format!("capture:{review_id}"));
        let candidate = Candidate {
            posting_key: posting_key.clone(),
            id: review_id.to_string(),
            fields: fields.clone(),
            status: "posting".into(),
            activity_id: previous
                .map(|c| c.activity_id.clone())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            source_text: previous
                .map(|c| c.source_text.clone())
                .unwrap_or_else(|| capture.input.text.clone()),
            source_dates: previous.map(|c| c.source_dates.clone()).unwrap_or_default(),
            source_state: previous
                .map(|c| c.source_state.clone())
                .unwrap_or_else(|| "user_confirmed".into()),
            extraction_version: previous
                .map(|c| c.extraction_version.clone())
                .unwrap_or_default(),
            field_origins: if automatic {
                previous
                    .map(|c| c.field_origins.clone())
                    .unwrap_or_default()
            } else {
                [
                    "accountId",
                    "date",
                    "amount",
                    "currency",
                    "direction",
                    "kind",
                    "merchant",
                    "reference",
                ]
                .into_iter()
                .map(|key| (key.into(), "user_confirmation".into()))
                .collect()
            },
        };
        let activity: NewActivity = serde_json::from_value(serde_json::json!({
            "id": candidate.activity_id, "accountId": fields.account_id,
            "activityType": fields.activity_type(),
            "activityDate": format!("{}T12:00:00Z", date), "amount": fields.amount, "currency": fields.currency,
            "notes": fields.merchant, "sourceSystem":"QUICK_ADD", "sourceRecordId":fields.reference,
            "sourceGroupId": if ["transfer","card_payment"].contains(&fields.kind.as_str()) {format!("capture:{}:{}",capture.id,review_id)} else {capture.id.clone()}, "idempotencyKey":posting_key, "needsReview":false,
            "subtype":if fields.kind=="refund" {Some("REFUND")} else {None},
            "metadata":serde_json::json!({"quick_add":{"capture_id":capture.id,"candidate_id":review_id},"flow":if ["transfer","card_payment","refund"].contains(&fields.kind.as_str()) {serde_json::json!({"is_external":false})} else {serde_json::Value::Null}}).to_string()
        }))?;
        capture.pending_resolution = Some(record);
        capture.candidates.retain(|c| c.id != review_id);
        capture.candidates.push(candidate);
        capture.status = "processing".into();
        capture.lease_until = Some(self.now() + 120);
        let version = capture.version;
        let mut capture = self.repository.save(capture, version).await?;
        // The durable candidate ID is the idempotency key. The ledger generates
        // its own activity ID, which must come from the successful write result.
        let (existing, conflict) = self.lookup_event(&fields, &posting_key)?;
        let write_result = if conflict {
            Err(crate::Error::ConstraintViolation(
                "Conflicting bank reference requires review".into(),
            ))
        } else if let Some(existing) = existing {
            Ok((existing, true))
        } else {
            match self.activities.create_activity(activity).await {
                Ok(created) => Ok((created, false)),
                Err(error) => {
                    let (existing, _) = self.lookup_event(&fields, &posting_key)?;
                    match existing {
                        Some(existing) => Ok((existing, true)),
                        None => Err(error),
                    }
                }
            }
        };
        match write_result {
            Ok((created, _already_recorded)) => {
                #[cfg(feature = "test-utils")]
                {
                    let pause = self.write_pause.lock().expect("write pause").clone();
                    if let Some((written, release)) = pause {
                        written.notify_one();
                        release.notified().await;
                    }
                }
                let index = capture
                    .candidates
                    .iter()
                    .position(|c| c.id == review_id)
                    .expect("candidate persisted");
                self.attach_evidence(&capture, index, &created).await?;
                Self::finish_posting(&mut capture, index, &created);
                self.prepare_category(&mut capture, index).await?;
                self.prepare_transfer_link(&mut capture, index)?;
            }
            Err(_) => {
                capture
                    .candidates
                    .iter_mut()
                    .filter(|c| c.id == review_id)
                    .for_each(|c| c.status = "needs_review".into());
                capture
                    .reviews
                    .iter_mut()
                    .filter(|r| r.id == review_id)
                    .for_each(|r| r.reason = "write_failed".into());
            }
        }
        capture.status = if capture.automatic_batch {
            "processing"
        } else if capture.reviews.iter().any(|r| r.status == "open") {
            "needs_review"
        } else {
            "complete"
        }
        .into();
        if !capture.automatic_batch {
            capture.lease_until = None;
        }
        if capture.pending_category.is_some() || capture.pending_link.is_some() {
            capture.status = "processing".into();
            capture.lease_until = Some(self.now() + 120);
        }
        let version = capture.version;
        let capture = self.repository.save(capture, version).await?;
        if capture.pending_category.is_some() {
            self.finish_category(capture).await
        } else if capture.pending_link.is_some() {
            self.finish_link(capture).await
        } else {
            Ok(capture)
        }
    }
    async fn prepare_category(&self, capture: &mut Capture, index: usize) -> Result<()> {
        let candidate = &capture.candidates[index];
        if candidate.status != "posted" {
            return Ok(());
        }
        if let Some((taxonomy, category)) = self.categorizer.suggest(&candidate.fields).await? {
            if let Some(review) = capture.reviews.iter().find(|r| {
                r.status == "open"
                    && r.reason == "category_required"
                    && r.activity_id.as_deref() == Some(candidate.activity_id.as_str())
            }) {
                capture.pending_category = Some(CategoryResolution {
                    learn: false,
                    review_id: review.id.clone(),
                    activity_id: candidate.activity_id.clone(),
                    taxonomy_id: taxonomy,
                    category_id: category,
                });
            }
        }
        Ok(())
    }
    async fn finish_category(&self, mut capture: Capture) -> Result<Capture> {
        let pending = capture
            .pending_category
            .as_ref()
            .expect("category operation persisted");
        let result = self
            .categorizer
            .assign(
                &pending.activity_id,
                &pending.taxonomy_id,
                &pending.category_id,
            )
            .await;
        if result.is_ok() {
            if pending.learn {
                if let Some(candidate) = capture
                    .candidates
                    .iter()
                    .find(|c| c.activity_id == pending.activity_id)
                {
                    self.categorizer
                        .learn(
                            &candidate.fields,
                            &pending.taxonomy_id,
                            &pending.category_id,
                        )
                        .await?;
                }
            }
            if let Some(record) = capture.pending_resolution.take() {
                capture.resolution_history.push(record);
            }
            for review in &mut capture.reviews {
                if review.id == pending.review_id {
                    review.status = "resolved".into();
                }
            }
        }
        capture.pending_resolution = None;
        capture.pending_category = None;
        capture.lease_until = None;
        capture.status = if capture.automatic_batch {
            "processing"
        } else if capture.reviews.iter().any(|r| r.status == "open") {
            "needs_review"
        } else {
            "complete"
        }
        .into();
        if capture.automatic_batch {
            capture.lease_until = Some(self.now() + 120);
        }
        let version = capture.version;
        let saved = self.repository.save(capture, version).await?;
        result?;
        Ok(saved)
    }
    async fn attach_evidence(
        &self,
        capture: &Capture,
        index: usize,
        activity: &crate::activities::Activity,
    ) -> Result<()> {
        let candidate = &capture.candidates[index];
        self.activities
            .attach_source_evidence(crate::activities::bank_reference::SourceEvidence {
                activity_id: activity.id.clone(),
                source_id: format!("capture:{}:{}", capture.id, candidate.id),
                source_system: "QUICK_ADD".into(),
                reference: candidate.fields.reference.clone().unwrap_or_default(),
                date: candidate.fields.date.clone(),
            })
            .await
    }
    fn lookup_event(
        &self,
        fields: &Fields,
        key: &str,
    ) -> Result<(Option<crate::activities::Activity>, bool)> {
        let matches = crate::activities::bank_reference::referenced_matches(
            self.activities
                .get_activities_by_account_id(&fields.account_id)?,
            &fields.account_id,
            fields.activity_type(),
            fields.reference.as_deref(),
            key,
        );
        if matches.len() == 1 && Self::compatible(&matches[0], fields) {
            Ok((matches.into_iter().next(), false))
        } else {
            Ok((None, !matches.is_empty()))
        }
    }
    fn compatible(activity: &crate::activities::Activity, fields: &Fields) -> bool {
        use std::str::FromStr;
        let Ok(date) = chrono::NaiveDate::parse_from_str(&fields.date, "%Y-%m-%d") else {
            return false;
        };
        crate::activities::bank_reference::compatible(
            activity,
            &fields.account_id,
            fields.activity_type(),
            rust_decimal::Decimal::from_str(&fields.amount).ok(),
            &fields.currency,
            date,
        )
    }
    fn finish_posting(capture: &mut Capture, index: usize, activity: &crate::activities::Activity) {
        if let Some(record) = capture.pending_resolution.take() {
            capture.resolution_history.push(record);
        }
        let own_write = activity.source_group_id.as_deref() == Some(capture.id.as_str())
            || activity
                .metadata
                .as_ref()
                .and_then(|m| m["quick_add"]["capture_id"].as_str())
                == Some(capture.id.as_str());
        let candidate = &mut capture.candidates[index];
        candidate.activity_id = activity.id.clone();
        candidate.status = if own_write {
            "posted"
        } else {
            "already_recorded"
        }
        .into();
        for review in &mut capture.reviews {
            if review.id == candidate.id {
                review.status = "resolved".into();
            }
        }
        if own_write {
            let mut reasons =
                if ["transfer", "card_payment"].contains(&candidate.fields.kind.as_str()) {
                    vec!["counterpart_required"]
                } else {
                    vec!["category_required"]
                };
            if candidate.fields.kind == "refund" {
                reasons.push("original_payment_required");
            }
            if ["loan_payment", "investment_payment", "epf_payment"]
                .contains(&candidate.fields.kind.as_str())
            {
                reasons.push("balance_update_required");
            }
            for reason in reasons {
                if !capture.reviews.iter().any(|r| {
                    r.reason == reason && r.activity_id.as_deref() == Some(activity.id.as_str())
                }) {
                    capture.reviews.push(Review {
                        id: uuid::Uuid::new_v4().to_string(),
                        reason: reason.into(),
                        status: "open".into(),
                        activity_id: Some(activity.id.clone()),
                    });
                }
            }
        }
    }
    pub async fn retry(&self, id: &str, version: i64) -> Result<Capture> {
        let mut capture = self
            .get(id, None)?
            .ok_or_else(|| ValidationError::InvalidInput("Capture not found".into()))?;
        if capture.version != version
            || capture.status == "processing"
            || !capture.candidates.is_empty()
            || capture.input.text.is_empty()
            || !capture.reviews.iter().any(|r| {
                r.status == "open"
                    && [
                        "configuration_required",
                        "budget_exhausted",
                        "extraction_failed",
                    ]
                    .contains(&r.reason.as_str())
            })
        {
            return Err(crate::Error::ConstraintViolation(
                "Only unfinished extraction can be retried; reload review".into(),
            ));
        }
        if self.settings()?.model.is_empty() {
            return Err(ValidationError::InvalidInput(
                "Configure Quick Add before retrying".into(),
            )
            .into());
        }
        capture.reviews.clear();
        capture.extraction_attempts = 0;
        capture.status = "processing".into();
        capture.lease_until = None;
        self.repository.save(capture, version).await
    }
    pub async fn clear_source(&self, id: &str, version: i64) -> Result<Capture> {
        let mut capture = self
            .get(id, None)?
            .ok_or_else(|| ValidationError::InvalidInput("Capture not found".into()))?;
        if capture.version != version
            || capture.status == "processing"
            || capture.reviews.iter().any(|r| r.status == "open")
        {
            return Err(crate::Error::ConstraintViolation(
                "Resolve review before deleting source text".into(),
            ));
        }
        capture.input.text.clear();
        capture.input.sender = None;
        for candidate in &mut capture.candidates {
            candidate.source_text.clear();
        }
        self.repository.save(capture, version).await
    }
    pub fn reviews(&self, page: u32, page_size: u32, reason: Option<&str>) -> Result<Vec<Capture>> {
        if !(1..=100).contains(&page_size) {
            return Err(ValidationError::InvalidInput(
                "Review page size must be between 1 and 100".into(),
            )
            .into());
        }
        self.repository
            .list_reviews(page, page_size, reason.unwrap_or_default())
    }
    pub async fn dismiss(&self, review_id: &str, version: i64) -> Result<Capture> {
        let mut capture = self
            .repository
            .list()?
            .into_iter()
            .find(|c| c.reviews.iter().any(|r| r.id == review_id))
            .ok_or_else(|| {
                crate::Error::Validation(ValidationError::InvalidInput("Review not found".into()))
            })?;
        let resolution = ResolveReview {
            version,
            fields: None,
            action: Some("dismiss".into()),
            taxonomy_id: None,
            category_id: None,
            related_activity_id: None,
        };
        let request_hash = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&(review_id, &resolution))?)
        );
        if capture
            .resolution_history
            .iter()
            .any(|record| record.request_hash == request_hash)
        {
            return Ok(capture);
        }
        if capture.version != version
            || capture.automatic_batch
            || capture.pending_category.is_some()
            || capture.pending_link.is_some()
            || capture.pending_refund.is_some()
            || capture
                .candidates
                .iter()
                .any(|candidate| candidate.status == "posting")
        {
            return Err(crate::Error::ConstraintViolation(
                "Review changed or payment is being saved; reload it".into(),
            ));
        }
        let review = capture
            .reviews
            .iter_mut()
            .find(|r| r.id == review_id && r.status == "open")
            .ok_or_else(|| {
                crate::Error::ConstraintViolation("Review already resolved; reload it".into())
            })?;
        review.status = "dismissed".into();
        capture.resolution_history.push(ResolutionRecord {
            review_id: review_id.into(),
            request_hash,
            request: resolution,
            resolved_at: Utc::now().to_rfc3339(),
        });
        if capture.reviews.iter().all(|r| r.status != "open") {
            capture.status = "complete".into();
        }
        self.repository.save(capture, version).await
    }
    pub fn get(&self, id: &str, owner: Option<&str>) -> Result<Option<Capture>> {
        Ok(self
            .repository
            .get(id)?
            .and_then(|(stored_owner, capture)| {
                if owner.is_none_or(|o| o == stored_owner) {
                    Some(capture)
                } else {
                    None
                }
            }))
    }
}
