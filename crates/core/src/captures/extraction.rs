use super::CaptureInput;
use crate::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExtractedEvent {
    pub account_hint: Option<String>,
    pub amount: Option<String>,
    pub currency: Option<String>,
    pub date: Option<String>,
    pub transaction_date: Option<String>,
    pub booking_date: Option<String>,
    pub value_date: Option<String>,
    pub direction: Option<String>,
    pub merchant: Option<String>,
    pub reference: Option<String>,
    pub kind: String,
    pub state: String,
    pub source_text: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Extraction {
    pub events: Vec<ExtractedEvent>,
    #[serde(skip)]
    pub usage: Option<TokenUsage>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

#[async_trait]
pub trait CaptureExtractor: Send + Sync {
    async fn extract(
        &self,
        input: &CaptureInput,
        provider: &str,
        model: &str,
    ) -> Result<Extraction>;
}

/// Bump when the extraction prompt or schema changes; automation needs requalification.
pub const EXTRACTION_VERSION: &str = "bedrock-luna-capture-v3";
