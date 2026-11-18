//! Job-local transfer of normal packages and generated recording evidence.
//!
//! The walk VM reaches its host over QEMU user networking. This instrument
//! serves a closed list of build inputs and accepts a closed list of outputs;
//! it has no command endpoint and never accepts a filesystem path from a VM.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

/// Maximum request-header size, independent of any supplied length.
const HEADER_LIMIT: usize = 16_384;
/// A generated clip must fit this bound before its body is read.
const MEDIA_LIMIT: u64 = 64 * 1024 * 1024;
/// Small provenance and report files have a separate bound.
const REPORT_LIMIT: u64 = 1024 * 1024;
/// The build products made available to this particular VM.
const INPUTS: &[&str] = &[
    "studio.flatpak",
    "fixture.flatpak",
    "source.txt",
    "features.txt",
    "checksums.txt",
    "studio-session",
    "studio-agent-probe",
    "sdk-versions.txt",
];
/// Only generated evidence is accepted, and it is never served back as input.
const OUTPUTS: &[&str] = &[
    "screen.webm",
    "window.webm",
    "reference.webm",
    "screen.json",
    "window.json",
    "reference.json",
    "session-report.json",
];

/// One private walk's immutable inputs and generated evidence directory.
pub struct Files {
    /// Canonical input directory owned by the job.
    inputs: PathBuf,
    /// Canonical output directory owned by the job.
    evidence: PathBuf,
}

impl Files {
    /// Open the inputs and create the separate evidence directory.
    ///
    /// # Errors
    /// When either location is unavailable or is not a directory.
    pub fn new(inputs: PathBuf, evidence: PathBuf) -> io::Result<Self> {
        let inputs = inputs.canonicalize()?;
        fs::create_dir_all(&evidence)?;
        let evidence = evidence.canonicalize()?;
        if !inputs.is_dir() || !evidence.is_dir() || inputs == evidence {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "separate input and evidence directories are required",
            ));
        }
        Ok(Self { inputs, evidence })
    }

    /// Answer one bounded HTTP request, then close the connection.
    ///
    /// # Errors
    /// When the connection fails or a host file cannot be read or written.
    pub fn answer(&self, connection: &mut (impl Read + Write)) -> io::Result<()> {
        let request = match request(connection) {
            Ok(request) => request,
            Err(status) => return response(connection, status, 0),
        };
        match request.method.as_str() {
            "GET" if INPUTS.contains(&request.name.as_str()) => {
                if request.length.is_some_and(|length| length != 0) {
                    return response(connection, 400, 0);
                }
                let path = self.inputs.join(&request.name);
                let metadata = match fs::symlink_metadata(&path) {
                    Ok(metadata) if metadata.is_file() => metadata,
                    _ => return response(connection, 404, 0),
                };
                let mut file = File::open(path)?;
                response(connection, 200, metadata.len())?;
                let copied = io::copy(&mut file, connection)?;
                if copied != metadata.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::UnexpectedEof,
                        "a build input changed during transfer",
                    ));
                }
                Ok(())
            }
            "POST" if OUTPUTS.contains(&request.name.as_str()) => {
                let limit = if request.name.ends_with(".webm") {
                    MEDIA_LIMIT
                } else {
                    REPORT_LIMIT
                };
                let Some(length) = request
                    .length
                    .filter(|length| *length > 0 && *length <= limit)
                else {
                    return response(connection, 400, 0);
                };
                self.receive(connection, &request.name, length)
            }
            "GET" | "POST" => response(connection, 404, 0),
            _ => response(connection, 405, 0),
        }
    }

    /// Publish a complete output atomically without ever replacing evidence.
    fn receive(
        &self,
        connection: &mut (impl Read + Write),
        name: &str,
        length: u64,
    ) -> io::Result<()> {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let destination = self.evidence.join(name);
        if destination.symlink_metadata().is_ok() {
            return response(connection, 409, 0);
        }
        let temporary = self.evidence.join(format!(
            ".{name}-{}-{}.partial",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
        ));
        let pending = Pending(temporary);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&pending.0)?;
        let copied = io::copy(&mut connection.take(length), &mut file)?;
        if copied != length {
            return response(connection, 400, 0);
        }
        file.sync_all()?;
        drop(file);
        // A hard link, unlike rename, cannot overwrite a destination another
        // upload created in the meantime. Both paths are on this same volume.
        match fs::hard_link(&pending.0, destination) {
            Ok(()) => response(connection, 201, 0),
            Err(why) if why.kind() == io::ErrorKind::AlreadyExists => response(connection, 409, 0),
            Err(why) => Err(why),
        }
    }
}

/// An unpublished partial body is removed on every exit path.
struct Pending(PathBuf);

impl Drop for Pending {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

/// The only fields this nonpersistent transfer protocol accepts.
struct Request {
    /// GET or POST; other methods receive 405.
    method: String,
    /// Exactly one filename, never an encoded or decoded path.
    name: String,
    /// A unique checked content length, if supplied.
    length: Option<u64>,
}

/// Read the header byte by byte so a missing newline cannot allocate forever.
fn request(input: &mut impl Read) -> Result<Request, u16> {
    let mut header = Vec::new();
    while !header.ends_with(b"\r\n\r\n") {
        if header.len() >= HEADER_LIMIT {
            return Err(431);
        }
        let mut byte = [0_u8];
        if input.read_exact(&mut byte).is_err() {
            return Err(400);
        }
        header.extend_from_slice(&byte);
    }
    let text = std::str::from_utf8(&header).map_err(|_| 400_u16)?;
    let mut lines = text.split("\r\n");
    let mut line = lines.next().ok_or(400_u16)?.split_whitespace();
    let method = line.next().ok_or(400_u16)?.to_owned();
    let name = line
        .next()
        .and_then(|path| path.strip_prefix('/'))
        .ok_or(400_u16)?
        .to_owned();
    if !matches!(line.next(), Some("HTTP/1.1" | "HTTP/1.0")) || line.next().is_some() {
        return Err(400);
    }
    let mut length = None;
    for line in lines.take_while(|line| !line.is_empty()) {
        let (name, value) = line.split_once(':').ok_or(400_u16)?;
        if name.eq_ignore_ascii_case("transfer-encoding") {
            return Err(400);
        }
        if name.eq_ignore_ascii_case("content-length") {
            if length.is_some() {
                return Err(400);
            }
            length = Some(value.trim().parse().map_err(|_| 400_u16)?);
        }
    }
    Ok(Request {
        method,
        name,
        length,
    })
}

/// A fixed response header contains no input path or request text.
fn response(output: &mut impl Write, status: u16, length: u64) -> io::Result<()> {
    let words = match status {
        200 => "OK",
        201 => "Created",
        400 => "Bad Request",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        431 => "Request Header Fields Too Large",
        _ => "Internal Server Error",
    };
    write!(
        output,
        "HTTP/1.1 {status} {words}\r\nContent-Length: {length}\r\nConnection: close\r\nContent-Type: application/octet-stream\r\n\r\n"
    )
}
