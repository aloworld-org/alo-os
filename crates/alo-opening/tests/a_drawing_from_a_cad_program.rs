//! **A drawing from a drawing program, recognised from its bytes** — against a
//! real one, with its provenance in `tests/files/README.md`.
//!
//! Task 6 of `docs/autonomy/v0-5-documents-and-paper-plan.md`, and the last of
//! its three formats. `docs/features.md`: ★ *"I can't open this file." A
//! `.pages`, a `.heic`, a `.dwg`: the system converts it where it can, and
//! where it cannot says plainly what will open it, instead of shrugging.*
//!
//! # Six characters, and nothing to confirm them
//!
//! A photograph is recognised from the brands its container names, and a Pages
//! document from the parts its zip holds. A drawing has neither: it begins with
//! six characters that are its version and then goes straight into its data.
//! **There is nothing after them to check the guess against**, which is why the
//! rule in `crate::looking` is a closed list of the versions that were released
//! rather than *anything beginning `AC`*. A rule that matched more than the
//! evidence supports is the thing
//! [ADR 0057](../../../docs/decisions/0057-a-format-is-recognised-on-the-evidence-of-a-real-file.md)
//! exists to refuse, and a pattern is how it would get in here.
//!
//! # Why the file is real
//!
//! Task 6: *each kind is tested against a real file with its provenance beside
//! it, and never a synthesised header alone.* The drawing is ours — a bench
//! plan, drawn with `ezdxf` — but **the DWG bytes were written by the Open
//! Design Alliance's own converter**, which is the implementation the trade
//! licenses, and were checked by converting the file back and reading what came
//! out. That is the half that matters: the rule is measured against bytes this
//! repository did not assemble.
//!
//! # And it is explained rather than converted
//!
//! Nothing this image pins reads the format — the rented office engine does
//! not — so a drawing takes task 4's road: *nothing on this machine opens it or
//! converts it into something that does*, and then what would. That is proved
//! below rather than assumed, twice: against a machine told it converts every
//! other kind there is, and against the registry that decides which kinds have a
//! conversion at all.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::ffi::OsStr;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use alo_opening::{Appears, Cannot, Decided, Kind, Macros, Outcome, ThisMachine, Would, decide};
use alo_strings::Strings;
use sha2::{Digest as _, Sha256};

/// The real drawing, and where it came from.
const THE_DRAWING: &str = "drawing.dwg";

/// The digest its provenance records.
const ITS_DIGEST: &str = "d63d11d6b592e9a09b6cbec8843723bd5a087c3f708e1708d34670a49609dddc";

/// What it weighs.
const ITS_SIZE: usize = 16_352;

/// The version its provenance records, which is the whole of what the rule
/// reads.
const ITS_VERSION: &[u8] = b"AC1032";

/// Its provenance, which is evidence rather than a note beside it.
const THE_PROVENANCE: &str = "README.md";

/// Where the closed set of conversions this repository registers is written.
const THE_REGISTRY: &str = "crates/alo-converting/src/conversion.rs";

/// Where both files live.
fn the_files() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("files")
}

/// One of them, read.
fn the_file(named: &str) -> Vec<u8> {
    fs::read(the_files().join(named)).expect("a file this crate is measured against")
}

/// A file of this repository, read.
fn of_the_repository(named: &str) -> String {
    let at = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(named);
    fs::read_to_string(&at).expect("a file of this repository")
}

/// What this machine would do with these bytes, its name claiming nothing.
///
/// The name is empty on purpose: a name that agreed with the bytes would let a
/// rule pass by reading the extension, which is what task 6 forbids.
fn outcome_of(bytes: &[u8], machine: &ThisMachine) -> Outcome {
    match decide(&mut Cursor::new(bytes), OsStr::new(""), machine)
        .expect("bytes in memory are readable")
    {
        Decided::AsItIs(outcome) | Decided::NotWhatItsNameSays { outcome, .. } => outcome,
    }
}

/// What this machine says these bytes are.
fn appears(bytes: &[u8]) -> Appears {
    outcome_of(bytes, &ThisMachine::with_nothing()).appears()
}

/// A machine told it converts **every kind there is except a drawing**.
///
/// Stronger than a list of what `alo-converting` registers today, and it cannot
/// go stale: the closed set of conversions grew while this file was being
/// written, and a helper naming its members would have said nothing more each
/// time than that somebody remembered to update it. What this machine asserts
/// is the thing that matters — a drawing is explained however much the rest of
/// the machine can do.
fn a_machine_that_converts_everything_but_a_drawing() -> ThisMachine {
    let mut machine = ThisMachine::with_nothing()
        .opens(Kind::Pdf)
        .expect("a machine may open a PDF");
    for kind in Kind::EVERY {
        if kind == Kind::AutocadDrawing || kind == Kind::Pdf {
            continue;
        }
        machine = machine
            .converts(kind, Kind::Pdf)
            .expect("a machine may convert a kind it does not open");
    }
    machine
}

/// **The file is the one its provenance describes.**
///
/// Its digest, so that swapping the file fails here rather than quietly
/// changing what every other test in this file is about.
#[test]
fn the_drawing_is_the_one_its_provenance_records() {
    let bytes = the_file(THE_DRAWING);
    assert_eq!(bytes.len(), ITS_SIZE, "the file is not the one recorded");
    let digest = format!("{:x}", Sha256::digest(&bytes));
    assert_eq!(
        digest, ITS_DIGEST,
        "the file's digest is not the one recorded"
    );
    assert_eq!(
        bytes.get(..6),
        Some(ITS_VERSION),
        "the file does not begin with the version its provenance records"
    );

    let provenance =
        fs::read_to_string(the_files().join(THE_PROVENANCE)).expect("the provenance beside it");
    assert!(
        provenance.contains(THE_DRAWING),
        "the provenance does not mention the file it is for"
    );
    assert!(
        provenance.contains(ITS_DIGEST),
        "the provenance does not record this file's digest"
    );
}

/// **A real drawing is a drawing.**
#[test]
fn a_real_drawing_is_recognised() {
    assert_eq!(
        appears(&the_file(THE_DRAWING)),
        Appears::A(Kind::AutocadDrawing)
    );
}

/// **It was not recognised before, and the sentence it got was the honest
/// one.**
///
/// Named as its own test because *unrecognised* is what a drawing answered
/// until this change, and somebody folding the rule away would otherwise put
/// that back without a test noticing.
#[test]
fn a_drawing_is_no_longer_answered_as_nothing_this_machine_knows() {
    assert_ne!(appears(&the_file(THE_DRAWING)), Appears::Unrecognised);
}

/// **It is recognised from its bytes, under a name that says something else.**
///
/// The whole of what this crate is for: an extension is a claim by whoever
/// named the file, and it is read after the bytes have decided rather than
/// instead of them.
#[test]
fn it_is_recognised_under_a_name_that_lies() {
    let bytes = the_file(THE_DRAWING);
    let decided = decide(
        &mut Cursor::new(&bytes),
        OsStr::new("plan.pdf"),
        &ThisMachine::with_nothing(),
    )
    .expect("bytes in memory are readable");
    let outcome = match decided {
        Decided::AsItIs(outcome) | Decided::NotWhatItsNameSays { outcome, .. } => outcome,
    };
    assert_eq!(outcome.appears(), Appears::A(Kind::AutocadDrawing));
    assert!(
        matches!(decided, Decided::NotWhatItsNameSays { .. }),
        "a drawing named .pdf is not reported as misnamed"
    );
}

/// **The rule admits the versions it says it admits, and nothing else.**
///
/// The one place a closed list has to be shown to be closed. The markers that
/// pass are the released versions; `AC1030` is the shape of the mistake this
/// guards — a plausible marker nobody released — and `AC9999` and a line of
/// text beginning with the same two letters are the rest of it.
#[test]
fn only_the_released_versions_are_admitted() {
    for version in [
        &b"AC1012"[..],
        b"AC1014",
        b"AC1015",
        b"AC1018",
        b"AC1021",
        b"AC1024",
        b"AC1027",
        b"AC1032",
    ] {
        let mut bytes = version.to_vec();
        bytes.extend_from_slice(&[0x00, 0x00, 0x0B, 0x00, 0x1F, 0x7F, 0x80, 0x00]);
        assert_eq!(
            appears(&bytes),
            Appears::A(Kind::AutocadDrawing),
            "{} stopped being a drawing",
            String::from_utf8_lossy(version)
        );
    }

    for not_one in [&b"AC1030"[..], b"AC1099", b"AC9999", b"AC0000", b"ACADIA"] {
        let mut bytes = not_one.to_vec();
        bytes.extend_from_slice(&[0x00, 0x00, 0x0B, 0x00, 0x1F, 0x7F, 0x80, 0x00]);
        assert_ne!(
            appears(&bytes),
            Appears::A(Kind::AutocadDrawing),
            "{} was claimed to be a drawing on no evidence",
            String::from_utf8_lossy(not_one)
        );
    }
}

/// **A letter about a drawing is a letter.**
///
/// The other half of the same worry: the marker is six printable characters, so
/// text is the thing it could swallow. A file that begins with those characters
/// and goes on in words is read as what it is.
#[test]
fn text_that_begins_with_the_same_letters_is_still_text() {
    let letter = b"ACADEMY of Fine Arts\n\nThe drawings you asked for are in the post.\n";
    assert_eq!(appears(letter), Appears::A(Kind::Text));
}

/// **A drawing is not something to play.**
///
/// `alo-playing` asks `Kind::is_played` which files it is the crate for.
#[test]
fn a_drawing_is_not_a_kind_that_is_played() {
    assert!(!Kind::AutocadDrawing.is_played());
}

/// **A drawing is not stored as a zip or as the older Office format**, so no
/// damaged container is ever consistent with a name claiming one.
#[test]
fn a_drawing_is_stored_as_neither_container() {
    assert!(!Kind::AutocadDrawing.is_stored_as_a_zip());
    assert!(!Kind::AutocadDrawing.is_stored_as_older_office());
    assert!(!Kind::AutocadDrawing.is_current_office());
}

/// **A machine with nothing explains it, and says what would open it.**
///
/// ADR 0057 decided this road for the drawing: no engine here reads the format
/// and adding one is a decision with its own cost that nothing in v0.5 asks
/// for. So it is `NothingHereOpens` with task 4's *what would* — the same road
/// a photograph takes.
#[test]
fn a_machine_with_nothing_explains_it_rather_than_shrugging() {
    assert_eq!(
        outcome_of(&the_file(THE_DRAWING), &ThisMachine::with_nothing()),
        Outcome::CannotOpen(Cannot::NothingHereOpens(Kind::AutocadDrawing))
    );
    assert_eq!(
        Cannot::NothingHereOpens(Kind::AutocadDrawing).would(),
        Would::AnotherMachineOrFormat
    );
}

/// **And so does a machine that converts everything else there is.**
///
/// The road proved rather than assumed, and proved against more than this
/// repository registers: a machine told it converts every other kind
/// `alo-opening` knows, into the PDF it opens, still explains a drawing.
#[test]
fn a_machine_that_converts_everything_else_still_explains_it() {
    assert_eq!(
        outcome_of(
            &the_file(THE_DRAWING),
            &a_machine_that_converts_everything_but_a_drawing()
        ),
        Outcome::CannotOpen(Cannot::NothingHereOpens(Kind::AutocadDrawing))
    );
}

/// **Nothing in this repository registers a conversion for a drawing**, which
/// is why the road above is the one it takes.
///
/// Read off the registry rather than reasoned about, so that the day somebody
/// registers one this test says the tests above are now describing the wrong
/// road.
#[test]
fn the_closed_set_of_conversions_holds_no_drawing() {
    let registry = of_the_repository(THE_REGISTRY);
    assert!(
        registry.contains("pub const fn of(kind: Kind)"),
        "the conversion a kind gets is no longer decided here, so reading this file proves \
         nothing about the road a drawing takes"
    );
    assert!(
        !registry.contains("AutocadDrawing"),
        "a conversion for a drawing is written, so a drawing no longer takes the road these \
         tests assert"
    );
}

/// **A machine that opens one opens it as it is.**
///
/// The other side of the same rule: recognising a drawing is only worth
/// anything if a machine that has something to show it with then shows it.
#[test]
fn a_machine_that_opens_drawings_opens_it() {
    let machine = ThisMachine::with_nothing()
        .opens(Kind::AutocadDrawing)
        .expect("a machine may open a drawing");
    assert_eq!(
        outcome_of(&the_file(THE_DRAWING), &machine),
        Outcome::OpensAsItIs {
            kind: Kind::AutocadDrawing,
            macros: Macros::NoneSeen,
        }
    );
}

/// **What a person reads names the thing and not a shop.**
///
/// ADR 0057's constraint, in the one place it could be broken: the kind carries
/// a product name because the format is that one program's, and **what would
/// open it must not**. A sentence saying *get AutoCAD* is an advertisement on a
/// machine that is meant to be telling somebody what they were sent.
#[test]
fn what_would_open_it_names_no_program() {
    let strings = Strings::of(
        alo_saying::everything_this_machine_can_say()
            .expect("alo OS's own words are collected into one vocabulary"),
    );
    let remedy = Would::AnotherMachineOrFormat
        .said(&strings)
        .text()
        .to_lowercase();
    for named in ["autocad", "autodesk"] {
        assert!(
            !remedy.contains(named),
            "what would open a drawing names {named}"
        );
    }
    assert_eq!(
        Kind::AutocadDrawing.said(&strings).text(),
        "an AutoCAD drawing"
    );
}
