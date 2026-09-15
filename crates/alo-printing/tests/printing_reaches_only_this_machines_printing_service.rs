//! Nothing in this crate starts a program, writes a driver or opens a
//! connection anywhere but this machine's own printing service.
//!
//! The plan's constraint: the printing service is rented, configured and never
//! patched, no driver is written here and no maker's installer is run. The
//! strongest form of that is structural, so this reads what the crate is built
//! from — its manifest and its shipped source — and fails the day either gains
//! a road somewhere else. And it reaches the service the way a machine built
//! from the image does: over the service's own socket.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod serving;

use std::fs;
use std::path::{Path, PathBuf};

/// This crate, on the disk it is checked out on.
fn this_crate() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every `.rs` file under a folder.
fn sources_under(folder: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(folder).expect("the source folder to be listed") {
        let path = entry.expect("an entry of the source folder").path();
        if path.is_dir() {
            sources_under(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            found.push(path);
        }
    }
}

/// The names of the dependencies a section of the manifest lists.
fn dependencies_in(manifest: &str, section: &str) -> Vec<String> {
    let mut inside = false;
    let mut named = Vec::new();
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            inside = line == section;
            continue;
        }
        if inside
            && !line.starts_with('#')
            && let Some((name, _)) = line.split_once('=')
        {
            named.push(name.trim().to_owned());
        }
    }
    named
}

/// **Nothing here starts a program, and one file opens a connection** — to an
/// address that has already been checked to be this machine's, or to a socket.
/// No HTTP client, no protocol library and no process runner is depended on, so
/// every byte sent to the printing service is written in this crate.
#[test]
fn nothing_here_starts_a_program_or_reaches_past_this_machine() {
    let manifest =
        fs::read_to_string(this_crate().join("Cargo.toml")).expect("this crate's manifest");
    assert_eq!(
        dependencies_in(&manifest, "[dependencies]"),
        [
            "alo-capability",
            "alo-egress",
            "alo-opening",
            "alo-strings",
            "thiserror"
        ],
        "alo-printing gained a dependency: whatever it can reach, printing now can"
    );

    let mut sources = Vec::new();
    sources_under(&this_crate().join("src"), &mut sources);
    assert!(sources.len() > 10, "the shipped source was not found");
    for source in sources {
        let written = fs::read_to_string(&source).expect("a source file");
        for starting in [
            "std::process",
            "Command::new",
            "UdpSocket",
            "lpadmin",
            "unsafe",
        ] {
            assert!(
                !written.contains(starting),
                "{} names {starting}",
                source.display()
            );
        }
        let connects = written.contains("TcpStream") || written.contains("UnixStream");
        let is_the_door = source.file_name().is_some_and(|name| name == "service.rs");
        assert!(
            !connects || is_the_door,
            "{} opens a connection, and only service.rs may — after it has checked the \
             address is this machine's",
            source.display()
        );
    }
}

/// **The service is reached over its own socket** — the way a machine built
/// from the image reaches it, where who may add a printer is decided from the
/// credentials at the other end.
#[cfg(unix)]
#[test]
fn the_printing_service_is_reached_over_its_own_socket() {
    use std::io::{Read, Write};
    use std::os::unix::net::UnixListener;

    use alo_printing::ipp::Message;
    use alo_printing::{PrintingService, find};

    let socket = std::env::temp_dir().join(format!("alo-printing-{}.sock", std::process::id()));
    drop(fs::remove_file(&socket));
    let listener = UnixListener::bind(&socket).expect("a socket of our own");
    let answering = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("the one connection");
        let mut bytes = Vec::new();
        let mut chunk = [0_u8; 4096];
        let request = loop {
            let read = stream.read(&mut chunk).expect("the request");
            bytes.extend(chunk.get(..read).unwrap_or_default());
            let Some(head) = bytes.windows(4).position(|w| w == b"\r\n\r\n") else {
                continue;
            };
            if let Ok(request) = Message::read(bytes.get(head + 4..).unwrap_or_default()) {
                break request;
            }
        };
        let body = serving::with_device(
            Message::answer(0, request.request_id()).beginning(alo_printing::ipp::Group::Operation),
            "ippusb://Canon%20PIXMA/?serial=1",
            "Canon PIXMA G3560",
        )
        .written()
        .expect("an answer");
        let mut answer =
            format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n", body.len()).into_bytes();
        answer.extend(body);
        stream.write_all(&answer).expect("the answer");
        request.code()
    });

    let found = find(&PrintingService::at_its_socket(&socket)).expect("the service answered");
    assert_eq!(answering.join().expect("the service"), serving::GET_DEVICES);
    assert_eq!(found.len(), 1);
    assert!(
        found
            .first()
            .is_some_and(|printer| !printer.reached().crosses_the_network())
    );
    drop(fs::remove_file(&socket));
}
