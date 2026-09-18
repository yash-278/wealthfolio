//! Capture transport uses the configured Bedrock region and shared URL validation.
use crate::provider_model::ProviderApiError;

pub const DEFAULT_URL: &str = "https://bedrock-mantle.us-east-1.api.aws/v1";

pub fn capture_endpoint(value: &str) -> Result<String, ProviderApiError> {
    let base = crate::provider_urls::validate_bedrock_url(Some(value))
        .map_err(|message| ProviderApiError::InvalidInput { message })?;
    let mut url = reqwest::Url::parse(&base).expect("validated URL");
    #[cfg(feature = "test-utils")]
    if url.host_str() == Some("127.0.0.1") {
        return Ok(format!("{}/chat/completions", base.trim_end_matches('/')));
    }
    url.set_path("/openai/v1/chat/completions");
    Ok(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capture_preserves_configured_aws_region() {
        assert_eq!(
            capture_endpoint(DEFAULT_URL).unwrap(),
            "https://bedrock-mantle.us-east-1.api.aws/openai/v1/chat/completions"
        );
        assert_eq!(
            capture_endpoint("https://bedrock-mantle.us-west-2.api.aws/v1").unwrap(),
            "https://bedrock-mantle.us-west-2.api.aws/openai/v1/chat/completions"
        );
        assert!(capture_endpoint("https://api.openai.com/v1").is_err());
    }
}
