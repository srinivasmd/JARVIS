use std::io::{Read, Write};
use std::net::TcpListener;

use crate::{
    channels::{ChannelAdapter, WebhookAdapter},
    core::Agent,
};

pub fn web_ui_html() -> &'static str {
    r#"<!doctype html>
<html><head><meta charset='utf-8'><title>Secure LightClaw v2</title></head>
<body>
<h1>Secure LightClaw v2</h1>
<form method='POST' action='/chat'>
<input name='user' value='web-user'/>
<input name='message' placeholder='Say something'/>
<button type='submit'>Send</button>
</form>
</body></html>"#
}

pub fn run_web_ui_once(address: &str, agent: &mut Agent) -> std::io::Result<()> {
    let listener = TcpListener::bind(address)?;
    if let Ok((mut stream, _)) = listener.accept() {
        let mut buffer = [0; 2048];
        let size = stream.read(&mut buffer)?;
        let request = String::from_utf8_lossy(&buffer[..size]);

        let response_body = if request.starts_with("GET / ") {
            web_ui_html().to_string()
        } else if request.starts_with("POST /chat ") {
            let body = request.split("\r\n\r\n").nth(1).unwrap_or_default();
            let message = parse_form_value(body, "message").unwrap_or_else(|| "hello".to_string());
            let user = parse_form_value(body, "user").unwrap_or_else(|| "web-user".to_string());

            let adapter = WebhookAdapter;
            let incoming = adapter.normalize(&user, &message);
            match agent.handle_message(incoming) {
                Ok(out) => out.text,
                Err(err) => format!("error: {err}"),
            }
        } else {
            "not found".to_string()
        };

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
            response_body.len(),
            response_body
        );
        stream.write_all(response.as_bytes())?;
    }
    Ok(())
}

fn parse_form_value(body: &str, key: &str) -> Option<String> {
    for pair in body.split('&') {
        if let Some((k, v)) = pair.split_once('=') {
            if k == key {
                return Some(v.replace('+', " "));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serves_html_shell() {
        let html = web_ui_html();
        assert!(html.contains("Secure LightClaw v2"));
        assert!(html.contains("/chat"));
    }
}
