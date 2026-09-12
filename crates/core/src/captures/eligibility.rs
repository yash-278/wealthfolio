//! Conservative source checks for automatic posting. Unsupported wording stays in review.
use super::{settings::CaptureSettings, Candidate, Capture, InputKind};
use chrono::DateTime;
use regex::Regex;
use rust_decimal::Decimal;
use std::str::FromStr;

pub fn evidenced_payment(
    capture: &Capture,
    candidate: &Candidate,
    settings: &CaptureSettings,
) -> bool {
    let fields = &candidate.fields;
    if ![
        "payment",
        "loan_payment",
        "investment_payment",
        "epf_payment",
    ]
    .contains(&fields.kind.as_str())
    {
        return false;
    }
    let Ok(amount) = Decimal::from_str(&fields.amount) else {
        return false;
    };
    if amount <= Decimal::ZERO || !["debit", "credit"].contains(&fields.direction.as_str()) {
        return false;
    }
    let source = &candidate.source_text;
    if source.is_empty() || !capture.input.text.contains(source) {
        return false;
    }
    if let Some(reference) = fields.reference.as_deref() {
        if reference.len() < 6 || !source.contains(reference) {
            return false;
        }
    }
    let mappings: std::collections::HashSet<_> = settings
        .mappings
        .iter()
        .filter(|m| account_alias_in_source(source, &m.alias))
        .map(|m| m.account_id.as_str())
        .collect();
    let account_evidenced = mappings.len() == 1 && mappings.contains(fields.account_id.as_str());
    let typed_default = capture.input.input_kind == InputKind::TypedNote
        && settings.typed_note_account_id.as_deref() == Some(fields.account_id.as_str())
        && mappings.is_empty();
    if !account_evidenced && !typed_default {
        return false;
    }
    let local_submission_date = DateTime::parse_from_rfc3339(&capture.submitted_at)
        .ok()
        .zip(capture.timezone.parse::<chrono_tz::Tz>().ok())
        .map(|(date, tz)| date.with_timezone(&tz).date_naive().to_string());
    let date_pattern = Regex::new(r"\b[0-9]{4}-[0-9]{2}-[0-9]{2}\b").expect("date pattern");
    let dates: std::collections::HashSet<_> =
        date_pattern.find_iter(source).map(|m| m.as_str()).collect();
    if dates.len() > 1 {
        return false;
    }
    if !source.contains(&fields.date)
        && !(capture.input.input_kind == InputKind::TypedNote
            && settings.typed_note_today
            && local_submission_date.as_deref() == Some(fields.date.as_str()))
    {
        return false;
    }
    let leading = Regex::new(r"(?i)(INR|USD|EUR|GBP|Rs\.?|₹|\$|€|£)\s*([0-9][0-9,]*(?:\.[0-9]+)?)\s*(?:has been\s+|is\s+)?(debited|credited|spent|paid|received)").expect("amount pattern");
    let trailing = Regex::new(r"(?i)\b(debited|credited|spent|paid|received)\s+(?:(?:by|of)\s+)?(?:(INR|USD|EUR|GBP|Rs\.?|₹|\$|€|£)\s*)?([0-9][0-9,]*(?:\.[0-9]+)?)").expect("amount pattern");
    let mut paid = Vec::new();
    for captures in leading.captures_iter(source) {
        paid.push((
            captures[2].to_owned(),
            Some(captures[1].to_owned()),
            captures[3].to_owned(),
        ));
    }
    for captures in trailing.captures_iter(source) {
        paid.push((
            captures[3].to_owned(),
            captures.get(2).map(|m| m.as_str().to_owned()),
            captures[1].to_owned(),
        ));
    }
    if paid.len() != 1 {
        return false;
    }
    let (paid_amount, currency, verb) = &paid[0];
    if Decimal::from_str(&paid_amount.replace(',', "")).ok() != Some(amount) {
        return false;
    }
    let direction = if ["credited", "received"].contains(&verb.to_lowercase().as_str()) {
        "credit"
    } else {
        "debit"
    };
    if direction != fields.direction {
        return false;
    }
    match currency.as_deref().map(str::to_uppercase).as_deref() {
        Some("RS" | "RS." | "₹") => fields.currency == "INR",
        Some("$") => fields.currency == "USD",
        Some("€") => fields.currency == "EUR",
        Some("£") => fields.currency == "GBP",
        Some(currency) => fields.currency == currency,
        None => capture.input.input_kind == InputKind::TypedNote,
    }
}

fn account_alias_in_source(source: &str, alias: &str) -> bool {
    let alias = regex::escape(alias.trim());
    if alias.is_empty() {
        return false;
    }
    let pattern = format!(
        r"(?i)(?:\baccount|\bacct|\ba/c|\bcard|\bfrom|\bvia|\busing)\s*(?:(?:ending\s*(?:in)?|number|no\.?)\s*)?[:#-]?\s*[*x]*{}(?:$|[^a-z0-9])",
        alias
    );
    Regex::new(&pattern).is_ok_and(|pattern| pattern.is_match(source))
}

/// Only remove conventional masking from a numeric account suffix. Named aliases
/// remain exact, and callers still reject suffixes mapped to multiple accounts.
pub fn account_hint_matches(alias: &str, hint: &str) -> bool {
    fn normalized(value: &str) -> &str {
        let value = value.trim();
        let suffix = value.trim_start_matches(['x', 'X', '*']);
        if suffix.len() >= 4 && suffix.bytes().all(|b| b.is_ascii_digit()) {
            suffix
        } else {
            value
        }
    }
    normalized(alias).eq_ignore_ascii_case(normalized(hint))
}

#[cfg(test)]
mod tests {
    use super::account_hint_matches;

    #[test]
    fn masked_suffixes_match_without_fuzzy_account_selection() {
        for hint in ["XX1234", "xxxx1234", "****1234", "1234"] {
            assert!(account_hint_matches("1234", hint));
        }
        assert!(account_hint_matches("XX1234", "1234"));
        for hint in ["991234", "bank1234", "XX123", "1235", "1234 extra"] {
            assert!(!account_hint_matches("1234", hint));
        }
        assert!(account_hint_matches("My bank", "my BANK"));
        assert!(!account_hint_matches("Xbank", "bank"));
    }
}
