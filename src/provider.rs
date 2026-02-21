use std::io::{Read, Write};
use std::net::TcpStream;
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
        // Design choice: keep transport dependency-free using raw HTTP over TLS-disabled dev endpoint.
        // For production, wire a hardened HTTP client.
        let endpoint = OpenAiEndpoint::parse(&self.api_base)?;
        let mut stream = TcpStream::connect(format!("{}:{}", endpoint.host, endpoint.port))
            .map_err(|e| e.to_string())?;
        let body = format!(
            "{{\"model\":\"{}\",\"messages\":[{{\"role\":\"user\",\"content\":\"{}\"}}]}}",
            self.model,
            req.prompt.replace('"', "'")
        );
        let path = format!("{}/chat/completions", endpoint.base_path);
        let req = format!(
            "POST {} HTTP/1.1\r\nHost: {}\r\nAuthorization: Bearer {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            path,
            endpoint.host,
            self.api_key,
            body.len(),
            body
        );
        stream
            .write_all(req.as_bytes())
            .map_err(|e| e.to_string())?;
        let mut out = String::new();
        stream.read_to_string(&mut out).map_err(|e| e.to_string())?;
        Ok(LlmResponse {
            text: format!("openai_raw_response:{}", out.lines().next().unwrap_or("")),
            model: self.model.clone(),
        })
    }
}

#[derive(Debug)]
struct OpenAiEndpoint {
    host: String,
    port: u16,
    base_path: String,
}

impl OpenAiEndpoint {
    fn parse(input: &str) -> Result<Self, String> {
        let raw = input
            .trim()
            .trim_start_matches("http://")
            .trim_start_matches("https://");
        let (host_and_port, path) = raw
            .split_once('/')
            .map(|(host, p)| (host, format!("/{}", p.trim_matches('/'))))
            .unwrap_or((raw, "/v1".to_string()));
        let (host, port) = if let Some((h, p)) = host_and_port.split_once(':') {
            let parsed = p
                .parse::<u16>()
                .map_err(|_| format!("invalid port in api_base: {input}"))?;
            (h.to_string(), parsed)
        } else {
            (host_and_port.to_string(), 80)
        };
        if host.is_empty() {
            return Err("api_base host is empty".to_string());
        }
        Ok(Self {
            host,
            port,
            base_path: path,
        })
    }
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
        let endpoint = OpenAiEndpoint::parse("http://localhost:1234/custom/v1").expect("parse");
        assert_eq!(endpoint.host, "localhost");
        assert_eq!(endpoint.port, 1234);
        assert_eq!(endpoint.base_path, "/custom/v1");
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
}
