//! **A machine that converts everything it converts still says *nothing here
//! opens* a Pages document** — and never *this converts*, followed by a
//! failure.
//!
//! Task 6 of `docs/autonomy/v0-5-documents-and-paper-plan.md` recognised a Pages
//! document from its bytes, and ADR 0057 stands on that measurement: the format
//! is recognised because a real file said so. What recognition does not settle
//! is whether this machine will *convert* one, and the two are separate
//! promises: `alo-opening` naming a kind costs nothing if it is wrong about
//! what can be done with it, and a machine announcing a conversion it then
//! refuses is ADR 0039 §1's named failure.
//!
//! `alo-converting` now carries the fourth conversion whole — its kind, its
//! word on the socket, its export filter, the name its bytes take in the
//! scratch folder. What it does not carry is an inventory of one, because
//! ADR 0039 §4 makes the inventory the step before any copy and nobody has run
//! the engine over a Pages document on a machine this repository gates on.
//!
//! So the conversion is written and held back, and **this is the test that the
//! holding is real where a person meets it**: at the top of the machine, with
//! the service answering and every conversion it makes announced, a Pages
//! document is still a file this machine does not open.
//!
//! # Why the machine is assembled here rather than asked for
//!
//! `with_what_converts` needs a service that answers, which is a socket, a
//! scratch folder and a running engine — a harness
//! `the_walk_through_documents_and_paper.rs` builds and this does not need. What
//! it announces is `Conversion::EVERY`, and `alo_converting::machine`'s own test
//! holds it to that; this assembles the same announcement from the same list,
//! so a conversion added to `EVERY` reaches this test the day it is added.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::ffi::OsStr;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use alo_converting::Conversion;
use alo_opening::{Cannot, Decided, Kind, Outcome, ThisMachine, decide};

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

/// **The machine at its most capable does not open a Pages document.**
///
/// Recognised — the kind in the refusal is `PagesDocument` and not *something
/// this machine cannot name* — and not converted. Both halves matter: a machine
/// that could not name it would be a machine that had not read the file, and a
/// machine that offered to convert it would be promising a copy it cannot
/// inventory.
#[test]
fn a_pages_document_is_recognised_and_not_converted() {
    let decided = decide(
        &mut Cursor::new(the_document()),
        // No name is handed over, so no name can decide: this is the bytes.
        OsStr::new(""),
        &a_machine_that_converts(),
    )
    .expect("a document that is read");
    assert_eq!(
        decided,
        Decided::AsItIs(Outcome::CannotOpen(Cannot::NothingHereOpens(
            Kind::PagesDocument
        ))),
        "a machine with every conversion it makes announced said something else about a \
         Pages document"
    );
}

/// **Every conversion that is held back is a kind this machine neither opens
/// nor converts**, whatever else it announces.
///
/// The test above is this one for the document that exists; this is the same
/// promise for the list, so a second held-back conversion added later cannot
/// pass by nobody having a file of its kind to try.
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
