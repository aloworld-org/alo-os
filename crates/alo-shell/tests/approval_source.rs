//! The approval surface's promises that are about shape, read from the source
//! this crate ships.
//!
//! The plan's constraint for *the sentence a person approves* is three things
//! the surface must never have — **no approve all, no remember this, no timer**
//! — and three it must never be: a second place that words the change, a second
//! executor, or a road to a grant. A test that answered a question would only
//! show the roads it walked. These read the files instead, so a *remember this*
//! added tomorrow, a countdown, a sentence typed into a drawing file or a call
//! around `alo_approving::Approving` straight to the turn is a failing build
//! rather than approval turned into a formality.
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

/// The files that make up the approval surface: their names and their code.
fn the_approval_files() -> Vec<(String, Vec<(usize, String)>)> {
    let mut read = Vec::new();
    for entry in std::fs::read_dir(src()).unwrap() {
        let at = entry.unwrap().path();
        let named = at.file_name().unwrap().to_string_lossy().into_owned();
        let is_one = named.starts_with("approval_") || named == "nested_approval.rs";
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
            "approval_answers.rs",
            "approval_keys.rs",
            "approval_paint.rs",
            "approval_queue.rs",
            "approval_raster.rs",
            "approval_screen.rs",
            "approval_seat.rs",
            "approval_shown.rs",
            "nested_approval.rs",
        ],
        "an approval surface file was added or lost, and this test has to be told"
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

/// **The surface writes no words of its own.** The sentence is the proposal's,
/// the answers are `alo-approving`'s and every refusal is its refuser's; nothing
/// in these files declares a word, fills a gap, looks a key up, formats text or
/// holds a string literal a person could end up reading. The one literal
/// allowed is a lint's `reason`, which no person using the machine reads.
#[test]
fn the_approval_surface_writes_no_words_of_its_own() {
    for (named, code) in the_approval_files() {
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
        }
    }
}

/// **The surface draws the sentence it was handed, unedited.** Nothing in these
/// files shortens, trims, replaces or splits the text of a `Said`, and the only
/// way a question reaches them is `alo_approving::Compositor::ask` — no
/// proposal is read and worded here, and no `Asked` is made.
#[test]
fn the_approval_surface_draws_the_sentence_it_was_handed_unedited() {
    let mut composited = 0;
    for (named, code) in the_approval_files() {
        for (which, line) in code {
            for editing in [
                ".trim",
                ".replace",
                ".truncate",
                ".split",
                ".chars()",
                ".get(..",
                "Proposal {",
                "Wrap::None",
                "ellipsis",
                "Asked::of",
                ".sentence(strings",
                ".proposal",
                "Proposal::",
            ] {
                assert!(
                    !line.contains(editing),
                    "{named}:{which} could change what the turn wrote (`{editing}`): {line}"
                );
            }
            if line.contains("impl Compositor for") {
                composited += 1;
                assert_eq!(named, "approval_shown.rs");
            }
        }
    }
    assert_eq!(
        composited, 1,
        "the surface is not handed its question through the port"
    );
}

/// **One road to an answer, and it is `alo-approving`'s.** No file here names
/// the turn's own doors or the list of approvals beneath them, and the whole
/// surface calls `Approving::approve` and `Approving::decline` once each, in
/// the screen — so there is no second executor, and a second answer is refused
/// where the rule lives rather than here.
#[test]
fn every_answer_goes_through_alo_approving_once() {
    let mut approves = Vec::new();
    let mut declines = Vec::new();
    for (named, code) in the_approval_files() {
        for (which, line) in code {
            for around in [
                ".approving(",
                ".declining(",
                "Approvals",
                "Approved",
                ".redeem(",
            ] {
                assert!(
                    !line.contains(around),
                    "{named}:{which} goes around alo-approving (`{around}`): {line}"
                );
            }
            if line.contains(".approve(") {
                approves.push((named.clone(), line.trim().to_owned()));
            }
            if line.contains(".decline(") {
                declines.push((named.clone(), line.trim().to_owned()));
            }
        }
    }
    for calls in [&approves, &declines] {
        assert_eq!(calls.len(), 1, "{calls:#?}");
        let (named, line) = calls.first().unwrap();
        assert_eq!(named, "approval_screen.rs");
        assert!(line.contains("self.approving."), "{line}");
    }
}

/// **No approve all, no remember this, no timer — and no road to a grant.**
/// Nothing in these files is named for answering more than one question,
/// remembering an answer, reading the clock or counting down, and nothing
/// reaches a grant: a class of thing done without asking is a grant made in
/// `alo-picking`, and this surface is not a road to one.
#[test]
fn no_approve_all_no_remember_this_no_timer_and_no_road_to_a_grant() {
    for (named, code) in the_approval_files() {
        for (which, line) in code {
            let lowered = line.to_lowercase();
            for formality in [
                "approve_all",
                "all_approved",
                "every_question",
                "remember",
                "always",
                "trust",
                "session",
                "timer",
                "timeout",
                "countdown",
                "deadline",
                "instant",
                "now()",
                "sleep",
                "lapses_in",
                "auto",
                "default_answer",
                "preselect",
                "alo_picking",
                "grant(",
                "grant::",
                "reach::",
                "revoke",
            ] {
                assert!(
                    !lowered.contains(formality),
                    "{named}:{which} has `{formality}`, which turns approval into a formality: {line}"
                );
            }
        }
    }
}

/// **There are two answers, no third, and no way to give a reason.** The answer
/// and key types carry no data — nothing to type into — and each is held to its
/// list, so a *not now*, a *dismiss* or a field for why cannot be added without
/// this test being told.
#[test]
fn two_answers_no_third_and_nowhere_to_give_a_reason() {
    let variants_of = |file: &str, enumeration: &str| -> Vec<String> {
        let (_, code) = the_approval_files()
            .into_iter()
            .find(|(named, _)| named == file)
            .unwrap();
        let mut within = false;
        let mut variants = Vec::new();
        for (_, line) in code {
            let trimmed = line.trim();
            if trimmed.starts_with(&format!("pub enum {enumeration} ")) {
                within = true;
                continue;
            }
            if within && trimmed == "}" {
                break;
            }
            if within && !trimmed.is_empty() && !trimmed.starts_with("#[") {
                assert!(
                    !trimmed.contains('(') && !trimmed.contains('{'),
                    "{enumeration} carries something a person could type: {trimmed}"
                );
                variants.push(trimmed.trim_end_matches(',').to_owned());
            }
        }
        variants
    };
    assert_eq!(
        variants_of("approval_screen.rs", "ApprovalAnswer"),
        ["No", "Approve"]
    );
    // `Decline` is not a third answer: it is *no*, the answer this surface
    // already has, given by Escape because `alo_access::leaving` says
    // Escape declines here. Nothing was added to what a person can
    // answer — `ApprovalAnswer` above is still the two — and a key that
    // dismissed, deferred or asked for a reason would still fail here.
    assert_eq!(
        variants_of("approval_keys.rs", "ApprovalKey"),
        [
            "NextAnswer",
            "PreviousAnswer",
            "Choose",
            "Decline",
            "Nothing"
        ]
    );
    for (named, code) in the_approval_files() {
        for (which, line) in code {
            assert!(
                !names(&line, "char") && !names(&line, "Letter"),
                "{named}:{which} takes letters, which is somewhere to type a reason: {line}"
            );
        }
    }
}
