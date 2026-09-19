//! A file that begins like a film and is not one is not called a film.
//!
//! Found by task 6 of `docs/autonomy/v0-5-documents-and-paper-plan.md`, which
//! says the three files its promise names — a `.pages`, a `.heic` and a `.dwg`
//! — are met today as *this machine does not recognise what this file is*. For
//! two of them that was true. For the photograph it was not: `ftyp` at offset
//! four was taken for the container MP4 names, so a photograph from a telephone
//! was reported as **an MP4 video**, and a person who was told that would go
//! looking for something to play it with.
//!
//! The header is a family, not a format. The brands after it say which member
//! of the family a file is, and `src/iso_media.rs` reads them: a file whose
//! brands name none of the ones this machine knows is *not recognised*, which
//! is honest about this machine and sends nobody anywhere.
//!
//! # What this file is not
//!
//! **It is not recognising a photograph.** Naming a `.heic` as what it is —
//! and the `.pages` and `.dwg` beside it — is task 6, and waits on
//! [ADR 0056](../../../docs/decisions/0056-a-format-is-recognised-on-the-evidence-of-a-real-file.md)
//! and on a real file of each to measure the rule against. Nothing here says
//! what a photograph is; it says only what this machine will no longer claim
//! about one.
//!
//! Every sentence is the machine's collected vocabulary speaking.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod making;

use std::ffi::OsStr;
use std::fs::{self, File};

use alo_opening::{Cannot, Decided, Kind, Named, Outcome, ThisMachine, Would, decide};
use alo_strings::Strings;

use making::a_folder_of_our_own;

/// The header of a photograph in the format telephones save, as one writes it:
/// the brand `heic`, and the brands it says it is compatible with — none of
/// which is a film.
const A_PHOTOGRAPH: &[u8] = b"\0\0\0\x28ftypheic\0\0\0\0mif1MiHEMiPrmiafMiHBheic\0\0\0\x1emeta";

/// The header of a picture in the format a web page sends, which is the same
/// family again.
const A_PICTURE: &[u8] = b"\0\0\0\x1cftypavif\0\0\0\0avifmif1miaf\0\0\0\x0emeta";

/// A machine that opens PDFs and pictures: enough for a film or a photograph to
/// reach the road for a file nothing here opens.
fn a_machine() -> ThisMachine {
    ThisMachine::with_nothing()
        .opens(Kind::Pdf)
        .and_then(|machine| machine.opens(Kind::PngImage))
        .expect("a machine that can be said")
}

/// The machine's one vocabulary, in English.
fn in_english() -> Strings {
    Strings::of(
        alo_saying::everything_this_machine_can_say()
            .expect("alo OS's own words are collected into one vocabulary"),
    )
}

/// These bytes, written under this name to a real disk, opened, and decided
/// about.
fn decided_on_disk(name: &str, bytes: &[u8]) -> Decided {
    let folder = a_folder_of_our_own("not-a-film");
    let path = folder.join(name);
    fs::write(&path, bytes).expect("the file to be written");
    let mut file = File::open(&path).expect("the file to open");
    let decided = decide(&mut file, OsStr::new(name), &a_machine()).expect("the file to be read");
    drop(file);
    drop(fs::remove_dir_all(&folder));
    decided
}

/// What a person reads, as text.
fn read(decided: Decided, strings: &Strings) -> Vec<String> {
    decided
        .said(strings)
        .iter()
        .map(|said| said.text().to_owned())
        .collect()
}

/// **A photograph from a telephone is not called a film.**
///
/// The refusal this change exists for. What a person reads says this machine
/// does not recognise the file — true, and about this machine — and then what
/// would, which is whoever sent it saying what made it. It does not say *an MP4
/// video*, and no sentence sends them looking for something to play.
#[test]
fn a_photograph_from_a_telephone_is_not_called_a_film() {
    let strings = in_english();
    let decided = decided_on_disk("IMG_4812", A_PHOTOGRAPH);
    assert_eq!(
        decided,
        Decided::AsItIs(Outcome::CannotOpen(Cannot::Unrecognised))
    );
    assert_eq!(Cannot::Unrecognised.would(), Would::WhoeverMadeIt);
    assert_eq!(
        read(decided, &strings),
        [
            "This machine does not recognise what this file is, so it has not opened it",
            "Whoever sent it can say which program made it, or send a copy saved in a different \
             format",
        ]
    );
    assert_eq!(
        decided_on_disk("cover", A_PICTURE),
        Decided::AsItIs(Outcome::CannotOpen(Cannot::Unrecognised))
    );
}

/// **A photograph named as a film is the finding**, rather than a film this
/// machine cannot open: the name claims one thing, the bytes are not it, and
/// the disagreement is said first.
#[test]
fn a_photograph_named_as_a_film_is_the_finding() {
    let strings = in_english();
    let decided = decided_on_disk("IMG_4812.mp4", A_PHOTOGRAPH);
    assert_eq!(
        decided,
        Decided::NotWhatItsNameSays {
            named: Named::A(Kind::Mp4Video),
            outcome: Outcome::CannotOpen(Cannot::Unrecognised),
        }
    );
    assert_eq!(
        read(decided, &strings).first().map(String::as_str),
        Some("This file is named as an MP4 video, but it is not one")
    );
}

/// **The films and recordings people are actually sent are still named**, from
/// the major brand or from the compatible brands beside it — so the rule that
/// stopped over-claiming did not start under-claiming.
#[test]
fn the_films_people_are_sent_are_still_named() {
    for (name, bytes, kind) in [
        (
            "holiday.mp4",
            &b"\0\0\0\x18ftypmp42\0\0\0\0"[..],
            Kind::Mp4Video,
        ),
        ("clip", &b"\0\0\0\x18ftypisom\0\0\x02\0"[..], Kind::Mp4Video),
        (
            "from-a-camera",
            &b"\0\0\0\x1cftypXAVC\0\0\0\0isommp42"[..],
            Kind::Mp4Video,
        ),
        (
            "old.mov",
            &b"\0\0\0\x14ftypqt  \0\0\x02\0qt  "[..],
            Kind::Mp4Video,
        ),
        (
            "from-a-telephone.3gp",
            &b"\0\0\0\x18ftyp3gp4\0\0\x03\0isom"[..],
            Kind::Mp4Video,
        ),
        (
            "note.m4a",
            &b"\0\0\0\x18ftypM4A \0\0\0\0isom"[..],
            Kind::Mp4Audio,
        ),
    ] {
        assert_eq!(
            decided_on_disk(name, bytes),
            Decided::AsItIs(Outcome::CannotOpen(Cannot::NothingHereOpens(kind))),
            "{name} was not read as {kind:?}"
        );
    }
}

/// **A file cut off inside its own header claims nothing more than it named.**
///
/// A download that stopped in the first twenty bytes is not a film, and the
/// sentence a person reads is the one about this machine rather than one about
/// a format the file never got as far as naming.
#[test]
fn a_header_cut_short_claims_nothing_more_than_it_named() {
    for (name, bytes) in [
        ("cut", &b"\0\0\0\x20ftypheic\0\0\0\0mif"[..]),
        ("nothing-after-it", &b"\0\0\0\x20ftyp"[..]),
    ] {
        assert_eq!(
            decided_on_disk(name, bytes),
            Decided::AsItIs(Outcome::CannotOpen(Cannot::Unrecognised)),
            "{name}"
        );
    }
}
