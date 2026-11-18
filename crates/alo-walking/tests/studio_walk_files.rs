//! Only normal test packages go in; only generated recording evidence comes out.

use std::io::{Cursor, Read, Write};
use std::path::PathBuf;

use alo_walking::studio_walk_files::Files;

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
        let mut connection = Connection {
            input: Cursor::new(bytes.to_vec()),
            output: Vec::new(),
        };
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
    std::fs::write(
        fixture.root.join("inputs/studio.flatpak"),
        b"normal package\0\xff",
    )?;
    let answer = fixture.request(b"GET /studio.flatpak HTTP/1.1\r\nHost: fixture\r\n\r\n")?;
    let body_at = answer
        .windows(4)
        .position(|bytes| bytes == b"\r\n\r\n")
        .ok_or("the response has no header separator")?;
    assert!(answer.starts_with(b"HTTP/1.1 200"));
    assert_eq!(
        answer.get(body_at + 4..),
        Some(b"normal package\0\xff".as_slice())
    );
    Ok(())
}

#[test]
fn unknown_inputs_and_traversal_never_serve_a_host_file() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = Fixture::new()?;
    std::fs::write(fixture.root.join("secret"), b"private host file")?;
    for name in [
        "../secret",
        "%2e%2e/secret",
        "source.txt?file=../secret",
        "secret",
        "",
        "screen.webm",
    ] {
        let request = format!("GET /{name} HTTP/1.1\r\n\r\n");
        let answer = fixture.request(request.as_bytes())?;
        assert!(answer.starts_with(b"HTTP/1.1 404"), "{name}");
        assert!(
            !answer
                .windows(17)
                .any(|bytes| bytes == b"private host file")
        );
    }
    Ok(())
}

#[test]
fn only_complete_generated_evidence_is_kept_without_overwriting()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    let request = b"POST /screen.webm HTTP/1.1\r\nContent-Length: 5\r\n\r\nvideo";
    assert!(fixture.request(request)?.starts_with(b"HTTP/1.1 201"));
    assert_eq!(
        std::fs::read(fixture.root.join("evidence/screen.webm"))?,
        b"video"
    );
    assert!(fixture.request(request)?.starts_with(b"HTTP/1.1 409"));
    assert!(
        fixture
            .request(b"POST /window.webm HTTP/1.1\r\nContent-Length: 9\r\n\r\nshort")?
            .starts_with(b"HTTP/1.1 400")
    );
    assert!(!fixture.root.join("evidence/window.webm").exists());
    assert_eq!(std::fs::read_dir(fixture.root.join("evidence"))?.count(), 1);
    Ok(())
}

#[test]
fn inputs_cannot_be_replaced_and_arbitrary_outputs_are_refused()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    for name in [
        "studio.flatpak",
        "source.txt",
        "../secret",
        "anything",
        "recording.key",
    ] {
        let request = format!("POST /{name} HTTP/1.1\r\nContent-Length: 1\r\n\r\nx");
        assert!(
            fixture
                .request(request.as_bytes())?
                .starts_with(b"HTTP/1.1 404"),
            "{name}"
        );
    }
    assert_eq!(std::fs::read_dir(fixture.root.join("evidence"))?.count(), 0);
    Ok(())
}

#[test]
fn disks_keys_packages_and_executable_fixtures_are_never_kept_as_evidence()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    for name in [
        "alo.raw",
        "alo.qcow2",
        "vars.fd",
        "disk.img",
        "private.key",
        "signing.pem",
        "test-account.json",
        "studio.flatpak",
        "fixture.flatpak",
        "studio-reference.flatpak",
        "studio-session",
        "checksums.txt",
        "features.txt",
        "sdk-versions.txt",
        "source.txt",
    ] {
        let request = format!("POST /{name} HTTP/1.1\r\nContent-Length: 4\r\n\r\ntest");
        assert!(
            fixture
                .request(request.as_bytes())?
                .starts_with(b"HTTP/1.1 404"),
            "{name}"
        );
    }
    assert_eq!(std::fs::read_dir(fixture.root.join("evidence"))?.count(), 0);
    Ok(())
}

#[test]
fn forbidden_input_names_are_refused_even_when_real_files_exist_in_the_input_directory()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    for name in [
        "alo.raw",
        "alo.qcow2",
        "private.key",
        "signing.pem",
        "test-account.json",
        "studio-reference.flatpak",
        "screen.webm",
        "session-report.json",
    ] {
        std::fs::write(
            fixture.root.join("inputs").join(name),
            b"test-only forbidden payload",
        )?;
        let request = format!("GET /{name} HTTP/1.1\r\n\r\n");
        let answer = fixture.request(request.as_bytes())?;
        assert!(answer.starts_with(b"HTTP/1.1 404"), "{name}");
        assert!(
            !answer
                .windows(27)
                .any(|part| part == b"test-only forbidden payload")
        );
    }
    Ok(())
}

#[test]
fn all_declared_recording_evidence_names_are_received_and_never_served_back()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    for name in [
        "screen.webm",
        "window.webm",
        "reference.webm",
        "screen.json",
        "window.json",
        "reference.json",
        "session-report.json",
    ] {
        let request = format!("POST /{name} HTTP/1.1\r\nContent-Length: 4\r\n\r\ntest");
        assert!(
            fixture
                .request(request.as_bytes())?
                .starts_with(b"HTTP/1.1 201"),
            "{name}"
        );
        assert_eq!(
            std::fs::read(fixture.root.join("evidence").join(name))?,
            b"test"
        );
        let get = format!("GET /{name} HTTP/1.1\r\n\r\n");
        assert!(
            fixture
                .request(get.as_bytes())?
                .starts_with(b"HTTP/1.1 404"),
            "{name}"
        );
    }
    Ok(())
}

#[test]
fn ambiguous_or_unbounded_bodies_are_refused_before_creating_evidence()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    for headers in [
        "",
        "Content-Length: -1\r\n",
        "Content-Length: 67108865\r\n",
        "Content-Length: 1\r\nContent-Length: 1\r\n",
        "Transfer-Encoding: chunked\r\nContent-Length: 1\r\n",
    ] {
        let request = format!("POST /window.webm HTTP/1.1\r\n{headers}\r\nx");
        assert!(
            fixture
                .request(request.as_bytes())?
                .starts_with(b"HTTP/1.1 400"),
            "{headers}"
        );
    }
    let long = format!(
        "POST /window.webm HTTP/1.1\r\nX-Large: {}",
        "x".repeat(16_384)
    );
    assert!(
        fixture
            .request(long.as_bytes())?
            .starts_with(b"HTTP/1.1 431")
    );
    assert_eq!(std::fs::read_dir(fixture.root.join("evidence"))?.count(), 0);
    Ok(())
}

#[cfg(unix)]
#[test]
fn a_package_symlink_cannot_expose_another_host_file() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::new()?;
    std::fs::write(fixture.root.join("secret"), b"private host file")?;
    std::os::unix::fs::symlink(
        fixture.root.join("secret"),
        fixture.root.join("inputs/studio.flatpak"),
    )?;
    assert!(
        fixture
            .request(b"GET /studio.flatpak HTTP/1.1\r\n\r\n")?
            .starts_with(b"HTTP/1.1 404")
    );
    Ok(())
}
