//! **A grade is the person's machine's, and it travels nowhere.**
//!
//! Task 4 of `docs/autonomy/v0-5-the-models-measured-plan.md` asks for this to
//! be a test that reads the crate's shipped source, rather than a sentence. A
//! measurement of a person's own weights is written into their own settings by
//! `Choosing::measuring` and read back by `Settings::at`; this reads every file
//! this crate ships and holds it to having no way to put anything on a network
//! — no socket, no HTTP client, and no road to a model runtime or a provider —
//! so the file on their disk is the only place a grade can go.
//!
//! A crate that gained one would be asked, here, why a settings store needs it.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

/// What would let this crate send something somewhere.
///
/// The runtime is matched as the type is spelled in code — `Ollama::`, or named
/// in a `use` list — and not as a word, because a grade's own record names the
/// runtime it was measured under (`runtime = "Ollama 0.34.0"`), which is a
/// statement about a program and not a road to one.
const A_WAY_OFF_THE_MACHINE: [&str; 14] = [
    "std::net",
    "TcpStream",
    "UdpSocket",
    "ureq",
    "reqwest",
    "hyper",
    "Ollama::",
    "Ollama,",
    "Ollama}",
    "found_on_this_machine",
    "ModelRuntime",
    "Asking",
    "to_a_provider",
    "Departing",
];

/// This crate's shipped source, file by file.
fn the_shipped_source() -> Vec<(PathBuf, String)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    let mut folders = vec![root];
    while let Some(folder) = folders.pop() {
        for entry in fs::read_dir(&folder).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                folders.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                let text = fs::read_to_string(&path).unwrap();
                files.push((path, text));
            }
        }
    }
    files
}

/// The lines of a file that are code rather than documentation about it.
fn code_in(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .map(str::trim_start)
        .filter(|line| !line.starts_with("//"))
}

#[test]
fn nothing_this_crate_ships_can_send_a_grade_anywhere() {
    let source = the_shipped_source();
    assert!(
        source.iter().any(|(path, _)| path.ends_with("choosing.rs")),
        "the source was not found, so nothing was read"
    );
    for (path, text) in &source {
        for line in code_in(text) {
            for way in A_WAY_OFF_THE_MACHINE {
                assert!(
                    !line.contains(way),
                    "{} can reach off this machine through `{way}`: {line}",
                    path.display()
                );
            }
        }
    }
}

#[test]
fn nothing_this_crate_depends_on_is_a_client_of_its_own() {
    let manifest =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")).unwrap();
    let dependencies = manifest
        .split("[dependencies]")
        .nth(1)
        .unwrap()
        .split("\n[")
        .next()
        .unwrap();
    for client in [
        "ureq",
        "reqwest",
        "hyper",
        "alo-asking",
        "alo-egress",
        "alo-nearby",
    ] {
        assert!(
            !code_in(dependencies).any(|line| line.starts_with(client)),
            "alo-choosing depends on `{client}`"
        );
    }
}

/// **And the check catches a crate that could**, which is the half a green run
/// cannot show.
#[test]
fn the_check_would_notice_a_socket() {
    let written = "use std::net::TcpStream;\n// std::net in a comment is not code\n";
    let caught: Vec<&str> = code_in(written)
        .filter(|line| A_WAY_OFF_THE_MACHINE.iter().any(|way| line.contains(way)))
        .collect();
    assert_eq!(caught, vec!["use std::net::TcpStream;"]);
}
