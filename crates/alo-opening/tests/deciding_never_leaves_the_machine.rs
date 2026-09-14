//! No outcome is reached by asking anything off this machine, and nothing here
//! opens, converts or runs anything.
//!
//! The plan's constraint: *what is this file* must never become an errand — the
//! answer to a question a person did not know they were asking. The strongest
//! form of that promise is structural rather than behavioural: this crate has
//! no road to the network at all. So these tests read what the crate is built
//! from — its manifest and its shipped source — and fail the day either gains
//! one.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

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

/// **Nothing under this crate can reach the network**: it depends on the
/// vocabulary's machinery and an error-deriving macro and nothing else, and its
/// shipped source names no socket, no process and no path it opens itself.
#[test]
fn nothing_under_this_crate_can_reach_the_network() {
    let manifest =
        fs::read_to_string(this_crate().join("Cargo.toml")).expect("this crate's manifest");
    assert_eq!(
        dependencies_in(&manifest, "[dependencies]"),
        ["alo-strings", "thiserror"],
        "alo-opening gained a dependency, and deciding what a file is must not be able to \
         become an errand: whatever that dependency can reach, this crate now can"
    );

    let mut sources = Vec::new();
    sources_under(&this_crate().join("src"), &mut sources);
    assert!(sources.len() > 5, "the shipped source was not found");
    for source in sources {
        let written = fs::read_to_string(&source).expect("a source file");
        for reaching in [
            "std::net",
            "TcpStream",
            "UdpSocket",
            "std::process",
            "Command::new",
            "File::open",
            "File::create",
            "fs::read",
            "fs::write",
            "OpenOptions",
            "alo_egress",
            "ureq",
        ] {
            assert!(
                !written.contains(reaching),
                "{} names `{reaching}`: deciding what a file is reads the file it was handed \
                 and nothing else",
                source.display()
            );
        }
    }
}

/// The dependency reader above reads what it is shown, so that the guarantee
/// is not a parser that never found a dependency to refuse.
#[test]
fn a_dependency_added_to_the_manifest_would_be_seen() {
    let widened = "[package]\nname = \"x\"\n\n[dependencies]\n# a comment\nalo-strings = { path \
                   = \"../alo-strings\" }\nthiserror = { workspace = true }\nureq = \"3\"\n\n\
                   [dev-dependencies]\nalo-saying = { path = \"../alo-saying\" }\n";
    assert_eq!(
        dependencies_in(widened, "[dependencies]"),
        ["alo-strings", "thiserror", "ureq"]
    );
    assert_eq!(
        dependencies_in(widened, "[dev-dependencies]"),
        ["alo-saying"]
    );
}
