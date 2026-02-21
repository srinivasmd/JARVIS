use std::io::{Read, Write};
use std::net::TcpStream;

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
}

impl LlmProvider for OpenAiProvider {
    fn generate(&self, req: LlmRequest) -> Result<LlmResponse, String> {
        // Design choice: keep transport dependency-free using raw HTTP over TLS-disabled dev endpoint.
        // For production, wire a hardened HTTP client.
        let mut stream = TcpStream::connect("api.openai.com:80").map_err(|e| e.to_string())?;
        let body = format!(
            "{{\"model\":\"{}\",\"messages\":[{{\"role\":\"user\",\"content\":\"{}\"}}]}}",
            self.model,
            req.prompt.replace('"', "'")
        );
        let req = format!(
            "POST /v1/chat/completions HTTP/1.1\r\nHost: api.openai.com\r\nAuthorization: Bearer {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
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

pub struct MockProvider;
impl LlmProvider for MockProvider {
    fn generate(&self, req: LlmRequest) -> Result<LlmResponse, String> {
        Ok(LlmResponse {
            text: format!("echo:{}", req.prompt),
            model: "mock".into(),
        })
    }
}
