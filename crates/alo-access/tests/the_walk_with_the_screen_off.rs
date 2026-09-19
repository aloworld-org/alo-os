//! The acceptance of task 7: **one walk, with the screen off, from turning the
//! reader on to reading the record — as the exact sequence a person meets.**
//!
//! Tasks 1 to 6 each end in something a person hears or reads: a setting, a
//! control's name, a language, an approval, a line of the record. Each crate's
//! own tests hold its own. What none of them can show is **the sequence** —
//! whether a person with the screen off is carried from one to the next, or
//! handed six crates' worth of correct sentences that do not join up.
//!
//! So this walks one person's sign-in without a screen:
//!
//! 1. the screen reader is turned on, before any account is chosen;
//! 2. they sign in with the keyboard alone, hearing each stop;
//! 3. they open the agent — without knowing the chord;
//! 4. they ask a question in Greek;
//! 5. they answer an approval, and hear that nothing was chosen for them;
//! 6. they read the record.
//!
//! # The table is the report's, and this test reads it
//!
//! The sequence is recorded in [`THE_REPORT`] under [`THE_WALK`], and this test
//! parses that table rather than a copy of it: a sentence that changes without
//! the table changing fails here. A later change that moves one publishes the
//! table again in a follow-up and points [`THE_REPORT`] at it, because a
//! published report is never rewritten.
//!
//! # Spoken and shown are one column, deliberately
//!
//! Every name in `crate::tree` is what a reader is told **and** what the shell
//! draws — one description, used by both, which is the argument that file makes
//! and the reason a blind person and an agent read the same machine. A table
//! with two columns would be two descriptions, and the day they differed nobody
//! would know which one was the machine.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};

use alo_access::setting::Setting;
use alo_access::tree::Surface;
use alo_access::{Leaving, focus_order, leaving};
use alo_strings::{Filling, Strings, Word};

/// The report this walk is recorded in.
const THE_REPORT: &str = "docs/autonomy/updates/the-walk-with-the-screen-off.md";

/// The heading its table is under.
const THE_WALK: &str = "## The walk, sentence by sentence";

/// The question, in Greek: *how much room have I left?*
const ASKED_IN_GREEK: &str = "Πόσο χώρο έχω;";

/// Everything this machine can say, in the language this test reads.
fn strings() -> Strings {
    Strings::of(
        alo_saying::everything_this_machine_can_say().expect("this machine's own vocabulary"),
    )
}

/// One sentence, as a person reads or hears it.
fn said(strings: &Strings, word: Word) -> String {
    strings
        .say(&word.key(), &Filling::nothing())
        .text()
        .to_owned()
}

/// **The walk**: each moment, and what a person meets at it.
fn the_walk() -> Vec<(String, String)> {
    let strings = strings();
    let mut walk: Vec<(String, String)> = Vec::new();

    // 1. The reader is turned on before there is an account — task 1's whole
    // point, and the only order that works for somebody who cannot see the
    // screen they would otherwise have to sign into first.
    walk.push((
        "The screen reader is turned on, before any account is chosen".to_owned(),
        said(&strings, Setting::ScreenReader.word()),
    ));

    // 2. Signing in, by keyboard alone: every stop, in the order the keyboard
    // reaches them, which is the order they are read.
    for stop in focus_order(Surface::SignIn) {
        walk.push((
            "Tab, and the keyboard is here".to_owned(),
            said(&strings, stop.name),
        ));
    }

    // 3. The agent, reached without knowing the chord.
    let agent = focus_order(Surface::Desktop)
        .into_iter()
        .find(|control| control.name.key() == alo_access::words::ASK_THE_AGENT.key())
        .expect("the desktop has a stop that opens the agent");
    walk.push((
        "Tab across the desktop, and the agent is one of the stops".to_owned(),
        said(&strings, agent.name),
    ));

    // 4. A question in Greek. What a person meets is an answer in their own
    // language; what this machine decides, and can be held to, is which
    // language that is.
    let language = alo_instructing::the_language_of(ASKED_IN_GREEK)
        .expect("a question written in Greek is written in Greek");
    walk.push((
        format!("The question is asked in Greek — {ASKED_IN_GREEK}"),
        format!(
            "{} ({})",
            language
                .in_its_own_language()
                .expect("a language alo OS speaks calls itself something"),
            language.tag()
        ),
    ));

    // 5. The approval: the sentence, then the two answers, with nothing chosen.
    for control in Surface::Approval.read_aloud() {
        walk.push((
            "The approval is read, and nothing is chosen for them".to_owned(),
            said(&strings, control.name),
        ));
    }
    walk.push((
        "Escape, on the approval".to_owned(),
        match leaving(Surface::Approval) {
            Leaving::Declines => "it answers no".to_owned(),
            other => panic!("Escape on an approval does something else: {other:?}"),
        },
    ));

    // 6. The record.
    for control in Surface::Record.read_aloud() {
        walk.push((
            "The record is read".to_owned(),
            said(&strings, control.name),
        ));
    }

    walk
}

/// This repository.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("this crate is two directories below the repository")
}

/// The rows of the table under [`THE_WALK`] in [`THE_REPORT`].
fn the_table() -> Vec<(String, String)> {
    let report = fs::read_to_string(the_repository().join(THE_REPORT))
        .unwrap_or_else(|why| panic!("{THE_REPORT} could not be read: {why}"));
    let rows: Vec<(String, String)> = report
        .lines()
        .skip_while(|line| line.trim() != THE_WALK)
        .skip(1)
        .take_while(|line| !line.starts_with("## "))
        .skip_while(|line| !line.starts_with('|'))
        .take_while(|line| line.starts_with('|'))
        .skip(2)
        .map(|row| {
            let cells: Vec<&str> = row
                .trim()
                .trim_matches('|')
                .split(" | ")
                .map(str::trim)
                .collect();
            let [_, moment, met] = cells.as_slice() else {
                panic!("a row is a step, a moment and what a person meets: {row}");
            };
            ((*moment).to_owned(), (*met).to_owned())
        })
        .collect();
    assert!(
        !rows.is_empty(),
        "{THE_REPORT} has no table under {THE_WALK:?}"
    );
    rows
}

/// The walk, written as the table's rows.
fn as_a_table(walk: &[(String, String)]) -> String {
    walk.iter()
        .enumerate()
        .map(|(step, (moment, met))| format!("| {} | {moment} | {met} |", step + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

/// **The walk produces exactly the table in the report**, in order.
#[test]
fn a_sign_in_with_the_screen_off_reads_as_the_table_in_the_report() {
    let walk = the_walk();
    assert!(
        walk == the_table(),
        "the walk and the table in {THE_REPORT} differ. The walk reads:\n\n{}\n",
        as_a_table(&walk)
    );
}

/// **Every sentence these crates can say is in the machine's vocabulary, with a
/// note for whoever translates it.**
#[test]
fn every_sentence_about_access_is_collected_and_carries_a_note() {
    let machine = alo_saying::everything_this_machine_can_say().expect("this machine's words");
    for word in alo_access::words::EVERY_WORD {
        assert!(
            machine.phrase(&word.key()).is_some(),
            "{} is said by this crate and this machine does not know it",
            word.named()
        );
        assert!(
            word.note().is_some(),
            "{} has no note for whoever translates it",
            word.named()
        );
        assert!(!word.says().trim().is_empty(), "{}", word.named());
    }
}

/// **No sentence names what is underneath.**
///
/// The plan's own list. A person turning on the screen reader is turning on the
/// screen reader; which one this machine rents, and how the tree reaches it, is
/// ours to change without changing a word anybody reads.
#[test]
fn no_sentence_names_the_reader_the_tree_or_the_data() {
    const NEVER: [&str; 8] = [
        "orca",
        "at-spi",
        "atspi",
        "espeak",
        "speech-dispatcher",
        "cldr",
        "unicode",
        "screen reader engine",
    ];
    for word in alo_access::words::EVERY_WORD {
        let said = word.says().to_lowercase();
        for never in NEVER {
            assert!(!said.contains(never), "{} names {never}", word.named());
        }
    }
}
