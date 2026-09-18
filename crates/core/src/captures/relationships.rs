use super::{Capture, CaptureService};
use crate::Result;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingLink {
    pub candidate_id: String,
    pub other_activity_id: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingRefund {
    pub review_id: String,
    pub refund_id: String,
    pub original_id: String,
}
impl CaptureService {
    pub(super) fn validate_refund_link(&self, refund_id: &str, original_id: &str) -> Result<()> {
        let refund = self.activities.get_activity(refund_id)?;
        let original = self.activities.get_activity(original_id)?;
        let valid = refund.activity_type == "CREDIT"
            && original.activity_type == "WITHDRAWAL"
            && refund.account_id == original.account_id
            && refund.currency == original.currency
            && original.activity_date.date_naive() <= refund.activity_date.date_naive()
            && refund
                .amount
                .zip(original.amount)
                .is_some_and(|(r, o)| r.abs() > rust_decimal::Decimal::ZERO && r.abs() <= o.abs());
        if !valid {
            return Err(crate::errors::ValidationError::InvalidInput(
                "Choose an earlier debit in the same account and currency that covers this refund"
                    .into(),
            )
            .into());
        }
        if refund
            .metadata
            .as_ref()
            .and_then(|m| m["refund"]["original_activity_id"].as_str())
            .is_some_and(|id| id != original_id)
        {
            return Err(crate::Error::ConstraintViolation(
                "Refund already linked to another payment".into(),
            ));
        }
        Ok(())
    }
    pub(super) async fn finish_refund(&self, mut capture: Capture) -> Result<Capture> {
        let pending = capture
            .pending_refund
            .clone()
            .expect("persisted refund link");
        let result=async {
            self.validate_refund_link(&pending.refund_id,&pending.original_id)?;
            let refund=self.activities.get_activity(&pending.refund_id)?;
            let mut metadata=refund.metadata.clone().unwrap_or_else(||serde_json::json!({}));
            if metadata["refund"]["original_activity_id"]!=pending.original_id {
                metadata["refund"]=serde_json::json!({"original_activity_id":pending.original_id});
                let update=serde_json::from_value(serde_json::json!({"id":refund.id,"accountId":refund.account_id,"activityType":refund.activity_type,"activityDate":refund.activity_date.to_rfc3339(),"currency":refund.currency,"metadata":metadata.to_string()}))?;
                self.activities.update_activity(update).await?;
            }
            Ok::<(),crate::Error>(())
        }.await;
        if result.is_ok() {
            if let Some(review) = capture
                .reviews
                .iter_mut()
                .find(|r| r.id == pending.review_id)
            {
                review.status = "resolved".into();
            }
            if let Some(record) = capture.pending_resolution.take() {
                capture.resolution_history.push(record);
            }
        } else {
            capture.pending_resolution = None;
        }
        capture.pending_refund = None;
        capture.lease_until = None;
        capture.status = if capture.reviews.iter().any(|r| r.status == "open") {
            "needs_review"
        } else {
            "complete"
        }
        .into();
        let version = capture.version;
        let saved = self.repository.save(capture, version).await?;
        result?;
        Ok(saved)
    }
    pub(super) fn prepare_transfer_link(&self, capture: &mut Capture, index: usize) -> Result<()> {
        let candidate = &capture.candidates[index];
        if candidate.status != "posted"
            || !["transfer", "card_payment"].contains(&candidate.fields.kind.as_str())
        {
            return Ok(());
        }
        let fields = &candidate.fields;
        let Some(reference) = fields
            .reference
            .as_deref()
            .map(str::trim)
            .filter(|r| r.len() >= 6)
        else {
            return Ok(());
        };
        let opposite = if fields.direction == "debit" {
            "TRANSFER_IN"
        } else {
            "TRANSFER_OUT"
        };
        let amount = rust_decimal::Decimal::from_str(&fields.amount).ok();
        let matches: Vec<_> = self
            .activities
            .get_activities()?
            .into_iter()
            .filter(|a| {
                a.id != candidate.activity_id
                    && a.account_id != fields.account_id
                    && a.activity_type == opposite
                    && a.source_record_id.as_deref().map(str::trim) == Some(reference)
                    && a.currency == fields.currency
                    && a.activity_date.date_naive().to_string() == fields.date
                    && a.amount.map(|v| v.abs()) == amount
            })
            .collect();
        if matches.len() == 1 {
            capture.pending_link = Some(PendingLink {
                candidate_id: candidate.id.clone(),
                other_activity_id: matches[0].id.clone(),
            });
        }
        Ok(())
    }
    pub(super) async fn finish_link(&self, mut capture: Capture) -> Result<Capture> {
        let pending = capture
            .pending_link
            .as_ref()
            .expect("link persisted")
            .clone();
        let candidate = capture
            .candidates
            .iter()
            .find(|c| c.id == pending.candidate_id)
            .expect("link candidate");
        let activity_id = candidate.activity_id.clone();
        let a = self.activities.get_activity(&activity_id)?;
        let b = self.activities.get_activity(&pending.other_activity_id)?;
        let already_linked = a.source_group_id.is_some() && a.source_group_id == b.source_group_id;
        let linked = already_linked
            || self
                .activities
                .link_transfer_activities(activity_id.clone(), pending.other_activity_id.clone())
                .await
                .is_ok();
        if linked {
            for review in &mut capture.reviews {
                if review.reason == "counterpart_required"
                    && review.activity_id.as_deref() == Some(activity_id.as_str())
                {
                    review.status = "resolved".into();
                }
            }
            for mut other in self.repository.list()?.into_iter().filter(|c| {
                c.id != capture.id
                    && c.reviews.iter().any(|r| {
                        r.reason == "counterpart_required"
                            && r.status == "open"
                            && r.activity_id.as_deref() == Some(pending.other_activity_id.as_str())
                    })
            }) {
                for review in &mut other.reviews {
                    if review.reason == "counterpart_required"
                        && review.activity_id.as_deref() == Some(pending.other_activity_id.as_str())
                    {
                        review.status = "resolved".into();
                    }
                }
                if other.status != "processing" && other.reviews.iter().all(|r| r.status != "open")
                {
                    other.status = "complete".into();
                }
                let version = other.version;
                self.repository.save(other, version).await?;
            }
        }
        capture.pending_link = None;
        capture.status = if capture.automatic_batch {
            "processing"
        } else if capture.reviews.iter().any(|r| r.status == "open") {
            "needs_review"
        } else {
            "complete"
        }
        .into();
        capture.lease_until = if capture.automatic_batch {
            Some(self.now() + 120)
        } else {
            None
        };
        let version = capture.version;
        self.repository.save(capture, version).await
    }
}
