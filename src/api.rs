use crate::lc_core::Core;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;

pub fn run_api(bind: &str, core: Arc<Core>) -> std::io::Result<()> {
    let listener = TcpListener::bind(bind)?;
    for stream in listener.incoming().take(1) {
        let mut stream = stream?;
        let mut buf = [0u8; 4096];
        let read = stream.read(&mut buf)?;
        let req = String::from_utf8_lossy(&buf[..read]);
        let body = if let Some((_, payload)) = req.split_once("\r\n\r\n") {
            payload.to_string()
        } else {
            String::new()
        };
        let reply = core
            .handle_message(body.trim())
            .unwrap_or_else(|e| format!("error:{e}"));
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            reply.len(),
            reply
        );
        stream.write_all(response.as_bytes())?;
    }
    Ok(())
}
