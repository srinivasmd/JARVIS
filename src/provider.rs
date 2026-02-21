use std::process::Command;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub prompt: String,
}

#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub text: String,
    pub model: String,
}

pub trait LlmProvider: Send + Sync {
    fn generate(&self, req: LlmRequest) -> Result<LlmResponse, String>;
}

pub struct OpenAiProvider {
    pub api_key: String,
    pub model: String,
    pub api_base: String,
}

impl LlmProvider for OpenAiProvider {
    fn generate(&self, req: LlmRequest) -> Result<LlmResponse, String> {
        let endpoint = OpenAiEndpoint::parse(&self.api_base);
        let body = format!(
            "{{\"model\":\"{}\",\"messages\":[{{\"role\":\"user\",\"content\":\"{}\"}}]}}",
            self.model,
            req.prompt.replace('"', "'")
        );

        let response = Command::new("curl")
            .arg("--silent")
            .arg("--show-error")
            .arg("--fail")
            .arg("--max-time")
            .arg("45")
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-H")
            .arg(format!("Authorization: Bearer {}", self.api_key))
            .arg("-d")
            .arg(body)
            .arg(endpoint.chat_completions_url())
            .output()
            .map_err(|e| format!("failed to launch curl: {e}"))?;

        if !response.status.success() {
            return Err(format!(
                "provider request failed: {}",
                String::from_utf8_lossy(&response.stderr).trim()
            ));
        }

        let out = String::from_utf8_lossy(&response.stdout).to_string();
        let content = extract_message_content(&out).unwrap_or_else(|| out.clone());
        Ok(LlmResponse {
            text: content,
            model: self.model.clone(),
        })
    }
}

struct OpenAiEndpoint {
    base_url: String,
}

impl OpenAiEndpoint {
    fn parse(input: &str) -> Self {
        Self {
            base_url: input.trim_end_matches('/').to_string(),
        }
    }

    fn chat_completions_url(&self) -> String {
        format!("{}/chat/completions", self.base_url)
    }
}

fn extract_message_content(payload: &str) -> Option<String> {
    let (_, after_content_key) = payload.split_once("\"content\":")?;
    let start_quote = after_content_key.find('"')? + 1;
    let rest = &after_content_key[start_quote..];
    let end_quote = rest.find('"')?;
    Some(rest[..end_quote].to_string())
}

pub struct FallbackProvider {
    pub primary: Arc<dyn LlmProvider>,
    pub fallback: Arc<dyn LlmProvider>,
}

impl LlmProvider for FallbackProvider {
    fn generate(&self, req: LlmRequest) -> Result<LlmResponse, String> {
        match self.primary.generate(req.clone()) {
            Ok(response) => Ok(response),
            Err(primary_error) => self.fallback.generate(req).map_err(|fallback_error| {
                format!("primary={primary_error}; fallback={fallback_error}")
            }),
        }
    }
}

pub struct MockProvider;
impl LlmProvider for MockProvider {
    fn generate(&self, req: LlmRequest) -> Result<LlmResponse, String> {
        Ok(LlmResponse {
            text: format!("echo:{}", req.prompt),
            model: "mock".into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct BrokenProvider;
    impl LlmProvider for BrokenProvider {
        fn generate(&self, _req: LlmRequest) -> Result<LlmResponse, String> {
            Err("boom".into())
        }
    }

    #[test]
    fn parses_openai_endpoint_with_custom_path() {
        let endpoint = OpenAiEndpoint::parse("http://localhost:1234/custom/v1/");
        assert_eq!(
            endpoint.chat_completions_url(),
            "http://localhost:1234/custom/v1/chat/completions"
        );
    }

    #[test]
    fn falls_back_when_primary_errors() {
        let provider = FallbackProvider {
            primary: Arc::new(BrokenProvider),
            fallback: Arc::new(MockProvider),
        };
        let response = provider
            .generate(LlmRequest {
                prompt: "hello".into(),
            })
            .expect("fallback should handle request");
        assert_eq!(response.model, "mock");
    }
    #[test]
    #[ignore = "requires external network and valid OPENAI-compatible credentials"]
    fn real_provider_smoke_test_from_env() {
        let api_key = std::env::var("LLM_PRIMARY_API_KEY").expect("LLM_PRIMARY_API_KEY missing");
        let model = std::env::var("LLM_PRIMARY_MODEL").expect("LLM_PRIMARY_MODEL missing");
        let api_base = std::env::var("LLM_PRIMARY_API_BASE").expect("LLM_PRIMARY_API_BASE missing");

        let provider = OpenAiProvider {
            api_key,
            model,
            api_base,
        };

        let response = provider
            .generate(LlmRequest {
                prompt: "Reply exactly: ok".into(),
            })
            .expect("real provider request failed");

        assert!(!response.text.trim().is_empty(), "empty response text");
    }
}
