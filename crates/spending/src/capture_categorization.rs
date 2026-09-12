//! Quick Add uses the existing category assignment and rule stores.
use crate::{
    cash_activities::CashActivityService,
    categorization_rules::{
        match_rules, CategorizationRulesService, NewCategorizationRule, RuleMatchType,
    },
};
use std::{collections::HashSet, str::FromStr, sync::Arc};
use wealthfolio_core::{
    captures::{CaptureCategorizer, Fields},
    Error, Result,
};

pub struct CaptureCategorization {
    cash: Arc<CashActivityService>,
    rules: Arc<CategorizationRulesService>,
}
impl CaptureCategorization {
    pub fn new(cash: Arc<CashActivityService>, rules: Arc<CategorizationRulesService>) -> Self {
        Self { cash, rules }
    }
}
fn failure(error: impl std::fmt::Display) -> Error {
    Error::Validation(wealthfolio_core::errors::ValidationError::InvalidInput(
        error.to_string(),
    ))
}
#[async_trait::async_trait]
impl CaptureCategorizer for CaptureCategorization {
    async fn assign(&self, activity_id: &str, taxonomy_id: &str, category_id: &str) -> Result<()> {
        self.cash
            .assign_category(activity_id, taxonomy_id, category_id)
            .await
            .map(|_| ())
            .map_err(failure)
    }
    async fn learn(&self, fields: &Fields, taxonomy: &str, category: &str) -> Result<()> {
        let Some(merchant) = fields
            .merchant
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        else {
            return Ok(());
        };
        let id = wealthfolio_core::captures::merchant_rule_id(fields);
        self.rules
            .upsert(NewCategorizationRule {
                id: Some(id),
                name: format!("Quick Add: {merchant}"),
                pattern: merchant.into(),
                match_type: RuleMatchType::Exact,
                taxonomy_id: Some(taxonomy.into()),
                category_id: Some(category.into()),
                activity_type: Some(fields.activity_type().into()),
                amount_op: None,
                amount_value: None,
                amount_value2: None,
                priority: i32::MIN,
                is_global: false,
                account_id: Some(fields.account_id.clone()),
                preset_id: None,
                preset_rule_key: None,
                preset_version: None,
            })
            .await
            .map_err(failure)?;
        Ok(())
    }
    async fn suggest(&self, fields: &Fields) -> Result<Option<(String, String)>> {
        let Some(merchant) = fields.merchant.as_deref() else {
            return Ok(None);
        };
        let rules = self.rules.list().await.map_err(failure)?;
        let kind = fields.activity_type();
        let amount = rust_decimal::Decimal::from_str(&fields.amount).ok();
        let matches: Vec<_> = rules
            .iter()
            .filter(|rule| {
                match_rules(
                    std::slice::from_ref(*rule),
                    merchant,
                    kind,
                    &fields.account_id,
                    amount,
                )
                .is_some()
            })
            .collect();
        let explicit = matches
            .iter()
            .any(|r| !r.id.starts_with("quick-add-learned:"));
        let eligible: Vec<_> = matches
            .into_iter()
            .filter(|r| !explicit || !r.id.starts_with("quick-add-learned:"))
            .collect();
        let priority = eligible.iter().map(|r| r.priority).max();
        let categories: HashSet<_> = eligible
            .into_iter()
            .filter(|r| Some(r.priority) == priority)
            .filter_map(|r| r.taxonomy_id.clone().zip(r.category_id.clone()))
            .collect();
        Ok(if categories.len() == 1 {
            categories.into_iter().next()
        } else {
            None
        })
    }
}
