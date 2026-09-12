use crate::provider_model::ProviderApiError;

pub const DEFAULT_URL: &str = "https://bedrock-mantle.us-east-1.api.aws/openai/v1";

/// Validate before attaching an AWS bearer credential to any request.
pub fn validate_endpoint(value: &str) -> Result<(), ProviderApiError> {
    let invalid = || ProviderApiError::InvalidInput {
        message: "Use a supported AWS Bedrock Mantle HTTPS endpoint ending in /openai/v1".into(),
    };
    let url = reqwest::Url::parse(value).map_err(|_| invalid())?;
    #[cfg(feature = "test-utils")]
    if url.scheme() == "http" && url.host_str() == Some("127.0.0.1") {
        return Ok(());
    }
    if url.scheme() != "https"
        || ![
            "bedrock-mantle.us-east-1.api.aws",
            "bedrock-mantle.us-east-2.api.aws",
            "bedrock-mantle.us-west-2.api.aws",
        ]
        .contains(&url.host_str().unwrap_or_default())
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path().trim_end_matches('/') != "/openai/v1"
    {
        return Err(invalid());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credentials_only_go_to_supported_mantle_endpoints() {
        assert!(validate_endpoint(DEFAULT_URL).is_ok());
        for url in [
            "https://api.openai.com/v1",
            "https://bedrock-mantle.us-east-1.api.aws.evil.example/openai/v1",
            "http://bedrock-mantle.us-east-1.api.aws/openai/v1",
            "https://bedrock-mantle.us-east-1.api.aws/v1",
        ] {
            assert!(validate_endpoint(url).is_err());
        }
    }
}
