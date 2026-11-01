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
            "record_clock.rs",
            "record_entry_drawn.rs",
            "record_frame_drawn.rs",
            "record_header.rs",
            "record_keys.rs",
            "record_kinds.rs",
            "record_lines.rs",
            "record_measures.rs",
            "record_paint.rs",
            "record_palette.rs",
            "record_put_back_drawn.rs",
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
///
/// **Since 2026-11-01 the window has a few words of its own**, which the
/// owner approved with its design (`560:142`): its title, the sentence under
/// it, the footer, *Already sent*, *Refused*, and today's three counts. They
/// are declared and looked up in `the_records_words.rs` — outside this list,
/// beside the shell's other surfaces' words — and handed in, so this test
/// still holds of every file here: none of them says anything. None of those
/// words describes an entry, and the counts are numbers of entries that the
/// list under them can be checked against.
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
                // **Today's counts count today, and narrow nothing drawn.**
                // `record_kinds.rs` reads which of the account's entries are
                // today's to number them beside *Today* (`560:195`, approved
                // on 2026-11-01); every entry is still drawn, which
                // `record_raster_tests` checks by walking the whole account,
                // and the numbers are checked against the list. So *today* is
                // allowed in this one file, and the filter that counts.
                let counts_today =
                    named == "record_kinds.rs" && (narrowing == "today" || narrowing == ".filter(");
                // And the header and the list say *today* where they hold or
                // draw those counts, which is naming them, not narrowing.
                let names_the_counts = narrowing == "today"
                    && ["record_header.rs", "record_raster.rs"].contains(&named.as_str());
                if counts_today || names_the_counts {
                    continue;
                }
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
            ] {
                assert!(
                    !line.contains(agent),
                    "{named}:{which} puts the agent on the road to the record (`{agent}`): {line}"
                );
            }
            // **Types are read as whole names**, as the writers are above:
            // `Grants` is the agent's grants, and `Outcome::TheGrantsSaidNo`
            // is one of the record's own outcomes, which the window sorts into
            // the design's kinds (`record_kinds.rs`) — reading the record, not
            // reaching the agent. A search on the letters could not tell them
            // apart.
            for agent in ["Turning", "Summoning", "Grants"] {
                assert!(
                    !names(&line, agent),
                    "{named}:{which} puts the agent on the road to the record (`{agent}`): {line}"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Whether anything a machine runs walks each of the two roads
// ---------------------------------------------------------------------------

/// Every production `.rs` of this crate, as (name, code), with unit tests,
/// comments and string literals already taken out by [`the_code_of`].
///
/// Wider than [`the_record_files`] on purpose: a caller of the window can be
/// any file in the crate, and reading only the record's own files would answer
/// *nothing calls this* about a road walked from the dock.
fn every_production_file() -> Vec<(String, Vec<(usize, String)>)> {
    let mut read = Vec::new();
    for entry in std::fs::read_dir(src()).unwrap() {
        let at = entry.unwrap().path();
        if at.extension().is_none_or(|it| it != "rs") {
            continue;
        }
        let named = at.file_name().unwrap().to_string_lossy().into_owned();
        if named.ends_with("_tests.rs") || named.ends_with("_testing.rs") {
            continue;
        }
        read.push((named, the_code_of(&std::fs::read_to_string(&at).unwrap())));
    }
    assert!(
        read.len() > 50,
        "only {} production files were read, which is not this crate",
        read.len()
    );
    read
}

/// Calls to a door, outside the file that declares it.
///
/// **`opened_by_hand` is a method on two windows** — `RecordWindow` takes a
/// `Recounting`, `SettingsWindow` takes places and a moment — and a search on
/// the bare name cannot tell them apart. So a call counts as the record
/// window's only when `recounting` is in it, and this is the weakness of the
/// instrument rather than of the roads: a caller that bound the recounting to a
/// differently-named variable first would not be seen. `asked_what_it_did` is
/// `RecordWindow`'s alone and needs no such discriminator.
fn calls_to(door: &str, discriminator: Option<&str>) -> Vec<String> {
    let mut found = Vec::new();
    for (named, code) in every_production_file() {
        if named == "record_window.rs" {
            continue;
        }
        for (which, line) in code {
            if !line.contains(&format!(".{door}(")) {
                continue;
            }
            if discriminator.is_some_and(|it| !line.contains(it)) {
                continue;
            }
            found.push(format!("{named}:{which}"));
        }
    }
    found
}

/// A door nothing a machine runs calls, and why.
///
/// **Every line here is a measurement with a date, not an excuse.** A road
/// bound to a surface nothing reaches teaches a person a way in and then takes
/// it away, which is what `the_road_every_action_takes.rs` refuses for the
/// keyboard's actions; this is the same rule for the record window's two doors.
const CALLED_BY_NOTHING: [(&str, &str); 1] = [(
    "asked_what_it_did",
    "The agent's road, and nothing walks it. Measured 2026-10-17: zero \
         calls anywhere outside tests. Nothing is wrong with the door — what is \
         missing is that an agent's answer cannot reach this window. \
         `what_a_person_asked_goes_to_the_agent.rs` addresses an answer to a task \
         window, by `WhichTask`, and there is no *who asked* that can also be a \
         record or a terminal.",
)];

/// **Each of the window's two doors is called by something a machine runs, or
/// is named above as one that is not.**
#[test]
fn each_road_to_the_record_is_walked_or_named_as_unwalked() {
    for (door, discriminator) in [
        ("opened_by_hand", Some("recounting")),
        ("asked_what_it_did", None),
    ] {
        let walked = calls_to(door, discriminator);
        let named = CALLED_BY_NOTHING.iter().any(|(which, _)| *which == door);
        assert!(
            !walked.is_empty() || named,
            "{door} is called by nothing a machine runs and is not named in \
             CALLED_BY_NOTHING. Either wire it, or write down why it is not wired — a door \
             nothing reaches is a promise of a road."
        );
    }
}

/// **Nothing named as unwalked has quietly got a caller.**
///
/// The other half, and the reason the list is a constant rather than a comment:
/// a reason that has stopped being true is one somebody reads as current.
#[test]
fn nothing_named_as_unwalked_is_walked_after_all() {
    for (door, _) in CALLED_BY_NOTHING {
        let discriminator = if door == "opened_by_hand" {
            Some("recounting")
        } else {
            None
        };
        let walked = calls_to(door, discriminator);
        assert!(
            walked.is_empty(),
            "{door} is named as called by nothing and is called at {walked:?} — take it out of \
             CALLED_BY_NOTHING"
        );
    }
}

/// **The search can see a production call**, which is the control.
///
/// Without it every assertion above would pass on a walker that found nothing
/// anywhere, and *nothing calls this* would be indistinguishable from *this
/// test cannot see calls* — the shape where an empty result and a negative
/// result look the same.
///
/// The call it is shown is the **settings** window's `opened_by_hand`, because
/// that is the one production call of either name in the crate. So this control
/// is also the measurement behind both entries above: the name is called, and
/// never on the record.
#[test]
fn the_search_can_see_a_production_call() {
    let any = calls_to("opened_by_hand", None);
    assert!(
        !any.is_empty(),
        "the walker found no call of `opened_by_hand` anywhere in production, so it cannot see \
         calls at all and every assertion in this file about there being none is vacuous"
    );
    assert!(
        any.iter()
            .any(|at| at.starts_with("a_press_on_the_dock.rs:")
                || at.starts_with("settings_command.rs:")),
        "the production calls of this name should be the settings window's, and they are \
         {any:?}"
    );
}

/// **And a call inside a `#[cfg(test)]` block is not a production call.**
///
/// The control for the control, and it is here because **I got this wrong
/// today.** `nested_record.rs` calls `opened_by_hand` at line 196 and its test
/// module begins at 162, and I read that call as the production road and wrote
/// it into a report. [`the_code_of`] was right all along — it cuts at the test
/// module — and what was wrong was my reasoning about a per-file search.
///
/// So this asserts the cut really happens, on the file that fooled me.
#[test]
fn a_call_below_a_test_module_is_not_counted() {
    let code = the_code_of(&std::fs::read_to_string(src().join("nested_record.rs")).unwrap());
    assert!(
        !code
            .iter()
            .any(|(_, line)| line.contains(".opened_by_hand(")),
        "nested_record.rs's call to opened_by_hand survived the cut at its test module, so \
         every count in this file includes tests"
    );
    // And the raw file does hold it, so the assertion above is about a cut and
    // not about a file that never had one.
    assert!(
        std::fs::read_to_string(src().join("nested_record.rs"))
            .unwrap()
            .contains(".opened_by_hand("),
        "nested_record.rs no longer calls opened_by_hand at all, so this test is about nothing"
    );
}

/// **And the search does not count the settings window's door of the same
/// name**, which is the second control and the one the discriminator exists for.
#[test]
fn the_settings_windows_door_of_the_same_name_is_not_counted() {
    let without = calls_to("opened_by_hand", None);
    let with = calls_to("opened_by_hand", Some("recounting"));
    assert!(
        without.len() > with.len(),
        "the undiscriminated search found {} and the discriminated one {}, so either the \
         settings window no longer has a door of this name or the discriminator has stopped \
         discriminating — and this test is then about nothing",
        without.len(),
        with.len()
    );
}
