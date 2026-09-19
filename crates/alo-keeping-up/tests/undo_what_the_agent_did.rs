//! ★ *Undo what the agent did* — the acceptance of task 4 of
//! `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md`, one criterion at a
//! time, and of the points
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md) says
//! hold whichever road is taken.
//!
//! | Criterion | Test |
//! |---|---|
//! | for a verb the record says ran, whether it can be undone | [`every_verb_this_machine_ships_is_answered_by_the_table`] |
//! | and it says plainly when it cannot | [`every_reason_it_cannot_is_a_sentence_this_machine_collects`] |
//! | a verb nobody classified is never undoable | [`a_verb_added_after_this_was_written_is_not_undoable`] |
//! | undone through the mechanism that made it undoable, never a second implementation | [`nothing_here_puts_a_file_back_or_invents_an_inverse`] |
//! | an undo never overwrites a later change | [`an_undo_is_refused_before_it_is_offered_when_anything_moved_since`] |
//! | undoing requires the same person's approval the doing did | [`the_only_way_to_an_undo_is_a_sentence_a_person_approves`] |
//! | no agent verb undoes, and none proposes one | [`no_verb_this_machine_ships_undoes_anything`] |
//! | what an undo may keep is bounded and forgettable | [`what_can_be_put_back_is_bounded_and_forgotten_in_one_act`] |
//!
//! # Why the verbs come from `alo-declared` and not from a list here
//!
//! The table in ADR 0045 point 1 is a promise about **every verb this machine
//! ships**, including the ones it gains after this file was written. A list
//! written out here would be a second copy of `alo-declared`'s, and the day
//! they disagreed this test would pass by agreeing with itself. So it asks the
//! one registry, and a verb added anywhere in the workspace arrives here
//! without anybody remembering to add it.
//!
//! # What this file cannot reach, and where that is tested instead
//!
//! The record is not `alo-keeping-up`'s: this crate names no
//! `alo_record::Happened` and takes what the record says as a
//! [`alo_keeping_up::WhatWasDone`] already read. The seam between the two, the
//! entry an undo leaves behind and the account a person reads it back in are
//! `alo-updating`'s and `alo-record`'s, and
//! `crates/alo-updating/tests/putting_back_what_an_agent_did.rs` is the other
//! half of this acceptance.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::BTreeSet;

use alo_capability::{Effect, Verb};
use alo_declared::every_verb_this_machine_ships;
use alo_keeping_up::{
    AnUndo, Bracket, Change, HowFarBack, NotUndoable, WhatWasDone, WhatWasKept, WhenItWasDone,
};
use alo_saying::everything_this_machine_can_say;
use alo_strings::Strings;

/// The machine's whole vocabulary, in English — everybody's words and not just
/// this crate's, because a sentence that is only in its own crate's list is a
/// sentence nobody on a screen can read.
fn the_machine() -> Strings {
    Strings::of(everything_this_machine_can_say().unwrap())
}

/// Every verb alo OS ships, by name and by whether it changes anything.
fn every_verb() -> Vec<(String, bool)> {
    let verbs = every_verb_this_machine_ships().expect("the verbs this machine ships");
    let mut every: Vec<(String, bool)> = verbs
        .all()
        .map(|verb: &Verb| (verb.name().to_owned(), verb.effect() == Effect::Change))
        .collect();
    every.sort();
    assert!(
        every.len() > 10,
        "only {} verbs were declared, so this test is asking almost nothing",
        every.len()
    );
    every
}

/// A turn that changed one file and left it exactly as it left it.
fn kept(changed: &[&str], moved_since: &[&str]) -> WhatWasKept {
    WhatWasKept::EitherSideOfTheTurn(Bracket::comparing(
        changed.iter().map(|name| (*name).to_owned()).collect(),
        moved_since.iter().map(|name| (*name).to_owned()).collect(),
    ))
}

/// The moment the person is shown, as somebody has already worded it.
fn on_tuesday() -> WhenItWasDone {
    WhenItWasDone::worded("Tuesday at 14:05").unwrap()
}

/// **For every verb the record can say ran, the table answers** — and the
/// three ADR 0045 names are the three it says yes to.
///
/// The first acceptance of the task, asked of every verb this machine ships
/// rather than of the handful somebody thought of: a machine that answered
/// *nothing to undo* for a verb that had moved a file would be lying about the
/// person's own files.
#[test]
fn every_verb_this_machine_ships_is_answered_by_the_table() {
    let mut undoable: BTreeSet<String> = BTreeSet::new();
    for (verb, changed) in every_verb() {
        let what = WhatWasDone::of_a_verb(&verb, changed);
        match what.could_be_put_back() {
            Ok(change) => {
                assert_eq!(change.verb(), verb, "{verb}");
                undoable.insert(verb);
            }
            Err(why) => {
                // Every refusal is one of the nine that are about what was
                // done. The five about this machine are `AnUndo::offered`'s and
                // cannot be reached from a verb's name alone.
                assert!(
                    !matches!(
                        why,
                        NotUndoable::NothingKeepsWhatWasThere
                            | NotUndoable::TheTurnWasNotKept
                            | NotUndoable::NoLongerKept
                            | NotUndoable::ChangedSinceItWasDone
                            | NotUndoable::NothingLeftToPutBack
                    ),
                    "{verb} was refused for what this machine kept, which its name cannot say"
                );
                // And a change verb is never refused as though it had only
                // looked, which is the one wrong answer that would read as
                // reassuring.
                if changed {
                    assert_ne!(why, NotUndoable::ItOnlyLooked, "{verb}");
                }
            }
        }
    }
    assert_eq!(
        undoable,
        Change::EVERY
            .into_iter()
            .map(|change| change.verb().to_owned())
            .collect::<BTreeSet<String>>(),
        "the verbs there is a road back for are not the three ADR 0045 names"
    );
}

/// **A verb added after this was written answers *never*.**
///
/// The rule that stops the table quietly starting to lie: the default is no
/// road back, so a verb becomes undoable only in the change that says it is,
/// never by arriving.
#[test]
fn a_verb_added_after_this_was_written_is_not_undoable() {
    for verb in [
        "a_change_verb_from_next_year",
        "delete_file",
        "rewrite_document",
        "empty_the_folder",
    ] {
        assert_eq!(
            WhatWasDone::of_a_verb(verb, true).could_be_put_back(),
            Err(NotUndoable::NotOneOfTheChangesPutBack),
            "{verb}"
        );
    }
    for verb in ["a_read_verb_from_next_year", "count_the_files"] {
        assert_eq!(
            WhatWasDone::of_a_verb(verb, false).could_be_put_back(),
            Err(NotUndoable::ItOnlyLooked),
            "{verb}"
        );
    }
}

/// **Every reason it cannot be undone is a sentence, in the machine's one
/// vocabulary, and no two of them read alike.**
///
/// A refusal a person cannot read the *why* of is a system they stop trusting,
/// and two refusals wearing one sentence is the same failure wearing a table.
/// Asked of the whole machine's words rather than this crate's own list,
/// because a sentence `alo-saying` does not collect is a key in guillemets on
/// the screen.
#[test]
fn every_reason_it_cannot_is_a_sentence_this_machine_collects() {
    let strings = the_machine();
    let mut said: Vec<String> = Vec::new();
    for why in NotUndoable::EVERY {
        let sentence = why.said(&strings);
        assert!(!sentence.is_a_bug(), "{why:?}: {sentence}");
        said.push(sentence.text().to_owned());
    }
    assert_eq!(said.len(), 14);
    let apart: BTreeSet<&String> = said.iter().collect();
    assert_eq!(apart.len(), said.len(), "two reasons read the same");

    // And the four that are gone beyond recall say **why** they are, rather
    // than refusing without one — the half of ★ that matters most.
    for (what, has_to_say) in [
        (WhatWasDone::ItLeft, "left your machine"),
        (WhatWasDone::AModelWasTold, "untell"),
        (WhatWasDone::ItWasPrinted, "printed on paper"),
        (
            WhatWasDone::AnApplicationWasInstalled,
            "Removing an application is yours",
        ),
    ] {
        let why = what.could_be_put_back().unwrap_err();
        let sentence = why.said(&strings);
        assert!(sentence.text().contains(has_to_say), "{what:?}: {sentence}");
    }
}

/// **Nothing here puts a file back, and no inverse is invented.**
///
/// The plan's constraint and ADR 0045's point: what can be undone is undone
/// through the mechanism that made it undoable, never by a second
/// implementation guessing at inverses. So this crate, which decides, must not
/// contain one — no rename, no copy, no removal, and nothing that maps a verb
/// onto the verb that would reverse it.
#[test]
fn nothing_here_puts_a_file_back_or_invents_an_inverse() {
    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut read = 0;
    for entry in std::fs::read_dir(&source).unwrap() {
        let path = entry.unwrap().path();
        let text = std::fs::read_to_string(&path).unwrap();
        for forbidden in [
            "fs::rename",
            "fs::copy",
            "fs::remove",
            "fs::write",
            "fs::create",
            "reflink",
            "Command",
            "the_inverse",
            "undo_by",
        ] {
            assert!(
                !text.contains(forbidden),
                "{} names {forbidden}",
                path.display()
            );
        }
        read += 1;
    }
    assert!(read >= 15, "only {read} source files were read");
}

/// **An undo never overwrites a later change** (ADR 0045 point 3), and it is
/// refused **before** it is offered rather than failing halfway.
///
/// One name no longer as the turn left it refuses the whole undo. Putting some
/// of it back would leave the person's files between two states nobody chose,
/// and taking the rest would take away their own newer work.
#[test]
fn an_undo_is_refused_before_it_is_offered_when_anything_moved_since() {
    let two = ["March.pdf", "April.pdf"];
    let still_as_it_was =
        AnUndo::offered(WhatWasDone::ChangedFiles(Change::Renamed), &kept(&two, &[])).unwrap();
    assert_eq!(still_as_it_was.puts_back(), two);

    let one = ["April.pdf"];
    for moved_since in [one.as_slice(), two.as_slice()] {
        let refused = AnUndo::offered(
            WhatWasDone::ChangedFiles(Change::Renamed),
            &kept(&two, moved_since),
        )
        .unwrap_err();
        assert_eq!(refused, NotUndoable::ChangedSinceItWasDone);
        assert!(!refused.said(&the_machine()).is_a_bug());
    }
}

/// **On the machine alo OS installs today, nothing kept what the files were,
/// and the person is told exactly that** — ADR 0045's *not yet on this
/// machine*, which is true rather than a stub, and is not the same refusal as
/// a machine that kept it and let it go.
#[test]
fn a_machine_that_kept_nothing_says_so_and_says_it_apart_from_having_let_go() {
    let strings = the_machine();
    for change in Change::EVERY {
        let what = WhatWasDone::ChangedFiles(change);
        for (what_was_kept, expected) in [
            (
                WhatWasKept::NothingOnThisMachine,
                NotUndoable::NothingKeepsWhatWasThere,
            ),
            (WhatWasKept::NotThisTurn, NotUndoable::TheTurnWasNotKept),
            (WhatWasKept::LetGo, NotUndoable::NoLongerKept),
        ] {
            let refused = AnUndo::offered(what, &what_was_kept).unwrap_err();
            assert_eq!(refused, expected, "{change:?} {what_was_kept:?}");
            assert!(!refused.said(&strings).is_a_bug());
        }
    }
}

/// **The only way to an undo is a sentence a person approves**, naming what
/// comes back and when it was changed.
///
/// An undo *is* a change, so it waits for one approval of one sentence exactly
/// as the doing did (ADR 0001 §5, ADR 0045 point 2). What is approved says both
/// halves: a person cannot check an offer that names neither the files nor the
/// moment.
#[test]
fn the_only_way_to_an_undo_is_a_sentence_a_person_approves() {
    let undo = AnUndo::offered(
        WhatWasDone::ChangedFiles(Change::Moved),
        &kept(&["/home/ana/Invoices/March.pdf"], &[]),
    )
    .unwrap();
    let said = undo.said(&on_tuesday(), &the_machine());
    assert!(!said.is_a_bug(), "{said}");
    assert!(
        said.text().contains("/home/ana/Invoices/March.pdf"),
        "{said}"
    );
    assert!(said.text().contains("Tuesday at 14:05"), "{said}");

    // And an offer cannot be worded with a blank where the moment belongs.
    assert_eq!(WhenItWasDone::worded(""), None);
    assert_eq!(WhenItWasDone::worded("  \t "), None);
}

/// **No verb this machine ships undoes anything, and none proposes an undo.**
///
/// ADR 0045 point 2, held against the one registry rather than against a
/// memory of it: an agent choosing which of a person's approvals to reverse is
/// the thing the decision forbids by name, so the capability a person gets is
/// one no agent has. Adding one is additive and needs its own decision — and
/// this test is what makes that a decision rather than an afternoon.
#[test]
fn no_verb_this_machine_ships_undoes_anything() {
    for (verb, _) in every_verb() {
        for forbidden in ["undo", "put_back", "revert", "restore", "roll_back"] {
            assert!(
                !verb.contains(forbidden),
                "`{verb}` reads as a verb that undoes, which ADR 0045 point 2 forbids"
            );
        }
    }
    // And the crates that decide about undoing declare no verbs at all, so
    // there is nowhere for one to have been added quietly.
    assert!(!alo_declared::WHO_DECLARES_THEM.contains(&"alo-keeping-up"));
    assert!(!alo_declared::WHO_DECLARES_THEM.contains(&"alo-updating"));
}

/// **What an undo may keep is bounded, and forgetting it is one act** — the
/// owner's first and fifth terms on ADR 0045.
///
/// Seven days or fifty changing turns, whichever ends first, so a disk does not
/// fill quietly; and one sentence that lets go of all of it, which says both
/// that nothing can be put back afterwards and that the space comes back.
#[test]
fn what_can_be_put_back_is_bounded_and_forgotten_in_one_act() {
    let window = HowFarBack::AS_SHIPPED;
    assert_eq!((window.days(), window.turns()), (7, 50));
    assert!(window.still_reaches(6, 49));
    assert!(!window.still_reaches(7, 0));
    assert!(!window.still_reaches(0, 50));
    // A window that reaches nothing is undo switched off by arithmetic, and is
    // refused: the honest way to hold nothing is the one act below.
    assert!(HowFarBack::of(0, 50).is_err());
    assert!(HowFarBack::of(7, 0).is_err());

    let strings = the_machine();
    let forgetting = WhatWasKept::forgetting(&strings);
    assert!(!forgetting.is_a_bug(), "{forgetting}");
    assert!(
        forgetting.text().contains("none of them can be put back"),
        "{forgetting}"
    );
    assert!(forgetting.text().contains("space"), "{forgetting}");
}
