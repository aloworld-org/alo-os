//! **This crate has no road to the network, and a test keeps it that way.**
//!
//! An adapted model carries the documents it was trained on — the sentences can
//! be drawn back out of the weights — so a road from here to a socket is a road
//! from somebody's correspondence to somebody else's computer, whatever the
//! code calls it.
//!
//! # If this test has stopped you
//!
//! **Adding a client here is not the fix.** What you are about to send is the
//! person's documents in another shape, so it is an `alo_egress` departure with
//! the indicator lit and a record written — and the errand for it **does not
//! exist yet** in that crate's closed list. `alo_adapting::leaving` says what
//! that errand would have to say, and why alo's own service is not an exception
//! (ADR 0014).
//!
//! Until somebody decides it, this crate reads folders and writes a working
//! folder, and nothing else.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// Every way a crate reaches the network, or asks somebody else to.
const A_ROAD_OFF_THIS_MACHINE: [&str; 12] = [
    "std::net",
    "TcpStream",
    "UdpSocket",
    "ureq",
    "reqwest",
    "hyper",
    "socket2",
    "alo_asking",
    "alo_egress",
    "Departing",
    "Hosted",
    "upload",
];

#[test]
fn no_source_file_in_this_crate_can_send_anything() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut roads: Vec<String> = Vec::new();
    for file in every_file(&src) {
        let text = std::fs::read_to_string(&file).expect("this crate's own source");
        for line in text.lines() {
            let code = only_the_code(line);
            for road in A_ROAD_OFF_THIS_MACHINE {
                if code.contains(road) {
                    roads.push(format!("{}: {}", file.display(), line.trim()));
                }
            }
        }
    }
    assert!(
        roads.is_empty(),
        "this crate has grown a road to the network, and what it would carry is the documents a \
         fine-tune was trained on:\n{roads:#?}\n\nSending an adapted model is an alo_egress \
         departure with an errand that does not exist yet — see alo_adapting::leaving — and not \
         a client added here."
    );
}

/// A line with its documentation and its string literals taken out, so that a
/// file which *names* a road in order to forbid it is not read as taking one.
///
/// This crate's own `leaving.rs` says the words `alo_egress` and *upload* on
/// purpose, in the sentence explaining why neither may appear in code here.
fn only_the_code(line: &str) -> String {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return String::new();
    }
    let mut code = String::new();
    let mut inside_a_string = false;
    let mut letters = line.chars().peekable();
    while let Some(letter) = letters.next() {
        match letter {
            '\\' if inside_a_string => {
                letters.next();
            }
            '"' => inside_a_string = !inside_a_string,
            _ if !inside_a_string => code.push(letter),
            _ => {}
        }
    }
    code
}

/// Every `.rs` under a folder.
fn every_file(folder: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(folder) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(every_file(&path));
        } else if path.extension().is_some_and(|kind| kind == "rs") {
            found.push(path);
        }
    }
    found
}

/// **And the crate declares no dependency that could send one either.**
#[test]
fn the_manifest_carries_nothing_that_reaches_the_network() {
    let manifest =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .expect("this crate's own manifest");
    let dependencies = manifest
        .split("[dependencies]")
        .nth(1)
        .unwrap_or_default()
        .split("\n[")
        .next()
        .unwrap_or_default();
    for reaching in [
        "ureq",
        "reqwest",
        "hyper",
        "socket2",
        "alo-asking",
        "alo-egress",
    ] {
        assert!(
            !dependencies.contains(reaching),
            "alo-adapting depends on {reaching}, which can reach off this machine"
        );
    }
}
