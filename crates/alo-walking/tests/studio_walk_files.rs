//! Only normal test packages go in; only generated recording evidence comes out.

#[path = "../src/studio_walk_files.rs"]
mod studio_walk_files;

use std::io::{Cursor, Read, Write};
use std::path::PathBuf;

use studio_walk_files::Files;

struct Connection {
    input: Cursor<Vec<u8>>,
    output: Vec<u8>,
}

impl Read for Connection {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        self.input.read(bytes)
    }
}

impl Write for Connection {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.output.write(bytes)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

struct Fixture {
    root: PathBuf,
    files: Files,
}

impl Fixture {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "studio-walk-files-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        ));
        std::fs::create_dir(&root)?;
        std::fs::create_dir(root.join("inputs"))?;
        let files = Files::new(root.join("inputs"), root.join("evidence"))?;
        Ok(Self { root, files })
    }

    fn request(&self, bytes: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut connection = Connection { input: Cursor::new(bytes.to_vec()), output: Vec::new() };
        self.files.answer(&mut connection)?;
        Ok(connection.output)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn a_normal_package_is_served_byte_for_byte() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    std::fs::write(fixture.root.join("inputs/studio.flatpak"), b"normal package\0\xff")?;
    let answer = fixture.request(b"GET /studio.flatpak HTTP/1.1\r\nHost: fixture\r\n\r\n")?;
    let answer = answer.splitn(2, |byte| *byte == 0).collect::<Vec<_>>();
    assert!(answer.first().is_some_and(|bytes| bytes.starts_with(b"HTTP/1.1 200")));
    assert!(answer.last().is_some_and(|bytes| *bytes == b"\xff"));
    Ok(())
}

#[test]
fn unknown_inputs_and_traversal_never_serve_a_host_file() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    std::fs::write(fixture.root.join("secret"), b"private host file")?;
    for name in ["../secret", "%2e%2e/secret", "source.txt?file=../secret", "secret", "", "screen.webm"] {
        let request = format!("GET /{name} HTTP/1.1\r\n\r\n");
        let answer = fixture.request(request.as_bytes())?;
        assert!(answer.starts_with(b"HTTP/1.1 404"), "{name}");
        assert!(!answer.windows(17).any(|bytes| bytes == b"private host file"));
    }
    Ok(())
}

#[test]
fn only_complete_generated_evidence_is_kept_without_overwriting() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let request = b"POST /screen.webm HTTP/1.1\r\nContent-Length: 5\r\n\r\nvideo";
    assert!(fixture.request(request)?.starts_with(b"HTTP/1.1 201"));
    assert_eq!(std::fs::read(fixture.root.join("evidence/screen.webm"))?, b"video");
    assert!(fixture.request(request)?.starts_with(b"HTTP/1.1 409"));
    assert!(fixture.request(b"POST /window.webm HTTP/1.1\r\nContent-Length: 9\r\n\r\nshort")?.starts_with(b"HTTP/1.1 400"));
    assert!(!fixture.root.join("evidence/window.webm").exists());
    assert_eq!(std::fs::read_dir(fixture.root.join("evidence"))?.count(), 1);
    Ok(())
}

#[test]
fn inputs_cannot_be_replaced_and_arbitrary_outputs_are_refused() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    for name in ["studio.flatpak", "source.txt", "../secret", "anything", "recording.key"] {
        let request = format!("POST /{name} HTTP/1.1\r\nContent-Length: 1\r\n\r\nx");
        assert!(fixture.request(request.as_bytes())?.starts_with(b"HTTP/1.1 404"), "{name}");
    }
    assert_eq!(std::fs::read_dir(fixture.root.join("evidence"))?.count(), 0);
    Ok(())
}

#[test]
fn ambiguous_or_unbounded_bodies_are_refused_before_creating_evidence() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    for headers in [
        "", "Content-Length: -1\r\n", "Content-Length: 67108865\r\n",
        "Content-Length: 1\r\nContent-Length: 1\r\n",
        "Transfer-Encoding: chunked\r\nContent-Length: 1\r\n",
    ] {
        let request = format!("POST /window.webm HTTP/1.1\r\n{headers}\r\nx");
        assert!(fixture.request(request.as_bytes())?.starts_with(b"HTTP/1.1 400"), "{headers}");
    }
    let long = format!("POST /window.webm HTTP/1.1\r\nX-Large: {}", "x".repeat(16_384));
    assert!(fixture.request(long.as_bytes())?.starts_with(b"HTTP/1.1 431"));
    assert_eq!(std::fs::read_dir(fixture.root.join("evidence"))?.count(), 0);
    Ok(())
}

#[cfg(unix)]
#[test]
fn a_package_symlink_cannot_expose_another_host_file() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    std::fs::write(fixture.root.join("secret"), b"private host file")?;
    std::os::unix::fs::symlink(fixture.root.join("secret"), fixture.root.join("inputs/studio.flatpak"))?;
    assert!(fixture.request(b"GET /studio.flatpak HTTP/1.1\r\n\r\n")?.starts_with(b"HTTP/1.1 404"));
    Ok(())
}
