//! Converting can reach nothing off this machine, and names its engine once.
//!
//! Two promises ADR 0039 makes about this crate that live in its source rather
//! than in anything it does, so they are read off the source:
//!
//! - **Nothing is uploaded.** No dependency here can speak to the network, and
//!   no file here opens a network socket: the service is reached over a Unix
//!   socket on this machine, and there is no address form to point it anywhere
//!   else.
//! - **A person never learns the name of anything we rented**, and the first
//!   step to that is that only one file here knows it: `src/engine.rs`.
//!
//! It reads files and runs nothing, so it holds on any machine.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

/// Where this crate is.
fn this_crate() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every `.rs` file under a folder, with its text.
fn sources_under(folder: &Path, found: &mut Vec<(PathBuf, String)>) {
    for entry in fs::read_dir(folder).expect("a source folder").flatten() {
        let path = entry.path();
        if path.is_dir() {
            sources_under(&path, found);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let text = fs::read_to_string(&path).expect("a source file");
            found.push((path, text));
        }
    }
}

/// **No dependency this crate ships with can reach the network.**
#[test]
fn no_dependency_can_reach_the_network() {
    let manifest = fs::read_to_string(this_crate().join("Cargo.toml")).expect("the manifest");
    let shipped = manifest
        .split("[dev-dependencies]")
        .next()
        .expect("the manifest has a first half");
    for network in [
        "ureq",
        "reqwest",
        "hyper",
        "rustls",
        "alo-asking",
        "alo-egress",
        "alo-nearby",
        "socket2",
        "tokio",
    ] {
        assert!(
            !shipped.contains(&format!("\n{network} ")),
            "alo-converting depends on {network}, which reaches the network"
        );
    }
    assert!(
        shipped.contains("\nrustix = { workspace = true }"),
        "the manifest this reads is not the one it expects, so the check above proves nothing"
    );
}

/// **No file here opens a network socket, or knows an address.**
#[test]
fn no_file_opens_a_network_socket() {
    let mut sources = Vec::new();
    sources_under(&this_crate().join("src"), &mut sources);
    assert!(sources.len() > 10, "the crate's source was not found");
    for (path, text) in &sources {
        for network in [
            "TcpStream",
            "TcpListener",
            "UdpSocket",
            "ToSocketAddrs",
            "SocketAddr",
        ] {
            assert!(
                !text.contains(network),
                "{} names {network}; converting reaches nothing off this machine",
                path.display()
            );
        }
    }
}

/// **The engine is named in one file.** Assembled rather than written out, so
/// that this file is never the thing it looks for.
#[test]
fn the_engine_is_named_in_one_file() {
    let names = [
        format!("{}{}", "Libre", "Office"),
        format!("{}{}", "libre", "office"),
        format!("{}{}", "sof", "fice"),
    ];
    let mut sources = Vec::new();
    sources_under(&this_crate().join("src"), &mut sources);
    let naming: Vec<&PathBuf> = sources
        .iter()
        .filter(|(_, text)| names.iter().any(|name| text.contains(name.as_str())))
        .map(|(path, _)| path)
        .collect();
    match naming.as_slice() {
        [only] => assert!(
            only.ends_with("engine.rs"),
            "{} names the engine, and only engine.rs may",
            only.display()
        ),
        other => panic!("the engine is named in {other:?}, and only in engine.rs is right"),
    }
}
