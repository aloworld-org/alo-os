//! A stand-in service for this crate's tests, on a real socket.
//!
//! A real server rather than a mocked HTTP client, because the thing worth
//! testing is what goes out on the wire and what we make of what comes back —
//! which a mock at the client boundary would assume rather than check. The
//! Ollama adapter and the provider test both need one, and two copies of a
//! fixture drift into two fixtures.

#![expect(
    clippy::unwrap_used,
    reason = "in a test fixture, a panic on a None or an Err is the failure being reported"
)]

use std::io::{BufRead as _, Read as _, Write as _};
use std::net::TcpListener;
use std::thread;

/// One request, one canned reply.
///
/// Answers with the address to point an adapter at, and a handle that yields
/// everything the client sent — head and body — so a test can assert on what
/// really went out rather than on what was intended.
pub(crate) fn serving(
    response_body: &'static str,
    status: u16,
) -> (String, thread::JoinHandle<String>) {
    serving_with(response_body, status, "")
}

/// The same, with extra header lines in the reply.
///
/// `extra_headers` is raw, each line ending `\r\n`. It exists for `Location:`,
/// which is the one reply where the head matters as much as the body: a
/// redirect is a provider telling us to go somewhere the policy never answered
/// about.
pub(crate) fn serving_with(
    response_body: &'static str,
    status: u16,
    extra_headers: &'static str,
) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        // Read the request head, and the body if one was announced.
        let mut reader = std::io::BufReader::new(stream.try_clone().unwrap());
        let mut head = String::new();
        let mut length = 0usize;
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap() == 0 {
                break;
            }
            if let Some(v) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                length = v.trim().parse().unwrap_or(0);
            }
            let done = line == "\r\n" || line == "\n";
            head.push_str(&line);
            if done {
                break;
            }
        }
        let mut body = vec![0u8; length];
        if length > 0 {
            reader.read_exact(&mut body).unwrap();
        }
        let reply = format!(
            "HTTP/1.1 {status} OK\r\nContent-Length: {}\r\nContent-Type: application/json\r\n{extra_headers}Connection: close\r\n\r\n{response_body}",
            response_body.len()
        );
        stream.write_all(reply.as_bytes()).unwrap();
        stream.flush().unwrap();
        head + &String::from_utf8_lossy(&body)
    });
    (format!("http://127.0.0.1:{port}"), handle)
}
