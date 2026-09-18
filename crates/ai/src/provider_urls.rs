/// Environment configuration for self-hosted Bedrock deployments.
pub(crate) fn bedrock_env_url(provider_id: &str) -> Option<String> {
    if provider_id != "bedrock" {
        return None;
    }
    let region = std::env::var("AWS_REGION").ok()?;
    Some(format!(
        "https://bedrock-mantle.{}.api.aws/v1",
        region.trim()
    ))
}

pub(crate) fn bedrock_env_key(provider_id: &str) -> Option<String> {
    if provider_id != "bedrock" {
        return None;
    }
    std::env::var("AWS_BEARER_TOKEN_BEDROCK")
        .ok()
        .filter(|key| !key.trim().is_empty())
}

/// Bedrock credentials must only be sent to a regional AWS Mantle endpoint.
/// There is deliberately no default region or OpenAI-hosted fallback.
pub(crate) fn validate_bedrock_url(value: Option<&str>) -> Result<String, String> {
    let invalid =
        || "Choose an AWS region in Amazon Bedrock settings before connecting.".to_string();
    let url = reqwest::Url::parse(value.ok_or_else(invalid)?).map_err(|_| invalid())?;
    let host = url.host_str().ok_or_else(invalid)?;
    // Normalize URLs saved by the first provider release without changing region.
    let (region, path) = if let Some(region) = host
        .strip_prefix("bedrock-mantle.")
        .and_then(|s| s.strip_suffix(".api.aws"))
    {
        (region, "/v1")
    } else {
        (
            host.strip_prefix("bedrock-runtime.")
                .and_then(|s| s.strip_suffix(".amazonaws.com"))
                .ok_or_else(invalid)?,
            "/openai/v1",
        )
    };
    let parts: Vec<_> = region.split('-').collect();
    if parts.len() < 3
        || parts.len() > 4
        || parts[..parts.len() - 1]
            .iter()
            .any(|p| p.is_empty() || !p.bytes().all(|c| c.is_ascii_lowercase()))
        || !parts.last().unwrap().bytes().all(|c| c.is_ascii_digit())
        || parts.last().unwrap().is_empty()
        || url.scheme() != "https"
        || !url.username().is_empty()
        || url.password().is_some()
        || url.port().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.path().trim_end_matches('/') != path
    {
        return Err(invalid());
    }
    Ok(format!("https://bedrock-mantle.{region}.api.aws/v1"))
}

pub(crate) fn ensure_openai_v1_base_url(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    if base.ends_with("/v1") {
        base.to_string()
    } else {
        format!("{}/v1", base)
    }
}

pub(crate) fn openai_compatible_models_url(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    let base = base.strip_suffix("/v1").unwrap_or(base);
    format!("{}/v1/models", base)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ensure_openai_v1_base_url_accepts_bare_or_v1_url() {
        let cases = [
            ("http://localhost:8080", "http://localhost:8080/v1"),
            ("http://localhost:8080/", "http://localhost:8080/v1"),
            ("http://localhost:8080/v1", "http://localhost:8080/v1"),
            ("http://localhost:8080/v1/", "http://localhost:8080/v1"),
            ("https://api.openai.com", "https://api.openai.com/v1"),
            (
                "https://api.groq.com/openai",
                "https://api.groq.com/openai/v1",
            ),
            ("https://openrouter.ai/api", "https://openrouter.ai/api/v1"),
        ];

        for (input, expected) in cases {
            assert_eq!(ensure_openai_v1_base_url(input), expected, "input: {input}");
        }
    }

    #[test]
    fn openai_compatible_models_url_accepts_bare_or_v1_url() {
        let cases = [
            ("http://localhost:8080", "http://localhost:8080/v1/models"),
            ("http://localhost:8080/", "http://localhost:8080/v1/models"),
            (
                "http://localhost:8080/v1",
                "http://localhost:8080/v1/models",
            ),
            (
                "http://localhost:8080/v1/",
                "http://localhost:8080/v1/models",
            ),
            (
                "https://api.groq.com/openai",
                "https://api.groq.com/openai/v1/models",
            ),
            (
                "https://openrouter.ai/api/v1",
                "https://openrouter.ai/api/v1/models",
            ),
        ];

        for (input, expected) in cases {
            assert_eq!(
                openai_compatible_models_url(input),
                expected,
                "input: {input}"
            );
        }
    }
}

#[cfg(test)]
mod bedrock_tests {
    use super::*;

    #[test]
    fn accepts_regional_mantle_endpoints() {
        for region in ["us-east-1", "ap-south-1", "eu-central-1", "us-gov-west-1"] {
            let url = format!("https://bedrock-mantle.{region}.api.aws/v1");
            assert_eq!(validate_bedrock_url(Some(&format!("{url}/"))).unwrap(), url);
        }
    }

    #[test]
    fn rejects_missing_region_and_credential_leaking_endpoints() {
        assert!(validate_bedrock_url(None).is_err());
        for url in [
            "",
            "https://api.openai.com/v1",
            "http://bedrock-mantle.us-east-1.api.aws/v1",
            "https://bedrock-mantle.us-east-1.api.aws.attacker.test/v1",
            "https://bedrock-mantle.us-east-1.api.aws/openai/v1",
            "https://bedrock-mantle.us-east-1.api.aws/v1?token=secret",
            "http://bedrock-runtime.us-east-1.amazonaws.com/openai/v1",
            "https://bedrock-runtime.us-east-1.amazonaws.com.attacker.test/openai/v1",
            "https://bedrock-runtime.us-east-1.amazonaws.com:9000/openai/v1",
            "https://user:secret@bedrock-runtime.us-east-1.amazonaws.com/openai/v1",
            "https://bedrock-runtime.us-east-1.amazonaws.com/openai/v1?token=secret",
            "https://bedrock-runtime.us-east-1.amazonaws.com/openai/v1#fragment",
            "https://bedrock-runtime.us-east-1.amazonaws.com/v1",
            "https://bedrock-runtime.us-east-.amazonaws.com/openai/v1",
        ] {
            let error = validate_bedrock_url(Some(url)).unwrap_err();
            assert!(!error.contains("secret"));
        }
    }
}

#[cfg(test)]
mod mantle_regression_tests {
    use super::*;
    #[test]
    fn saved_runtime_endpoint_routes_discovery_and_chat_to_mantle() {
        let url = validate_bedrock_url(Some(
            "https://bedrock-runtime.us-east-1.amazonaws.com/openai/v1",
        ))
        .unwrap();
        assert_eq!(url, "https://bedrock-mantle.us-east-1.api.aws/v1");
        assert_eq!(
            openai_compatible_models_url(&url),
            "https://bedrock-mantle.us-east-1.api.aws/v1/models"
        );
        assert_eq!(ensure_openai_v1_base_url(&url), url);
    }
}
