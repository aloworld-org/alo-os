//! The egress indicator's promises that are about shape, read from the source
//! this crate ships.
//!
//! The plan's constraint for *the egress indicator, on a screen* is three
//! sentences, and none of them is a behaviour a test could walk into: **the
//! indicator shows; it never decides**, it puts **no words of its own** on a
//! screen, and there is **no dismissing, no hiding, no setting that turns it
//! off**. A test that drew a lit indicator would only show the roads it took.
//! These read the files instead, so a policy consulted from a drawing file, an
//! English sentence typed into one, or a `hide` added tomorrow is a failing
//! build rather than a quieter machine.
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

/// The files that make up the egress indicator: their names and their code.
fn the_egress_status_files() -> Vec<(String, Vec<(usize, String)>)> {
    let mut read = Vec::new();
    for entry in std::fs::read_dir(src()).unwrap() {
        let at = entry.unwrap().path();
        let named = at.file_name().unwrap().to_string_lossy().into_owned();
        let is_one = named.starts_with("egress_status") || named == "nested_egress_status.rs";
        let is_a_test = named.ends_with("_tests.rs") || named.ends_with("_testing.rs");
        if is_one && !is_a_test {
            read.push((named, the_code_of(&std::fs::read_to_string(&at).unwrap())));
        }
    }
    read.sort();
    let names: Vec<&str> = read.iter().map(|(named, _)| named.as_str()).collect();
    assert_eq!(
        names,
        [
            "egress_status.rs",
            "egress_status_mark.rs",
            "egress_status_paint.rs",
            "egress_status_place.rs",
            "egress_status_raster.rs",
            "nested_egress_status.rs",
        ],
        "an egress indicator file was added or lost, and this test has to be told"
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

/// Whether `line` uses `word` as a whole identifier.
fn names(line: &str, word: &str) -> bool {
    line.match_indices(word).any(|(at, _)| {
        let before = line.get(..at).and_then(|up_to| up_to.chars().next_back());
        let after = line.get(at + word.len()..).and_then(|on| on.chars().next());
        let part_of_a_word = |letter: Option<char>| {
            letter.is_some_and(|letter| letter.is_alphanumeric() || letter == '_')
        };
        !part_of_a_word(before) && !part_of_a_word(after)
    })
}

/// **The indicator shows; it never decides.** No egress indicator file asks a
/// policy, begins or ends a departure, or makes one: whether something may
/// leave is `alo-egress`'s, asked before a socket opens, and a drawing file
/// that could reach any of it would be a second place to decide.
#[test]
fn the_egress_indicator_shows_and_never_decides() {
    for (named, code) in the_egress_status_files() {
        for (which, line) in code {
            for deciding in [
                "EgressPolicy",
                "SourcePolicy",
                "Departing",
                "Underway",
                "Leaving",
                "OnItsOwn",
                "NotPermitted",
                "beginning",
                "beginning_on_its_own",
                "ended",
                "ended_on_its_own",
                "alo_egress",
                "alo_capability",
            ] {
                assert!(
                    !names(&line, deciding),
                    "{named}:{which} reaches `{deciding}`, which decides: {line}"
                );
            }
        }
    }
}

/// **The indicator writes no words of its own.** Every line is `alo-egress`'s
/// sentence, handed over through `alo_indicator::Drawn`; nothing in these files
/// declares a word, fills a gap, looks a key up, or holds a string literal a
/// person could end up reading.
#[test]
fn the_egress_indicator_writes_no_words_of_its_own() {
    for (named, code) in the_egress_status_files() {
        for (which, line) in code {
            for writing in [
                "Word::",
                "Vocabulary",
                "Filling",
                ".say(",
                ".count(",
                "Key::",
                "format!",
                "\"",
            ] {
                assert!(
                    !line.contains(writing),
                    "{named}:{which} writes its own words (`{writing}`): {line}"
                );
            }
        }
    }
}

/// **Nothing can dismiss it, hide it, or turn it off.** No function in these
/// files is named for any of that, nothing could say it is off, and the public
/// surface is held to a list — so a way to take a picture away cannot be added
/// without this test being told.
#[test]
fn nothing_can_dismiss_hide_or_turn_off_the_egress_indicator() {
    let mut functions = Vec::new();
    for (named, code) in the_egress_status_files() {
        for (which, line) in code {
            let lowered = line.to_lowercase();
            for off in [
                "hide", "hidden", "dismiss", "mute", "silence", "disable", "enabled", "visible",
                "turn_off", "snooze", "opacity", "alpha",
            ] {
                assert!(
                    !lowered.contains(off),
                    "{named}:{which} has a way to take the indicator away (`{off}`): {line}"
                );
            }
            if let Some(after) = line.trim().split("fn ").nth(1) {
                let function: String = after
                    .chars()
                    .take_while(|letter| letter.is_alphanumeric() || *letter == '_')
                    .collect();
                if line.trim_start().starts_with("pub fn ") {
                    functions.push(function);
                }
            }
        }
    }
    functions.sort();
    assert_eq!(
        functions,
        [
            "is_told",
            "on_an_output",
            "submit_with_egress_status",
            "with_no_output",
        ],
        "the egress indicator's public surface changed, and this test has to be told"
    );
}
