//! The record window's promises that are about shape, read from the source this
//! crate ships.
//!
//! The plan's constraint for *what the machine did, in front of the person* is
//! three things the window must never have — **it writes nothing to the
//! record, it has no filter that could hide a refusal and no search that
//! changes what today means, and it has no summary** — and two roads that must
//! stay one: the plain way a person opens it, which passes through no agent
//! (ADR 0009), and asking the agent *what did you do?*, which reaches the same
//! account. A test that opened the window would only show the roads it walked.
//! These read the files instead, so a filter added tomorrow, a second telling,
//! a write to the record or a road that goes through a turn is a failing build.
//!
//! Comments and string literals' contents are taken out first, and unit tests
//! are not read: every `#[cfg(test)]` module in these files is the last thing
//! in its file, and `*_tests.rs` and `*_testing.rs` files are tests.
#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None, Err or missing door is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// This crate's source directory.
fn src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// The files that make up the record window: their names and their code.
fn the_record_files() -> Vec<(String, Vec<(usize, String)>)> {
    let mut read = Vec::new();
    for entry in std::fs::read_dir(src()).unwrap() {
        let at = entry.unwrap().path();
        let named = at.file_name().unwrap().to_string_lossy().into_owned();
        let is_one = named.starts_with("record_") || named == "nested_record.rs";
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
            "nested_record.rs",
            "record_keys.rs",
            "record_lines.rs",
            "record_paint.rs",
            "record_raster.rs",
            "record_room.rs",
            "record_seat.rs",
            "record_shown.rs",
            "record_window.rs",
        ],
        "a record window file was added or lost, and this test has to be told"
    );
    read
}

/// The code of one of those files, by name.
fn the_code_of_file(file: &str) -> Vec<(usize, String)> {
    the_record_files()
        .into_iter()
        .find(|(named, _)| named == file)
        .map(|(_, code)| code)
        .unwrap()
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

/// **The window writes no words of its own, and so no summary.** Every clause,
/// remark and refusal is `alo-recounting`'s or the record's; nothing in these
/// files declares a word, fills a gap, looks a key up, formats text or holds a
/// string literal a person could end up reading. The one literal allowed is a
/// lint's `reason`, which no person using the machine reads.
#[test]
fn the_record_window_writes_no_words_of_its_own() {
    for (named, code) in the_record_files() {
        for (which, line) in code {
            for writing in [
                "Word::",
                "Vocabulary",
                "Filling",
                ".say(",
                "Key::named",
                "format!",
                "to_string",
                "push_str",
                "concat",
                ".join(",
                "summar",
            ] {
                assert!(
                    !line.to_lowercase().contains(&writing.to_lowercase()),
                    "{named}:{which} writes its own words (`{writing}`): {line}"
                );
            }
            assert!(
                !line.contains('"') || line.trim_start().starts_with("reason = "),
                "{named}:{which} holds a string a person could read: {line}"
            );
        }
    }
}

/// **The surface reads the record and writes nothing to it.** Nothing in these
/// files opens a file to write, keeps an entry, shortens a record or names the
/// types that do — so the window is a reading of the evidence and never a
/// keyboard attached to it.
#[test]
fn the_record_window_writes_nothing_to_the_record() {
    for (named, code) in the_record_files() {
        for (which, line) in code {
            for writer in [
                "Writing",
                "Kept",
                "Shortening",
                "OpenOptions",
                "Entry",
                "Record",
            ] {
                assert!(
                    !names(&line, writer),
                    "{named}:{which} names a writer of the record (`{writer}`): {line}"
                );
            }
            for writing in [
                ".keep(",
                ".shorten(",
                "fs::",
                "File::",
                "write_all",
                "set_len",
                "alo_keeping",
            ] {
                assert!(
                    !line.contains(writing),
                    "{named}:{which} could write to the record (`{writing}`): {line}"
                );
            }
        }
    }
}

/// **No filter that could hide a refusal, and no search that changes what
/// today means.** The whole surface asks the record one question —
/// `Asking::anything()`, once, at `AtMost::ONE_SITTING` — and nothing narrows
/// it, searches it, or drops a line from what it answered.
#[test]
fn no_filter_that_could_hide_a_refusal_and_no_search() {
    let mut asked = Vec::new();
    for (named, code) in the_record_files() {
        for (which, line) in code {
            let lowered = line.to_lowercase();
            for narrowing in [
                ".only(",
                ".by(",
                "only::",
                ".filter(",
                "filter_map",
                "filtering",
                "search",
                "find",
                "query",
                "matching",
                "today",
                "since",
                "until",
                "retain",
                "take_while",
                "skip_while",
                "atmost::entries",
                "hide",
                "hidden",
                "collapse",
                "refusals",
            ] {
                assert!(
                    !lowered.contains(narrowing),
                    "{named}:{which} narrows what the record answered (`{narrowing}`): {line}"
                );
            }
            if line.contains("Asking::") {
                asked.push((named.clone(), line.trim().to_owned()));
            }
            if line.contains("AtMost::") {
                assert!(
                    line.contains("AtMost::ONE_SITTING"),
                    "{named}:{which}: {line}"
                );
            }
        }
    }
    assert_eq!(asked.len(), 1, "{asked:#?}");
    let (named, line) = asked.first().unwrap();
    assert_eq!(named, "record_window.rs");
    assert!(line.contains("&Asking::anything()"), "{line}");
}

/// **Both roads reach one account.** The whole surface calls
/// `Recounting::show` once, in the window's private read, and the plain road,
/// the agent's road and reading again all go through that one read — so asking
/// the agent cannot reach a second telling, and there is no second door to the
/// record.
#[test]
fn both_roads_reach_the_one_account() {
    let mut shown = Vec::new();
    let mut read = Vec::new();
    for (named, code) in the_record_files() {
        for (which, line) in code {
            assert!(
                !line.contains(".about("),
                "{named}:{which} reads the record around the window: {line}"
            );
            if line.contains(".show(") {
                shown.push((named.clone(), line.trim().to_owned()));
            }
            if line.contains("self.read(recounting)") {
                read.push(named.clone());
            }
        }
    }
    assert_eq!(shown.len(), 1, "{shown:#?}");
    let (named, line) = shown.first().unwrap();
    assert_eq!(named, "record_window.rs");
    assert!(line.contains("recounting.show("), "{line}");
    assert_eq!(
        read, ["record_window.rs"; 3],
        "the plain road, the agent's road and reading again"
    );

    let window = the_code_of_file("record_window.rs");
    for door in ["pub fn opened_by_hand(", "pub fn asked_what_it_did("] {
        let at = window
            .iter()
            .position(|(_, line)| line.contains(door))
            .unwrap_or_else(|| panic!("{door} is gone"));
        let body: Vec<&str> = window
            .iter()
            .skip(at + 1)
            .map(|(_, line)| line.trim())
            .take_while(|line| *line != "}")
            .filter(|line| !line.is_empty())
            .collect();
        assert_eq!(
            body,
            ["self.read(recounting)"],
            "{door} does something of its own"
        );
    }

    let mut composited = 0;
    for (named, code) in the_record_files() {
        for (_, line) in code {
            if line.contains("impl Compositor for") {
                composited += 1;
                assert_eq!(named, "record_shown.rs");
            }
        }
    }
    assert_eq!(
        composited, 1,
        "the window is not handed its account through the port"
    );
}

/// **ADR 0009: the plain road passes through no agent.** Nothing in these
/// files names a turn, a model, a provider, the overlay, the daemon or its
/// protocol — so the window opens on a machine where the agent was declined,
/// never set up, or cannot be paid for.
#[test]
fn the_plain_road_passes_through_no_agent() {
    for (named, code) in the_record_files() {
        for (which, line) in code {
            for agent in [
                "alo_turn",
                "alo_overlay",
                "alo_agentd",
                "alo_answering",
                "alo_asking",
                "alo_models",
                "alo_protocol",
                "alo_approving",
                "alo_capability",
                "Turning",
                "Summoning",
                "Grants",
            ] {
                assert!(
                    !line.contains(agent),
                    "{named}:{which} puts the agent on the road to the record (`{agent}`): {line}"
                );
            }
        }
    }
}
