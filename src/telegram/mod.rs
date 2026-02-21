use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;

use crate::{
    channels::{ChannelAdapter, IncomingMessage, WebhookAdapter},
    config::TelegramConfig,
    core::Agent,
    net::{HttpRequest, HttpTransport, StdHttpTransport},
};

#[derive(Debug, Clone)]
pub struct TelegramUpdate {
    pub update_id: i64,
    pub chat_id: String,
    pub text: String,
    pub user: String,
}

#[derive(Clone)]
pub struct TelegramAdapter {
    bot_token: String,
    api_base: String,
    default_chat_id: String,
    transport: Arc<dyn HttpTransport + Send + Sync>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelegramError {
    Config(String),
    Transport(String),
    Parse,
}

impl TelegramAdapter {
    pub fn from_config(config: &TelegramConfig) -> Result<Self, TelegramError> {
        let bot_token = config
            .bot_token
            .clone()
            .ok_or_else(|| TelegramError::Config("missing bot token".to_string()))?;
        let chat_id = config
            .chat_id
            .clone()
            .ok_or_else(|| TelegramError::Config("missing chat id".to_string()))?;
        let api_base = config
            .api_base
            .clone()
            .unwrap_or_else(|| "http://127.0.0.1:8081".to_string());

        Ok(Self {
            bot_token,
            api_base,
            default_chat_id: chat_id,
            transport: Arc::new(StdHttpTransport),
        })
    }

    pub fn with_transport(
        config: &TelegramConfig,
        transport: Arc<dyn HttpTransport + Send + Sync>,
    ) -> Result<Self, TelegramError> {
        let mut base = Self::from_config(config)?;
        base.transport = transport;
        Ok(base)
    }

    pub fn poll_updates_once(
        &self,
        offset: Option<i64>,
    ) -> Result<Vec<TelegramUpdate>, TelegramError> {
        let mut url = format!(
            "{}/bot{}/getUpdates",
            self.api_base.trim_end_matches('/'),
            self.bot_token
        );
        if let Some(ofs) = offset {
            url = format!("{url}?offset={ofs}");
        }

        let response = self
            .transport
            .send(&HttpRequest {
                method: "GET".to_string(),
                url,
                headers: vec![],
                body: String::new(),
            })
            .map_err(|e| TelegramError::Transport(format!("{e:?}")))?;

        parse_updates(&response.body)
    }

    pub fn send_message(&self, chat_id: Option<&str>, text: &str) -> Result<(), TelegramError> {
        let target_chat = chat_id.unwrap_or(&self.default_chat_id);
        let url = format!(
            "{}/bot{}/sendMessage",
            self.api_base.trim_end_matches('/'),
            self.bot_token
        );
        let body = format!(
            "{{\"chat_id\":\"{}\",\"text\":\"{}\"}}",
            target_chat,
            text.replace('"', "\\\"")
        );

        self.transport
            .send(&HttpRequest {
                method: "POST".to_string(),
                url,
                headers: vec![("Content-Type".to_string(), "application/json".to_string())],
                body,
            })
            .map_err(|e| TelegramError::Transport(format!("{e:?}")))?;

        Ok(())
    }

    pub fn handle_polling_once(&self, agent: &mut Agent) -> Result<usize, TelegramError> {
        let updates = self.poll_updates_once(None)?;
        for update in &updates {
            let adapter = WebhookAdapter;
            let incoming = adapter.normalize(&update.user, &update.text);
            if let Ok(outgoing) = agent.handle_message(incoming) {
                self.send_message(Some(&update.chat_id), &outgoing.text)?;
            }
        }
        Ok(updates.len())
    }
}

pub fn run_telegram_webhook_once(bind: &str, agent: &mut Agent) -> std::io::Result<()> {
    let listener = TcpListener::bind(bind)?;
    if let Ok((mut stream, _)) = listener.accept() {
        let mut buffer = [0; 4096];
        let size = stream.read(&mut buffer)?;
        let request = String::from_utf8_lossy(&buffer[..size]);
        let body = request.split("\r\n\r\n").nth(1).unwrap_or_default();

        let incoming = parse_webhook_message(body).unwrap_or(IncomingMessage {
            channel: "telegram-webhook".to_string(),
            user: "unknown".to_string(),
            text: "".to_string(),
        });

        let response_text = match agent.handle_message(incoming) {
            Ok(response) => response.text,
            Err(err) => format!("error: {err}"),
        };

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            response_text.len(),
            response_text
        );
        stream.write_all(response.as_bytes())?;
    }
    Ok(())
}

fn parse_updates(body: &str) -> Result<Vec<TelegramUpdate>, TelegramError> {
    if !body.contains("\"result\"") {
        return Err(TelegramError::Parse);
    }

    let mut updates = Vec::new();
    let chunks: Vec<&str> = body.split("\"update_id\":").collect();
    for chunk in chunks.iter().skip(1) {
        let update_id = chunk
            .split(|c: char| !c.is_ascii_digit())
            .next()
            .and_then(|v| v.parse::<i64>().ok())
            .unwrap_or_default();
        let chat_id = extract_json_number(chunk, "id").unwrap_or_else(|| "0".to_string());
        let text = extract_json_string(chunk, "text").unwrap_or_default();
        let user =
            extract_json_string(chunk, "username").unwrap_or_else(|| "telegram-user".to_string());

        if !text.is_empty() {
            updates.push(TelegramUpdate {
                update_id,
                chat_id,
                text,
                user,
            });
        }
    }

    Ok(updates)
}

fn parse_webhook_message(body: &str) -> Option<IncomingMessage> {
    let text = extract_json_string(body, "text")?;
    let user = extract_json_string(body, "username").unwrap_or_else(|| "telegram-user".to_string());
    Some(IncomingMessage {
        channel: "telegram-webhook".to_string(),
        user,
        text,
    })
}

fn extract_json_string(source: &str, key: &str) -> Option<String> {
    let token = format!("\"{}\":\"", key);
    let (_, tail) = source.split_once(&token)?;
    Some(tail.split('"').next()?.to_string())
}

fn extract_json_number(source: &str, key: &str) -> Option<String> {
    let token = format!("\"{}\":", key);
    let (_, tail) = source.split_once(&token)?;
    Some(
        tail.chars()
            .skip_while(|c| c.is_whitespace())
            .take_while(|c| c.is_ascii_digit() || *c == '-')
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::{HttpError, HttpResponse};

    struct MockTelegramTransport {
        body: String,
    }

    impl HttpTransport for MockTelegramTransport {
        fn send(&self, _request: &HttpRequest) -> Result<HttpResponse, HttpError> {
            Ok(HttpResponse {
                status_code: 200,
                body: self.body.clone(),
            })
        }
    }

    #[test]
    fn parse_polling_updates() {
        let config = TelegramConfig {
            enabled: true,
            bot_token: Some("token".to_string()),
            chat_id: Some("1".to_string()),
            webhook_url: None,
            api_base: Some("http://localhost:8081".to_string()),
            polling_interval_ms: 500,
        };

        let adapter = TelegramAdapter::with_transport(
            &config,
            Arc::new(MockTelegramTransport {
                body: "{\"ok\":true,\"result\":[{\"update_id\":1,\"message\":{\"chat\":{\"id\":42},\"from\":{\"username\":\"alice\"},\"text\":\"hello\"}}]}"
                    .to_string(),
            }),
        )
        .expect("adapter should build");

        let updates = adapter.poll_updates_once(None).expect("poll should parse");
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].text, "hello");
    }
}
