use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct CaptureSettings {
    pub provider: String,
    pub model: String,
    pub monthly_budget_micros: Option<u64>,
    pub timezone: String,
    pub mappings: Vec<AccountMapping>,
    pub typed_note_account_id: Option<String>,
    pub typed_note_today: bool,
    pub automatic_posting: bool,
    pub source_retention_days: Option<u32>,
    pub evaluation_passed: bool,
    pub qualification_version: Option<String>,
    pub extraction_version: String,
    pub supervised_trial_completed: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AccountMapping {
    pub alias: String,
    pub account_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureUsage {
    pub month: String,
    pub reserved_or_used_micros: u64,
    pub monthly_budget_micros: Option<u64>,
}
