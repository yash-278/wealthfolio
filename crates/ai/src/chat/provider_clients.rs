//! Provider-specific rig client constructors + error remapping.
//!
//! Each function builds a configured rig client for one provider (Anthropic,
//! Gemini, Groq, OpenAI, OpenRouter, Ollama). Kept here so the streaming agent
//! builder doesn't have to mix provider plumbing with response handling.

use log::debug;
use reqwest::Client as HttpClient;
use rig::{
    client::Nothing,
    providers::{anthropic, gemini, groq, ollama, openai, openrouter},
};
use std::time::Duration;

use crate::error::AiError;
use crate::provider_urls::{ensure_openai_v1_base_url, validate_bedrock_url};

pub(crate) fn create_anthropic_client(
    api_key: Option<String>,
    provider_id: &str,
    provider_url: Option<String>,
) -> Result<anthropic::Client<HttpClient>, AiError> {
    let key = api_key.ok_or_else(|| AiError::MissingApiKey(provider_id.to_string()))?;
    let mut builder = anthropic::Client::builder().api_key(&key);
    if let Some(url) = provider_url {
        builder = builder.base_url(&url);
    }
    builder
        .build()
        .map_err(|e| AiError::Provider(e.to_string()))
}

pub(crate) fn create_gemini_client(
    api_key: Option<String>,
    provider_id: &str,
    provider_url: Option<String>,
) -> Result<gemini::Client<HttpClient>, AiError> {
    let key = api_key.ok_or_else(|| AiError::MissingApiKey(provider_id.to_string()))?;
    let mut builder = gemini::Client::builder().api_key(&key);
    if let Some(url) = provider_url {
        builder = builder.base_url(&url);
    }
    builder
        .build()
        .map_err(|e| AiError::Provider(e.to_string()))
}

pub(crate) fn create_groq_client(
    api_key: Option<String>,
    provider_id: &str,
    provider_url: Option<String>,
) -> Result<groq::Client<HttpClient>, AiError> {
    let key = api_key.ok_or_else(|| AiError::MissingApiKey(provider_id.to_string()))?;
    let mut builder = groq::Client::builder().api_key(&key);
    if let Some(url) = provider_url {
        let normalized = ensure_openai_v1_base_url(&url);
        builder = builder.base_url(&normalized);
    }
    builder
        .build()
        .map_err(|e| AiError::Provider(e.to_string()))
}

/// Create OpenAI client using Completions API (not Responses API).
/// Responses API has issues with reasoning items in multi-turn conversations.
/// See: <https://community.openai.com/t/error-badrequesterror-400-item-of-type-reasoning-was-provided-without-its-required-following-item/1303809>
pub(crate) fn create_openai_client(
    api_key: Option<String>,
    provider_id: &str,
    provider_url: Option<String>,
) -> Result<openai::CompletionsClient<HttpClient>, AiError> {
    let key = api_key.ok_or_else(|| AiError::MissingApiKey(provider_id.to_string()))?;
    let mut builder = openai::CompletionsClient::builder().api_key(&key);
    if let Some(url) = provider_url {
        let normalized = ensure_openai_v1_base_url(&url);
        builder = builder.base_url(&normalized);
    }
    builder
        .build()
        .map_err(|e| AiError::Provider(e.to_string()))
}

pub(crate) fn create_bedrock_client(
    api_key: Option<String>,
    provider_url: Option<String>,
) -> Result<openai::CompletionsClient<HttpClient>, AiError> {
    let url = validate_bedrock_url(provider_url.as_deref()).map_err(AiError::InvalidInput)?;
    create_openai_client(api_key, "bedrock", Some(url))
}

pub(crate) fn create_openrouter_client(
    api_key: Option<String>,
    provider_id: &str,
    provider_url: Option<String>,
) -> Result<openrouter::Client<HttpClient>, AiError> {
    let key = api_key.ok_or_else(|| AiError::MissingApiKey(provider_id.to_string()))?;
    let mut builder = openrouter::Client::builder().api_key(&key);
    if let Some(url) = provider_url {
        let normalized = ensure_openai_v1_base_url(&url);
        builder = builder.base_url(&normalized);
    }
    builder
        .build()
        .map_err(|e| AiError::Provider(e.to_string()))
}

pub(crate) fn create_ollama_client(
    provider_url: Option<String>,
) -> Result<ollama::Client<HttpClient>, AiError> {
    let mut builder = ollama::Client::builder().api_key(Nothing);
    if let Some(url) = provider_url {
        let normalized = url.trim_end_matches('/').trim_end_matches("/v1");
        builder = builder.base_url(normalized);
    }
    builder
        .build()
        .map_err(|e| AiError::Provider(e.to_string()))
}

/// Map low-level provider errors to clearer actionable messages.
pub(super) fn remap_provider_error(provider_id: &str, model_id: &str, error: AiError) -> AiError {
    match error {
        AiError::Provider(msg)
            if provider_id == "ollama" && msg.contains("missing field `model`") =>
        {
            AiError::Provider(format!(
                "Ollama returned an error payload for model '{}'. \
                Common causes: model not installed, context too large, or insufficient memory. \
                Check `ollama list` and Ollama logs. Original error: {}",
                model_id, msg
            ))
        }
        other => other,
    }
}

pub(super) fn ollama_model_matches(candidate: &str, selected: &str) -> bool {
    candidate == selected
        || candidate.trim_end_matches(":latest") == selected.trim_end_matches(":latest")
}

/// Validate selected Ollama model when `/api/tags` is reachable.
///
/// This is best-effort:
/// - If tags endpoint is unavailable/unparseable, we skip validation and continue.
/// - If tags are available and model is missing, we return a clear invalid-input error.
pub(super) async fn validate_ollama_model_if_possible(
    provider_url: Option<&str>,
    model_id: &str,
) -> Result<(), AiError> {
    let base = provider_url.unwrap_or("http://localhost:11434");
    let normalized = base.trim_end_matches('/');
    let tags_url = if normalized.ends_with("/v1") {
        format!("{}/api/tags", normalized.trim_end_matches("/v1"))
    } else {
        format!("{}/api/tags", normalized)
    };

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(4))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            debug!(
                "Skipping Ollama model preflight (client build failed): {}",
                e
            );
            return Ok(());
        }
    };

    let response = match client.get(&tags_url).send().await {
        Ok(r) => r,
        Err(e) => {
            debug!("Skipping Ollama model preflight (tags fetch failed): {}", e);
            return Ok(());
        }
    };

    if !response.status().is_success() {
        debug!(
            "Skipping Ollama model preflight (tags status {} at {})",
            response.status(),
            tags_url
        );
        return Ok(());
    }

    let payload: serde_json::Value = match response.json().await {
        Ok(v) => v,
        Err(e) => {
            debug!("Skipping Ollama model preflight (invalid tags JSON): {}", e);
            return Ok(());
        }
    };

    let available: Vec<String> = payload
        .get("models")
        .and_then(|v| v.as_array())
        .map(|models| {
            models
                .iter()
                .filter_map(|m| m.get("name").and_then(|v| v.as_str()))
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default();

    if available.is_empty() {
        debug!("Skipping Ollama model preflight (no models in tags response)");
        return Ok(());
    }

    if available
        .iter()
        .any(|candidate| ollama_model_matches(candidate, model_id))
    {
        return Ok(());
    }

    let preview = available
        .iter()
        .take(5)
        .cloned()
        .collect::<Vec<_>>()
        .join(", ");
    Err(AiError::InvalidInput(format!(
        "Ollama model '{}' is not available. Install it with `ollama pull {}` or select an installed model in AI Providers settings. Available models: {}",
        model_id, model_id, preview
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_compatible_clients_normalize_base_url() {
        let openai = create_openai_client(
            Some("test-key".to_string()),
            "openai",
            Some("http://localhost:8080/".to_string()),
        )
        .expect("openai client");
        assert_eq!(openai.base_url(), "http://localhost:8080/v1");

        let groq = create_groq_client(
            Some("test-key".to_string()),
            "groq",
            Some("https://api.groq.com/openai".to_string()),
        )
        .expect("groq client");
        assert_eq!(groq.base_url(), "https://api.groq.com/openai/v1");

        let openrouter = create_openrouter_client(
            Some("test-key".to_string()),
            "openrouter",
            Some("https://openrouter.ai/api/v1/".to_string()),
        )
        .expect("openrouter client");
        assert_eq!(openrouter.base_url(), "https://openrouter.ai/api/v1");
    }

    #[test]
    fn test_ollama_model_match_without_latest_suffix() {
        assert!(ollama_model_matches("ministral-3:latest", "ministral-3"));
        assert!(ollama_model_matches("ministral-3", "ministral-3:latest"));
        assert!(!ollama_model_matches("qwen3:8b", "ministral-3"));
    }

    #[test]
    fn test_remap_provider_error_for_ollama_json_error() {
        let input = AiError::Provider(
            "CompletionError: JsonError: missing field `model` at line 1 column 44".to_string(),
        );
        let remapped = remap_provider_error("ollama", "ministral-3", input);
        match remapped {
            AiError::Provider(msg) => {
                assert!(msg.contains("Ollama returned an error payload"));
                assert!(msg.contains("ministral-3"));
            }
            _ => panic!("expected provider error"),
        }
    }
}

#[cfg(test)]
mod bedrock_tests {
    use super::*;
    use futures::StreamExt;
    use rig::{
        client::CompletionClient, completion::CompletionModel, streaming::StreamedAssistantContent,
    };
    use std::io::{Read, Write};

    #[test]
    fn bedrock_never_uses_the_openai_default_endpoint() {
        assert!(create_bedrock_client(Some("synthetic".into()), None).is_err());
        assert!(create_bedrock_client(
            Some("synthetic".into()),
            Some("https://api.openai.com/v1".into())
        )
        .is_err());
        let url = "https://bedrock-mantle.us-east-1.api.aws/v1";
        assert!(create_bedrock_client(None, Some(url.into())).is_err());
        let client = create_bedrock_client(Some("synthetic".into()), Some(url.into())).unwrap();
        assert_eq!(client.base_url(), url);
    }

    /// Exercise the same Completions transport used by Bedrock against synthetic SSE.
    /// Production URL validation remains enabled; only this test constructs a local client.
    #[tokio::test]
    async fn bedrock_transport_preserves_streamed_text_and_tool_arguments() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut bytes = Vec::new();
            let mut buf = [0; 4096];
            loop {
                let n = socket.read(&mut buf).unwrap();
                assert!(n > 0);
                bytes.extend_from_slice(&buf[..n]);
                if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                    let headers = String::from_utf8_lossy(&bytes[..end]);
                    let length: usize = headers
                        .lines()
                        .find_map(|line| {
                            line.to_ascii_lowercase()
                                .strip_prefix("content-length:")
                                .map(|v| v.trim().parse().unwrap())
                        })
                        .unwrap();
                    if bytes.len() >= end + 4 + length {
                        break;
                    }
                }
            }
            let request = String::from_utf8(bytes).unwrap();
            assert!(request.starts_with("POST /v1/chat/completions "));
            assert!(request
                .to_ascii_lowercase()
                .contains("authorization: bearer synthetic-bedrock-key"));
            let body: serde_json::Value =
                serde_json::from_str(request.split("\r\n\r\n").nth(1).unwrap()).unwrap();
            assert_eq!(body["model"], "openai.gpt-oss-20b");
            assert_eq!(body["stream"], true);
            assert_eq!(body["tools"][0]["function"]["name"], "get_accounts");
            let deltas = [
                serde_json::json!({"content":"Checking accounts."}),
                serde_json::json!({"tool_calls":[{"index":0,"id":"call_test","type":"function","function":{"name":"get_accounts","arguments":"{"}}]}),
                serde_json::json!({"tool_calls":[{"index":0,"function":{"arguments":"}"}}]}),
            ];
            let mut sse = String::new();
            for delta in deltas {
                sse.push_str(&format!("data: {}\n\n", serde_json::json!({"id":"test","object":"chat.completion.chunk","created":1,"model":"openai.gpt-oss-20b","choices":[{"index":0,"delta":delta,"finish_reason":null}]})));
            }
            sse.push_str(&format!("data: {}\n\n", serde_json::json!({"id":"test","object":"chat.completion.chunk","created":1,"model":"openai.gpt-oss-20b","choices":[{"index":0,"delta":{},"finish_reason":"tool_calls"}]})));
            write!(socket, "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", sse.len(), sse).unwrap();
        });
        let client = create_openai_client(
            Some("synthetic-bedrock-key".into()),
            "bedrock",
            Some(format!("http://{address}/v1")),
        )
        .unwrap();
        let model = client.completion_model("openai.gpt-oss-20b");
        let mut stream = tokio::time::timeout(
            Duration::from_secs(5),
            model
                .completion_request("List accounts")
                .tool(rig::completion::ToolDefinition {
                    name: "get_accounts".into(),
                    description: "List accounts".into(),
                    parameters: serde_json::json!({"type":"object", "properties":{}}),
                })
                .stream(),
        )
        .await
        .unwrap()
        .unwrap();
        let mut text = String::new();
        let mut tool = None;
        while let Some(chunk) = tokio::time::timeout(Duration::from_secs(5), stream.next())
            .await
            .unwrap()
        {
            match chunk.unwrap() {
                StreamedAssistantContent::Text(delta) => text.push_str(&delta.text),
                StreamedAssistantContent::ToolCall { tool_call, .. } => tool = Some(tool_call),
                _ => {}
            }
        }
        server.join().unwrap();
        assert_eq!(text, "Checking accounts.");
        let tool = tool.expect("completed tool call");
        assert_eq!(tool.function.name, "get_accounts");
        assert_eq!(tool.function.arguments, serde_json::json!({}));
    }
}
