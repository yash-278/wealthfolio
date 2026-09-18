//! One financial-event key for explicitly referenced cash statement and capture rows.
use sha2::{Digest, Sha256};

pub fn key(account_id: &str, activity_type: &str, reference: Option<&str>) -> Option<String> {
    if ![
        "WITHDRAWAL",
        "DEPOSIT",
        "TRANSFER_IN",
        "TRANSFER_OUT",
        "CREDIT",
    ]
    .contains(&activity_type)
    {
        return None;
    }
    let reference = reference?.trim();
    if reference.len() < 6 {
        return None;
    }
    let bytes = serde_json::to_vec(&(account_id, direction(activity_type)?, reference)).ok()?;
    Some(format!("bank-event-v1:{:x}", Sha256::digest(bytes)))
}

pub fn compatible(
    activity: &super::Activity,
    account: &str,
    kind: &str,
    amount: Option<rust_decimal::Decimal>,
    currency: &str,
    date: chrono::NaiveDate,
) -> bool {
    activity.account_id == account
        && activity.activity_type == kind
        && activity.amount.map(|v| v.abs()) == amount.map(|v| v.abs())
        && activity.currency == currency
        && activity.activity_date.date_naive() == date
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceEvidence {
    pub activity_id: String,
    pub source_id: String,
    pub source_system: String,
    pub reference: String,
    pub date: String,
}

pub fn referenced_matches(
    activities: Vec<super::Activity>,
    account: &str,
    kind: &str,
    reference: Option<&str>,
    event_key: &str,
) -> Vec<super::Activity> {
    activities
        .into_iter()
        .filter(|a| {
            a.account_id == account
                && direction(&a.activity_type).is_some()
                && direction(&a.activity_type) == direction(kind)
                && (a.idempotency_key.as_deref() == Some(event_key)
                    || reference
                        .map(str::trim)
                        .filter(|r| r.len() >= 6)
                        .is_some_and(|reference| {
                            a.source_record_id.as_deref().map(str::trim) == Some(reference)
                        }))
        })
        .collect()
}

// A statement may call a transfer a withdrawal or a refund a deposit. Find
// that reference too; compatibility then sends the type disagreement to review.
fn direction(kind: &str) -> Option<&'static str> {
    match kind {
        "WITHDRAWAL" | "TRANSFER_OUT" => Some("debit"),
        "DEPOSIT" | "TRANSFER_IN" | "CREDIT" => Some("credit"),
        _ => None,
    }
}
