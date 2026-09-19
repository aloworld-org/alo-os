//! *Sunset for your location* is arithmetic here, and on most systems it is a
//! request.
//!
//! The ordinary way to know when the sun sets where somebody is, is to send
//! their address — or the list of wireless networks their machine can hear — to
//! a service that answers with a latitude, and then to ask a second service for
//! the sunset. Two requests, and one of them tells a stranger which building a
//! person is sitting in. `crates/alo-displays/src/sun.rs` does it with a series
//! approximation instead, from two numbers the person typed into Settings.
//!
//! *There is no network call in it* is a sentence about today's code, and the
//! failure it guards against is tomorrow's: somebody adding *look my town up
//! for me* because typing a latitude is awkward, which is a kindness that
//! quietly turns a sovereign machine into one that phones out every evening. So
//! this reads the crate's own source and its manifest for every road out of the
//! machine this workspace has, and fails on any of them.

#![expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, UNIX_EPOCH};

use alo_displays::{Moment, Sun, Whereabouts};

/// This crate's directory.
fn here() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every Rust file under `src`, with its text.
fn the_source() -> Vec<(PathBuf, String)> {
    let Ok(entries) = fs::read_dir(here().join("src")) else {
        panic!("this crate's source could not be read");
    };
    let mut found = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|extension| extension == "rs") {
            let Ok(text) = fs::read_to_string(&path) else {
                panic!("{} could not be read", path.display());
            };
            found.push((path, text));
        }
    }
    found
}

/// The lines of a file that are code rather than documentation.
///
/// Prose about the network is exactly what this crate should be full of — the
/// argument for why there is none lives in the files themselves — so a comment
/// line, and the one attribute that carries this repository's own address, are
/// not code.
fn code_of(text: &str) -> String {
    text.lines()
        .filter(|line| {
            let line = line.trim_start();
            !line.starts_with("//") && !line.starts_with("#![doc(html_root_url")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Everything in this workspace that reaches a network, and everything that
/// would reach the machinery that does.
///
/// Deliberately not `socket`: a screen is plugged into one, and this crate says
/// so several hundred times.
const ROADS_OFF_THIS_MACHINE: [&str; 17] = [
    // The standard library's own.
    "std::net",
    "TcpStream",
    "TcpListener",
    "UdpSocket",
    "ToSocketAddrs",
    "IpAddr",
    // The crates this workspace pins for the two services that are allowed to
    // speak, and the ones a hurried change would reach for instead.
    "ureq",
    "socket2",
    "reqwest",
    "hyper",
    "curl",
    // An address, however it is spelled.
    "http://",
    "https://",
    // Somebody else answering the question this file answers.
    "geoclue",
    "GeoClue",
    "getaddrinfo",
    // And a process that could ask on this crate's behalf.
    "Command::new",
];

/// **Nothing in this crate's code names a road off this machine.**
#[test]
fn nothing_here_can_reach_a_network() {
    let source = the_source();
    assert!(
        source.iter().any(|(path, _)| path.ends_with("sun.rs")),
        "the arithmetic was not where this test looks for it"
    );
    for (path, text) in source {
        let code = code_of(&text);
        for road in ROADS_OFF_THIS_MACHINE {
            assert!(
                !code.contains(road),
                "{} names {road}: where a person is, is what they typed and nothing else",
                path.display()
            );
        }
    }
}

/// **And nothing it depends on is there to reach one with.** The manifest names
/// no client, no socket library and no bus.
#[test]
fn nothing_this_crate_depends_on_is_a_way_off_this_machine() {
    let Ok(manifest) = fs::read_to_string(here().join("Cargo.toml")) else {
        panic!("this crate's manifest could not be read");
    };
    let dependencies: Vec<&str> = manifest
        .lines()
        .filter(|line| !line.trim_start().starts_with('#'))
        .collect();
    for forbidden in [
        "ureq", "socket2", "reqwest", "hyper", "curl", "zbus", "rustix", "nix", "libc",
    ] {
        assert!(
            !dependencies.iter().any(|line| {
                let line = line.trim_start();
                line.starts_with(&format!("{forbidden} "))
                    || line.starts_with(&format!("{forbidden}="))
            }),
            "this crate depends on {forbidden}"
        );
    }
}

/// **The reader above is looking at real code.** One that found nothing would
/// pass on an empty directory, so it is shown refusing a line that would go and
/// ask somebody where the person is.
#[test]
fn the_reader_refuses_a_line_that_asks_somebody_else() {
    let written = "    let here = ureq::get(\"https://example.invalid/where-am-i\").call();\n";
    assert!(
        ROADS_OFF_THIS_MACHINE
            .iter()
            .any(|road| code_of(written).contains(road))
    );
    assert!(
        !code_of("// https:// in a comment is an argument, not a request").contains("https://"),
        "prose about the network is what these files are full of"
    );
    assert!(here().join("src").is_dir());
}

/// **And the arithmetic it is guarding actually answers.** A test that proved
/// only an absence would still pass if the calculation were deleted, so this
/// asks it the question a person asks: when does the sun set here tonight —
/// London on the longest day of 2026, which an almanac puts at 21:21 British
/// summer time.
#[test]
fn the_arithmetic_answers_without_asking_anybody() {
    let london = Whereabouts::typed(51.5074, -0.1278).unwrap();
    // 2026-06-21 at noon, British summer time.
    let noon = Moment::at(
        UNIX_EPOCH + Duration::from_secs(20_625 * 24 * 60 * 60 + 11 * 60 * 60),
        60,
    );
    let Sun::SetsAndRises(night) = london.sun_on(noon) else {
        panic!("the sun sets in London in June");
    };
    assert_eq!(night.begins().hour(), 21, "sunset was {}", night.begins());
    assert_eq!(night.ends().hour(), 4, "sunrise was {}", night.ends());
}
