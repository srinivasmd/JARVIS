use std::io::{Read, Write};
use std::net::TcpListener;

pub fn run_http_once(address: &str, body: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(address)?;
    if let Ok((mut stream, _)) = listener.accept() {
        let mut buffer = [0; 1024];
        let _ = stream.read(&mut buffer)?;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes())?;
    }
    Ok(())
}
