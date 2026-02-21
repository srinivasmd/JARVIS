use std::io::{Read, Write};
use std::net::TcpStream;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status_code: u16,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpError {
    InvalidUrl,
    Transport(String),
    InvalidResponse,
}

pub trait HttpTransport {
    fn send(&self, request: &HttpRequest) -> Result<HttpResponse, HttpError>;
}

#[derive(Debug, Default)]
pub struct StdHttpTransport;

impl HttpTransport for StdHttpTransport {
    fn send(&self, request: &HttpRequest) -> Result<HttpResponse, HttpError> {
        let (host, port, path) = parse_http_url(&request.url)?;
        let mut stream = TcpStream::connect((host.as_str(), port))
            .map_err(|e| HttpError::Transport(e.to_string()))?;

        let mut headers = String::new();
        for (k, v) in &request.headers {
            headers.push_str(&format!("{k}: {v}\r\n"));
        }

        let req = format!(
            "{} {} HTTP/1.1\r\nHost: {}\r\nContent-Length: {}\r\nConnection: close\r\n{}\r\n{}",
            request.method,
            path,
            host,
            request.body.len(),
            headers,
            request.body
        );

        stream
            .write_all(req.as_bytes())
            .map_err(|e| HttpError::Transport(e.to_string()))?;

        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .map_err(|e| HttpError::Transport(e.to_string()))?;

        let mut lines = response.lines();
        let status_line = lines.next().ok_or(HttpError::InvalidResponse)?;
        let status_code = status_line
            .split_whitespace()
            .nth(1)
            .and_then(|v| v.parse::<u16>().ok())
            .ok_or(HttpError::InvalidResponse)?;

        let body = response
            .split("\r\n\r\n")
            .nth(1)
            .unwrap_or_default()
            .to_string();

        Ok(HttpResponse { status_code, body })
    }
}

pub fn parse_http_url(url: &str) -> Result<(String, u16, String), HttpError> {
    if !url.starts_with("http://") {
        return Err(HttpError::InvalidUrl);
    }

    let rest = &url[7..];
    let mut parts = rest.splitn(2, '/');
    let host_port = parts.next().ok_or(HttpError::InvalidUrl)?;
    let path = format!("/{}", parts.next().unwrap_or_default());

    let mut host_parts = host_port.splitn(2, ':');
    let host = host_parts.next().ok_or(HttpError::InvalidUrl)?.to_string();
    let port = host_parts
        .next()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(80);

    Ok((host, port, path))
}
