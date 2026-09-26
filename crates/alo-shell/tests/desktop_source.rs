//! The ordinary desktop's promises that are about shape, read from the source
//! this crate ships.
//!
//! The plan's constraint for *the ordinary desktop* is that **the dock is
//! furniture and holds no authority** — nothing is granted, approved or revoked
//! from it — and that **what the status area and the windows show about the
//! machine is measured by the crates that measure, and this surface adds no
//! number of its own**. Its acceptance adds that **terracotta is never
//! offered** and that **light and dark are `alo-appearance`'s**, never this
//! crate's. A test that drew a desktop would only show the frames it drew.
//! These read the files instead, so a revoke button on the dock, a byte count
//! rounded into gigabytes, a kill key or a desktop that picks its own scheme is
//! a failing build.
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

/// The files that make up the ordinary desktop: their names and their code.
fn the_desktop_files() -> Vec<(String, Vec<(usize, String)>)> {
    let mut read = Vec::new();
    for entry in std::fs::read_dir(src()).unwrap() {
        let at = entry.unwrap().path();
        let named = at.file_name().unwrap().to_string_lossy().into_owned();
        let is_one = ["desktop_", "dock_", "running_", "filling_"]
            .iter()
            .any(|prefix| named.starts_with(prefix))
            || named == "nested_desktop.rs";
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
            "desktop_list.rs",
            "desktop_look.rs",
            // Which desktop is shown and which windows are on it. Not a
            // surface that draws, but held to the same promises: neither
            // grants anything, neither measures anything, and a switch that
            // picked its own scheme or counted its own bytes would be caught
            // here as surely as a dock button that revoked a grant.
            "desktop_membership.rs",
            "desktop_paint.rs",
            "desktop_raster.rs",
            "desktop_seat.rs",
            "desktop_swipes.rs",
            "dock_raster.rs",
            "filling_keys.rs",
            "filling_rows.rs",
            "filling_window.rs",
            "nested_desktop.rs",
            "running_keys.rs",
            "running_rows.rs",
            "running_window.rs",
        ],
        "a desktop file was added or lost, and this test has to be told"
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

/// **The dock grants, approves and revokes nothing.** Nothing in the desktop's
/// files names the capability model, a grant, a pairing, the approval surface
/// or an agent — so the dock and its windows are furniture, and there is no
/// road from them to a change a person did not make somewhere that asks.
#[test]
fn the_dock_grants_approves_and_revokes_nothing() {
    for (named, code) in the_desktop_files() {
        for (which, line) in code {
            for crate_name in [
                "alo_capability",
                "alo_granted",
                "alo_picking",
                "alo_nearby",
                "alo_approving",
                "alo_turn",
                "alo_overlay",
                "alo_agentd",
                "alo_asking",
                "alo_protocol",
                "alo_setting_up",
                "alo_choosing",
            ] {
                assert!(
                    !line.contains(crate_name),
                    "{named}:{which} reaches for authority (`{crate_name}`): {line}"
                );
            }
            for word in [
                "grant",
                "revoke",
                "approve",
                "approving",
                "decline",
                "propose",
                "pair",
            ] {
                assert!(
                    !line.to_lowercase().contains(word),
                    "{named}:{which} holds authority (`{word}`): {line}"
                );
            }
            assert!(
                !names(&line, "Measured") && !names(&line, "Measurement"),
                "{named}:{which} takes the agent's road to a measurement: {line}"
            );
        }
    }
}

/// **The desktop adds no number of its own.** Every number drawn is the digits
/// of a number `alo-measuring` holds — `value`, `pid` or `size`, turned into
/// text in exactly the two files that make rows and nowhere else — and those
/// files neither scale, round, sum, average, sort nor leave anything out.
/// Nothing in the desktop reads the kernel or the disk for itself except
/// through `alo-measuring`'s two doors, and nothing words a sentence.
#[test]
fn the_desktop_adds_no_number_of_its_own() {
    let mut turned_into_text = Vec::new();
    for (named, code) in the_desktop_files() {
        for (which, line) in code {
            for writing in [
                "Word::",
                "Vocabulary",
                "Filling::",
                ".say(",
                "strings.count(",
                "Key::named",
                "format!",
                "push_str",
                "concat",
                ".join(",
                "summar",
            ] {
                assert!(
                    !line.contains(writing),
                    "{named}:{which} writes its own words (`{writing}`): {line}"
                );
            }
            assert!(
                !line.contains('"') || line.trim_start().starts_with("reason = "),
                "{named}:{which} holds a string a person could read: {line}"
            );
            for reading in ["/proc", "fs::", "File::", "read_dir", "metadata", "Disk"] {
                assert!(
                    !line.contains(reading),
                    "{named}:{which} reads the machine for itself (`{reading}`): {line}"
                );
            }
            if line.contains("to_string") {
                turned_into_text.push((named.clone(), line.trim().to_owned()));
            }
            if named.ends_with("_rows.rs") {
                for deriving in [
                    ".sum(", ".max(", ".min(", " / ", " * ", "1024", "1000", "round", "sort",
                    "retain", ".filter(", ".skip(", ".take(", "as f", "percent", "dedup",
                ] {
                    assert!(
                        !line.contains(deriving),
                        "{named}:{which} derives from what was measured (`{deriving}`): {line}"
                    );
                }
            }
        }
    }
    for (named, line) in &turned_into_text {
        assert!(
            named == "running_rows.rs" || named == "filling_rows.rs",
            "{named} turns a number into text: {line}"
        );
        assert!(
            ["value.to_string()", "pid.to_string()", "size.to_string()"]
                .iter()
                .any(|number| line.contains(number)),
            "{named} turns something other than a measured number into text: {line}"
        );
    }
    assert_eq!(turned_into_text.len(), 4, "{turned_into_text:#?}");

    let mut measured = Vec::new();
    for (named, code) in the_desktop_files() {
        for (_, line) in code {
            if line.contains("Holding::of(") || line.contains(".since(") {
                measured.push(named.clone());
            }
        }
    }
    assert_eq!(measured, ["filling_window.rs", "running_window.rs"]);
}

/// **Neither window acts on what it shows.** Nothing in the desktop's files
/// signals, stops or reprioritises a process, and nothing deletes, moves,
/// renames or empties a file — a measurement ends at the answer.
#[test]
fn neither_window_acts_on_what_it_shows() {
    for (named, code) in the_desktop_files() {
        for (which, line) in code {
            for acting in [
                "kill",
                "signal",
                "Signal",
                "nice",
                "Command",
                "remove_file",
                "remove_dir",
                "rename",
                "OpenOptions",
                "set_permissions",
                "unlink",
                "truncate",
                "libc",
                "rustix",
            ] {
                assert!(
                    !names(&line, acting),
                    "{named}:{which} acts on the machine (`{acting}`): {line}"
                );
            }
        }
    }
}

/// **Light and dark are `alo-appearance`'s, and terracotta is never offered.**
/// A `DesktopLook` is built in one place, from `Appearance::scheme_at` and
/// `Appearance::accent_at`; a scheme is only ever matched on, never chosen; the
/// accent enters a palette only through `Accent::of_colour`, the door that
/// refuses terracotta; and no desktop file names terracotta at all.
#[test]
fn light_and_dark_are_alo_appearances_and_terracotta_is_never_offered() {
    let mut built = Vec::new();
    let mut accents = Vec::new();
    for (named, code) in the_desktop_files() {
        for (which, line) in code {
            assert!(
                !line.contains("Terracotta"),
                "{named}:{which} names terracotta: {line}"
            );
            for scheme in ["Scheme::Light", "Scheme::Dark"] {
                if line.contains(scheme) {
                    assert!(
                        named == "desktop_look.rs"
                            && line.trim_start().starts_with(scheme)
                            && line.contains("=>"),
                        "{named}:{which} chooses a scheme rather than matching on one: {line}"
                    );
                }
            }
            if line.contains("scheme_at(") || line.contains("accent_at(") {
                built.push((named.clone(), line.trim().to_owned()));
            }
            if line.contains("Accent::") {
                accents.push((named.clone(), line.trim().to_owned()));
            }
            assert!(
                !line.contains("Colour::of(") && !line.contains("Colour::written("),
                "{named}:{which} invents a colour: {line}"
            );
        }
    }
    assert_eq!(
        built,
        [
            (
                "desktop_look.rs".to_owned(),
                "scheme: appearance.scheme_at(now),".to_owned()
            ),
            (
                "desktop_look.rs".to_owned(),
                "accent: appearance.accent_at(now),".to_owned()
            ),
        ]
    );
    assert_eq!(
        accents,
        [(
            "desktop_look.rs".to_owned(),
            "let accent = Accent::of_colour(accent)?.on(scheme);".to_owned()
        )]
    );
}
