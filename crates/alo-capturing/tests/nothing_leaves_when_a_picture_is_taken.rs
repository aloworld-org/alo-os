//! Taking a picture of the screen sends nothing anywhere.
//!
//! The plan's constraint for this task: *nothing is uploaded, shared or sent
//! anywhere by taking a screenshot. No automatic cloud folder, no share step
//! that is on by default.* On a machine sold on sovereignty, the one file that
//! holds everything a person had on their screen is the last file that should
//! quietly acquire a second copy somewhere.
//!
//! **There is no code here that enforces it**, and that is the point: there is
//! no code here that could break it. A picture has two destinations, both on
//! this machine, and nothing in the tree this crate pulls in can open a
//! connection.
//!
//! # Why a test reads a manifest
//!
//! Because a behaviour test can only show that today's screenshot sends
//! nothing. What stops tomorrow's is that **nothing reachable from here can
//! speak to a network**: law 1 says every egress an agent causes is visible at
//! the moment it happens, `alo-egress` and `alo-indicator` are that law as
//! working code, and the honest way for a screenshot to stay outside all of it
//! is to have nothing that carries a request. Somebody adding one would be
//! doing it for a good reason, which is exactly why it is worth failing a gate
//! over.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

/// Everything alo OS rents that carries a request off this machine, and the
/// crates of ours that exist to reach one.
const WHAT_WOULD_CARRY_IT_AWAY: [&str; 8] = [
    "ureq",
    "rustls",
    "reqwest",
    "hyper",
    "socket2",
    "alo-asking",
    "alo-nearby",
    "alo-models",
];

/// A crate's manifest, read off the disk beside this test.
fn manifest(crate_named: &str) -> String {
    let at: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the crates directory")
        .join(crate_named)
        .join("Cargo.toml");
    fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// Whether a manifest names this crate as a dependency.
///
/// A dependency is a line that names a crate and opens its table, rather than
/// any mention of the name: these manifests argue in prose about crates they do
/// not depend on, which is the argument being kept beside the thing it is about
/// and not something a check should be confused by.
fn depends_on(manifest: &str, crate_named: &str) -> bool {
    manifest
        .lines()
        .any(|line| line.trim_start().starts_with(&format!("{crate_named} = ")))
}

/// **Nothing this crate ships can carry a picture off this machine.** Not a
/// client of somebody else's, and not one of ours.
#[test]
fn nothing_this_crate_ships_can_reach_a_network() {
    let ours = manifest("alo-capturing");
    let ships = ours
        .split_once("[dev-dependencies]")
        .map_or(ours.as_str(), |(ships, _)| ships);

    for elsewhere in WHAT_WOULD_CARRY_IT_AWAY {
        assert!(
            !depends_on(ships, elsewhere),
            "alo-capturing depends on {elsewhere}: taking a picture of the screen sends \
             nothing anywhere"
        );
    }
}

/// **And nothing it depends on can either.** One step out is where a
/// dependency that carries a request would actually arrive: nobody adds `ureq`
/// to a screenshot crate, and everybody adds a crate that happens to have it.
#[test]
fn nothing_it_depends_on_can_reach_a_network_either() {
    for depended_on in [
        "alo-in-use",
        "alo-portals",
        "alo-capability",
        "alo-clipboard",
        "alo-strings",
    ] {
        let theirs = manifest(depended_on);
        let ships = theirs
            .split_once("[dev-dependencies]")
            .map_or(theirs.as_str(), |(ships, _)| ships);

        for elsewhere in WHAT_WOULD_CARRY_IT_AWAY {
            assert!(
                !depends_on(ships, elsewhere),
                "{depended_on} depends on {elsewhere}, and alo-capturing depends on \
                 {depended_on}"
            );
        }
    }
}

/// **There is no third destination**, and there is no name in this crate for
/// one. A *share* step, a cloud folder or an upload would have to be written
/// somewhere in this source, and this reads every line of it.
#[test]
fn nothing_in_this_crate_names_a_place_off_this_machine() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut read = 0_usize;
    for file in fs::read_dir(&source).expect("this crate's own source") {
        let at = file.expect("a file").path();
        if at.extension().is_none_or(|ending| ending != "rs") {
            continue;
        }
        read = read.saturating_add(1);
        let written = fs::read_to_string(&at).expect("a readable file");
        for absent in [
            "http://",
            "TcpStream",
            "UdpSocket",
            "reqwest",
            "ureq",
            "socket2",
        ] {
            assert!(!written.contains(absent), "{} names {absent}", at.display());
        }
    }
    assert!(read > 10, "only {read} files were read");
}
