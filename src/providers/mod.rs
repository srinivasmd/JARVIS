use std::sync::Arc;

use crate::{
    config::LlmConfig,
    net::{HttpRequest, HttpTransport, StdHttpTransport},
};

#[derive(Debug, Clone)]
pub struct ProviderRequest {
    pub user_input: String,
    pub context: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub output: String,
    pub model: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderError {
    Unavailable,
    InvalidConfig,
    Transport(String),
    Parse,
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderError::Unavailable => write!(f, "provider unavailable"),
            ProviderError::InvalidConfig => write!(f, "provider invalid config"),
            ProviderError::Transport(err) => write!(f, "provider transport error: {err}"),
            ProviderError::Parse => write!(f, "provider parse error"),
        }
    }
}

impl std::error::Error for ProviderError {}

pub trait LlmProvider {
    fn name(&self) -> &str;
    fn generate(&self, request: &ProviderRequest) -> Result<ProviderResponse, ProviderError>;
}

pub struct ConfiguredEchoProvider {
    provider: String,
    model: String,
}

impl ConfiguredEchoProvider {
    pub fn new(provider: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            provider: provider.into(),
            model: model.into(),
        }
    }
}

impl LlmProvider for ConfiguredEchoProvider {
    fn name(&self) -> &str {
        &self.provider
    }

    fn generate(&self, request: &ProviderRequest) -> Result<ProviderResponse, ProviderError> {
        let prefix = if request.context.is_empty() {
            ""
        } else {
            "[ctx] "
        };
        Ok(ProviderResponse {
            output: format!("{}{}", prefix, request.user_input),
            model: self.model.clone(),
        })
    }
}

pub struct OpenAiCompatibleProvider {
    model: String,
    api_base: String,
    api_key: String,
    transport: Arc<dyn HttpTransport + Send + Sync>,
}

impl OpenAiCompatibleProvider {
    pub fn new(
        model: impl Into<String>,
        api_base: impl Into<String>,
        api_key: impl Into<String>,
        transport: Arc<dyn HttpTransport + Send + Sync>,
    ) -> Self {
        Self {
            model: model.into(),
            api_base: api_base.into(),
            api_key: api_key.into(),
            transport,
        }
    }
}

impl LlmProvider for OpenAiCompatibleProvider {
    fn name(&self) -> &str {
        "openai-compatible"
    }

    fn generate(&self, request: &ProviderRequest) -> Result<ProviderResponse, ProviderError> {
        let url = format!("{}/chat/completions", self.api_base.trim_end_matches('/'));
        let body = format!(
            "{{\"model\":\"{}\",\"messages\":[{{\"role\":\"user\",\"content\":\"{}\"}}]}}",
            self.model,
            escape_json(&request.user_input)
        );
        let response = self
            .transport
            .send(&HttpRequest {
                method: "POST".to_string(),
                url,
                headers: vec![
                    ("Content-Type".to_string(), "application/json".to_string()),
                    (
                        "Authorization".to_string(),
                        format!("Bearer {}", self.api_key),
                    ),
                ],
                body,
            })
            .map_err(|e| ProviderError::Transport(format!("{e:?}")))?;

        if response.status_code >= 400 {
            return Err(ProviderError::Unavailable);
        }

        let content = extract_json_string(&response.body, "content").ok_or(ProviderError::Parse)?;
        Ok(ProviderResponse {
            output: content,
            model: self.model.clone(),
        })
    }
}

pub struct AnthropicProvider {
    model: String,
    api_base: String,
    api_key: String,
    transport: Arc<dyn HttpTransport + Send + Sync>,
}

impl AnthropicProvider {
    pub fn new(
        model: impl Into<String>,
        api_base: impl Into<String>,
        api_key: impl Into<String>,
        transport: Arc<dyn HttpTransport + Send + Sync>,
    ) -> Self {
        Self {
            model: model.into(),
            api_base: api_base.into(),
            api_key: api_key.into(),
            transport,
        }
    }
}

impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn generate(&self, request: &ProviderRequest) -> Result<ProviderResponse, ProviderError> {
        let url = format!("{}/messages", self.api_base.trim_end_matches('/'));
        let body = format!(
            "{{\"model\":\"{}\",\"max_tokens\":256,\"messages\":[{{\"role\":\"user\",\"content\":\"{}\"}}]}}",
            self.model,
            escape_json(&request.user_input)
        );
        let response = self
            .transport
            .send(&HttpRequest {
                method: "POST".to_string(),
                url,
                headers: vec![
                    ("Content-Type".to_string(), "application/json".to_string()),
                    ("x-api-key".to_string(), self.api_key.clone()),
                    ("anthropic-version".to_string(), "2023-06-01".to_string()),
                ],
                body,
            })
            .map_err(|e| ProviderError::Transport(format!("{e:?}")))?;

        if response.status_code >= 400 {
            return Err(ProviderError::Unavailable);
        }

        let text = extract_json_string(&response.body, "text").ok_or(ProviderError::Parse)?;
        Ok(ProviderResponse {
            output: text,
            model: self.model.clone(),
        })
    }
}

pub fn provider_from_config(config: &LlmConfig) -> Box<dyn LlmProvider> {
    let transport: Arc<dyn HttpTransport + Send + Sync> = Arc::new(StdHttpTransport);

    match config.provider.as_str() {
        "openai" | "openai-compatible" => {
            if let (Some(base), Some(key)) = (config.api_base.clone(), config.api_key.clone()) {
                Box::new(OpenAiCompatibleProvider::new(
                    config.model.clone(),
                    base,
                    key,
                    transport,
                ))
            } else {
                Box::new(ConfiguredEchoProvider::new(
                    "echo-local",
                    config.model.clone(),
                ))
            }
        }
        "anthropic" => {
            if let (Some(base), Some(key)) = (config.api_base.clone(), config.api_key.clone()) {
                Box::new(AnthropicProvider::new(
                    config.model.clone(),
                    base,
                    key,
                    transport,
                ))
            } else {
                Box::new(ConfiguredEchoProvider::new(
                    "echo-local",
                    config.model.clone(),
                ))
            }
        }
        _ => Box::new(ConfiguredEchoProvider::new(
            config.provider.clone(),
            config.model.clone(),
        )),
    }
}

pub struct ProviderRouter {
    providers: Vec<Box<dyn LlmProvider>>,
}

impl ProviderRouter {
    pub fn new(providers: Vec<Box<dyn LlmProvider>>) -> Self {
        Self { providers }
    }

    pub fn generate(&self, request: &ProviderRequest) -> Result<ProviderResponse, ProviderError> {
        for provider in &self.providers {
            if let Ok(response) = provider.generate(request) {
                return Ok(response);
            }
        }
        Err(ProviderError::Unavailable)
    }
}

fn escape_json(input: &str) -> String {
    input.replace('"', "\\\"")
}

fn extract_json_string(body: &str, field: &str) -> Option<String> {
    let token = format!("\"{}\":\"", field);
    let (_, tail) = body.split_once(&token)?;
    let value = tail.split('"').next()?.to_string();
    Some(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::{HttpError, HttpResponse};

    struct FailingProvider;

    impl LlmProvider for FailingProvider {
        fn name(&self) -> &str {
            "down"
        }

        fn generate(&self, _request: &ProviderRequest) -> Result<ProviderResponse, ProviderError> {
            Err(ProviderError::Unavailable)
        }
    }

    struct MockTransport {
        body: String,
    }

    impl HttpTransport for MockTransport {
        fn send(&self, _request: &HttpRequest) -> Result<HttpResponse, HttpError> {
            Ok(HttpResponse {
                status_code: 200,
                body: self.body.clone(),
            })
        }
    }

    #[test]
    fn router_falls_back_to_next_provider() {
        let router = ProviderRouter::new(vec![
            Box::new(FailingProvider),
            Box::new(ConfiguredEchoProvider::new("echo-local", "mock-1")),
        ]);
        let response = router
            .generate(&ProviderRequest {
                user_input: "hello".to_string(),
                context: vec![],
            })
            .expect("fallback provider should respond");

        assert_eq!(response.output, "hello");
        assert_eq!(response.model, "mock-1");
    }

    #[test]
    fn openai_provider_parses_response() {
        let p = OpenAiCompatibleProvider::new(
            "gpt-4o-mini",
            "http://localhost:1234/v1",
            "k",
            Arc::new(MockTransport {
                body: "{\"choices\":[{\"message\":{\"content\":\"hello from openai\"}}]}"
                    .to_string(),
            }),
        );
        let resp = p
            .generate(&ProviderRequest {
                user_input: "hi".to_string(),
                context: vec![],
            })
            .expect("should parse mocked response");
        assert_eq!(resp.output, "hello from openai");
    }

    #[test]
    fn anthropic_provider_parses_response() {
        let p = AnthropicProvider::new(
            "claude-3-5-sonnet",
            "http://localhost:1234/v1",
            "k",
            Arc::new(MockTransport {
                body: "{\"content\":[{\"type\":\"text\",\"text\":\"hello from anthropic\"}]}"
                    .to_string(),
            }),
        );
        let resp = p
            .generate(&ProviderRequest {
                user_input: "hi".to_string(),
                context: vec![],
            })
            .expect("should parse mocked response");
        assert_eq!(resp.output, "hello from anthropic");
    }
}
