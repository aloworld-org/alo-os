//! The several-screens promise that is about shape, read from the source this
//! crate ships.
//!
//! The plan's constraint for *several displays, and a background and a dock on
//! each* is that **the arrangement is `alo-displays`' and is never adjusted
//! here**. A test that drew two screens would only show the desk it was handed.
//! This one reads the files instead, so a compositor that works out where a
//! screen goes, how large it draws, or which one is the main one — or that
//! names a colour, or writes to the crates that own what a screen wears — is a
//! failing build.
//!
//! Comments and string literals' contents are taken out first, and unit tests
//! are not read: every `#[cfg(test)]` module in these files is the last thing
//! in its file, and `*_tests.rs` and `*_testing.rs` files are tests.
#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// This crate's source directory.
fn src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// The files that make up the desk: their names and their code.
fn the_screen_files() -> Vec<(String, Vec<(usize, String)>)> {
    let mut read = Vec::new();
    for entry in std::fs::read_dir(src()).unwrap() {
        let at = entry.unwrap().path();
        let named = at.file_name().unwrap().to_string_lossy().into_owned();
        let is_one = named.starts_with("screen_") || named.starts_with("screens");
        let is_a_test = named.ends_with("_tests.rs") || named.ends_with("_testing.rs");
        if is_one && !is_a_test {
            read.push((named, the_code_of(&std::fs::read_to_string(&at).unwrap())));
        }
    }
    read.sort();
    let names: Vec<&str> = read.iter().map(|(named, _)| named.as_str()).collect();
    assert_eq!(
        names,
        ["screen_background.rs", "screens.rs", "screens_raster.rs"],
        "a file of the desk was added or lost, and this test has to be told"
    );
    read
}

/// The code of a file: no unit tests, no comments, no string literals.
fn the_code_of(written: &str) -> Vec<(usize, String)> {
    let mut code = Vec::new();
    let mut inside = false;
    let mut escaped = false;
    for (which, line) in written.lines().enumerate() {
        if line.trim_start().starts_with("#[cfg(test)]") {
            break;
        }
        if !inside && line.trim_start().starts_with("//") {
            continue;
        }
        let outside = outside_strings(line, &mut inside, &mut escaped);
        let code_part = match outside.split_once("//") {
            Some((before, _)) => before.to_owned(),
            None => outside,
        };
        code.push((which + 1, code_part));
    }
    code
}

/// One line with the contents of its string literals taken out.
fn outside_strings(line: &str, inside: &mut bool, escaped: &mut bool) -> String {
    let mut outside = String::new();
    let mut previous = None;
    for letter in line.chars() {
        if *inside {
            if *escaped {
                *escaped = false;
            } else if letter == '\\' {
                *escaped = true;
            } else if letter == '"' {
                *inside = false;
                outside.push('"');
            }
            previous = Some(letter);
            continue;
        }
        if letter == '"' && previous != Some('\'') {
            *inside = true;
        }
        outside.push(letter);
        previous = Some(letter);
    }
    *escaped = false;
    outside
}

/// **The arrangement is never made, adjusted or remembered here.** Nothing in
/// the desk's files builds an arrangement, places a screen, invents a position
/// or a size, or writes to the file a person's screens are remembered in —
/// every one of those is `alo-displays`', asked once and drawn from.
#[test]
fn the_arrangement_is_never_adjusted_here() {
    for (named, code) in the_screen_files() {
        for (which, line) in code {
            for making in [
                "Arrangement::",
                "Placed::",
                "Position::",
                "Scale::",
                "Screens::of(",
                "Attached::now",
                ".remember(",
                ".forget(",
                ".forget_everything(",
                "Changes::untouched",
            ] {
                assert!(
                    !line.contains(making),
                    "{named}:{which} adjusts the arrangement (`{making}`): {line}"
                );
            }
        }
    }
}

/// **Nothing here writes to the crates that own what a screen wears.** A
/// background, an accent, a text size and a dock's edge are read from
/// `alo-appearance` and `alo-dock` and written back by the surfaces that own
/// those settings, never by the one that draws the desk — with the single
/// exception of the per-screen dock this file lays out, whose edge is taken
/// from that screen's own `Wearing` and from nowhere else.
#[test]
fn nothing_here_writes_to_the_crates_that_own_what_a_screen_wears() {
    for (named, code) in the_screen_files() {
        for (which, line) in code {
            for writing in [
                "set_background",
                "set_accent",
                "set_text",
                ".follow(",
                "put_back",
                "put_everything_back",
                "keeping::",
                "::write",
            ] {
                assert!(
                    !line.contains(writing),
                    "{named}:{which} writes a setting it only draws (`{writing}`): {line}"
                );
            }
            assert!(
                !line.contains("set_edge") || named == "screens_raster.rs",
                "{named}:{which} moves the dock: {line}"
            );
        }
    }
}

/// **No colour is named here.** Every colour on a screen is `alo-appearance`'s
/// token or the person's own background, warmed by `alo-displays`' own
/// arithmetic — so there is no second palette in the compositor that could
/// disagree with the one a person chose, and no terracotta reachable from the
/// desk.
#[test]
fn no_colour_is_named_here() {
    for (named, code) in the_screen_files() {
        for (which, line) in code {
            for naming in ["Token::", "Accent::", "Scheme::", "0x"] {
                assert!(
                    !line.contains(naming),
                    "{named}:{which} names a colour (`{naming}`): {line}"
                );
            }
        }
    }
}
