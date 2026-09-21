//! **A machine that converts what it converts says so about a Pages
//! document** — and about nothing it cannot inventory.
//!
//! Task 6 of `docs/autonomy/v0-5-documents-and-paper-plan.md` recognised a
//! Pages document from its bytes, and ADR 0057 stands on that measurement: the
//! format is recognised because a real file said so. Recognition and conversion
//! were two separate promises, and for three months only the first was kept:
//! `alo-converting` carried the conversion whole and out of
//! `Conversion::EVERY`, because ADR 0039 §4 makes the inventory of the original
//! the step before any copy and nobody had inventoried one.
//!
//! One was inventoried on 2026-09-21, on a machine with the engine, and this is
//! the test of what that changed **where a person meets it**: at the top of the
//! machine, with the service answering and every conversion it makes announced,
//! a Pages document is a file this machine opens by converting a copy of it.
//!
//! # And the promise that was underneath the holding, kept
//!
//! A machine whose service does *not* answer still says *nothing here opens
//! it*, which is the sentence that was true all along and stays true — and
//! anything still held back is still announced by nobody. Both are here,
//! because the reason the conversion waited was never that Pages documents are
//! unimportant: it was that a machine must not say *this converts* and then
//! fail.
//!
//! # Why the machine is assembled here rather than asked for
//!
//! `with_what_converts` needs a service that answers, which is a socket, a
//! scratch folder and a running engine — a harness
//! `the_walk_through_documents_and_paper.rs` builds and this does not need.
//! What it announces is `Conversion::EVERY`, and `alo_converting::machine`'s own
//! test holds it to that; this assembles the same announcement from the same
//! list, so a conversion added to `EVERY` reaches this test the day it is
//! added.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::ffi::OsStr;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use alo_converting::Conversion;
use alo_opening::{Cannot, Costs, Decided, Kind, Macros, Outcome, ThisMachine, decide};

/// The one real Pages document this repository holds. It belongs to
/// `alo-opening`, whose `tests/files/README.md` records where it came from.
const THE_DOCUMENT: &str = "crates/alo-opening/tests/files/document.pages";

/// This repository.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("this crate is two directories below the repository")
}

/// The Pages document, as its bytes.
fn the_document() -> Vec<u8> {
    let at = the_repository().join(THE_DOCUMENT);
    std::fs::read(&at).unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// A machine with the converting service answering: it opens a PDF, and it
/// converts every conversion this machine makes into one. The same
/// announcement `alo_converting::with_what_converts` makes, from the same list.
fn a_machine_that_converts() -> ThisMachine {
    let mut machine = ThisMachine::with_nothing()
        .opens(Kind::Pdf)
        .expect("a machine that opens the kind it converts into");
    for conversion in Conversion::EVERY {
        machine = machine
            .converts(conversion.from(), conversion.into())
            .expect("a conversion this machine makes");
    }
    machine
}

/// What this machine says about the Pages document, from its bytes alone.
fn decided_about_the_document(machine: &ThisMachine) -> Decided {
    decide(
        &mut Cursor::new(the_document()),
        // No name is handed over, so no name can decide: this is the bytes.
        OsStr::new(""),
        machine,
    )
    .expect("a document that is read")
}

/// **A Pages document is a file this machine opens, by converting a copy into
/// a PDF.**
///
/// Recognised — the kind the outcome is from is `PagesDocument` and not
/// *something this machine cannot name* — and converted, which is the half that
/// was missing. What the copy costs is not decided here: that is measured by
/// converting the real document through the real engine, in
/// `converting_a_real_document.rs`.
#[test]
fn a_pages_document_is_recognised_and_converted() {
    let decided = decided_about_the_document(&a_machine_that_converts());
    let Decided::AsItIs(Outcome::Converts { from, into, costs }) = decided else {
        panic!("a machine with every conversion it makes announced said {decided:?}");
    };
    assert_eq!(from, Kind::PagesDocument);
    assert_eq!(into, Kind::Pdf);
    assert_eq!(costs, Costs::of(Macros::NoneSeen));
}

/// **Without the service it is still *nothing here opens it***, which is the
/// sentence that was true while the conversion was held back and is true now
/// on a machine where nothing converts.
///
/// ADR 0039 §1: a machine without the service says what it can do, not what
/// some other machine could.
#[test]
fn a_machine_that_converts_nothing_still_says_nothing_here_opens_it() {
    let decided = decided_about_the_document(&ThisMachine::with_nothing());
    assert_eq!(
        decided,
        Decided::AsItIs(Outcome::CannotOpen(Cannot::NothingHereOpens(
            Kind::PagesDocument
        )))
    );
}

/// **Every conversion that is held back is a kind this machine neither opens
/// nor converts**, whatever else it announces.
///
/// `Conversion::HELD_BACK` is empty, so this walks nothing today. It is kept
/// rather than deleted with the conversion that left it: the next format whose
/// original cannot be inventoried needs this promise already written, and a
/// promise rewritten from a report comes back weaker than one that was never
/// removed.
///
/// It asks by trying to tell the machine, because that is the question
/// `ThisMachine` answers in public: telling it a conversion it already makes is
/// [`alo_opening::NotAnAbility::AlreadyConverted`], and telling it one whose
/// kind it opens is `FromAKindItAlreadyOpens`. Being allowed to say it is
/// therefore proof that nobody has.
#[test]
fn no_held_back_conversion_is_announced() {
    for held in Conversion::HELD_BACK {
        assert!(
            a_machine_that_converts()
                .converts(held.from(), held.into())
                .is_ok(),
            "{held:?} is already announced by a machine that cannot inventory one"
        );
    }
}
