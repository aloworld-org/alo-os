//! **A photograph from a telephone, recognised from its bytes** — against a
//! real file, with its provenance in `tests/files/README.md`.
//!
//! Part of task 6 of `docs/autonomy/v0-5-documents-and-paper-plan.md`, whose
//! promise in `docs/features.md` is ★ *"I can't open this file." A `.pages`, a
//! `.heic`, a `.dwg`: the system converts it where it can, and where it cannot
//! says plainly what will open it, instead of shrugging.* The plan says a photo
//! from a phone is the commonest of the three.
//!
//! # This was worse than a shrug
//!
//! The plan was written when all three read as *this machine does not recognise
//! what this file is* — honest, if not the sentence the promise describes. By
//! the time this test was written a HEIC read as **a film**: the media kinds
//! added for ADR 0051 gave `ftyp` at offset four a meaning, and every ISO media
//! file that was not `M4A` or `M4B` fell through to `Mp4Video`. HEIF and MP4 are
//! the same container, and the brand is the only thing that separates them.
//!
//! Being confidently wrong is a worse failure than admitting ignorance, and it
//! is the failure this file exists to stop coming back.
//!
//! # Why the file is real
//!
//! Task 6: *each kind is tested against a real file with its provenance beside
//! it, and never a synthesised header alone.* A header this repository assembled
//! would test that the rule matches what we think the format is. The file here
//! was written by the same library that writes them on a telephone, and carries
//! the boxes a real one carries in the order it really puts them.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::ffi::OsStr;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use alo_opening::{Appears, Cannot, Decided, Kind, Macros, Outcome, ThisMachine, Would, decide};

/// The real photograph, and where it came from.
const THE_PHOTO: &str = "photo.heic";

/// Its provenance, which is part of the evidence rather than a note beside it.
const THE_PROVENANCE: &str = "README.md";

/// Where both live.
fn the_files() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("files")
}

/// One of them, read.
fn the_file(named: &str) -> Vec<u8> {
    fs::read(the_files().join(named)).expect("a file this crate is measured against")
}

/// What this machine says these bytes are.
fn appears(bytes: &[u8]) -> Appears {
    outcome_of(bytes, &ThisMachine::with_nothing()).appears()
}

/// What this machine would do with these bytes, its name claiming nothing.
///
/// The name is empty on purpose: what this file is about is the bytes, and a
/// name that agreed with them would let a rule pass by reading the extension —
/// which is exactly what task 6 forbids.
fn outcome_of(bytes: &[u8], machine: &ThisMachine) -> Outcome {
    match decide(&mut Cursor::new(bytes), OsStr::new(""), machine)
        .expect("bytes in memory are readable")
    {
        Decided::AsItIs(outcome) | Decided::NotWhatItsNameSays { outcome, .. } => outcome,
    }
}

/// **A real photograph from a telephone is a photograph.**
#[test]
fn a_real_heic_is_recognised_as_a_photo() {
    assert_eq!(appears(&the_file(THE_PHOTO)), Appears::A(Kind::HeicPhoto));
}

/// **And it is not a film**, which is what it used to be.
///
/// Written as its own test, naming the wrong answer, because the assertion
/// above would still pass if somebody later folded photographs back into the
/// film branch under a different name.
#[test]
fn a_photo_is_never_reported_as_a_film() {
    let decided = appears(&the_file(THE_PHOTO));
    assert_ne!(decided, Appears::A(Kind::Mp4Video));
    assert_ne!(decided, Appears::A(Kind::Mp4Audio));
    assert_eq!(decided, Appears::A(Kind::HeicPhoto));
}

/// **A photograph is not something to play.**
///
/// `alo-playing` asks `Kind::is_played` which files it is the crate for. A
/// photograph answering yes would send a still picture to be decoded as tracks,
/// and it has none.
#[test]
fn a_photo_is_not_a_kind_that_is_played() {
    assert!(!Kind::HeicPhoto.is_played());
}

/// **A machine that cannot open one says so, and says what would.**
///
/// No engine the image pins converts a HEIF photograph — the office engine
/// converts documents — so this is the `NothingHereOpens` road with task 4's
/// *what would*, and not a `Converts` this machine cannot perform.
#[test]
fn a_machine_with_nothing_explains_it_rather_than_shrugging() {
    assert_eq!(
        outcome_of(&the_file(THE_PHOTO), &ThisMachine::with_nothing()),
        Outcome::CannotOpen(Cannot::NothingHereOpens(Kind::HeicPhoto))
    );
    assert_eq!(
        Cannot::NothingHereOpens(Kind::HeicPhoto).would(),
        Would::AnotherMachineOrFormat
    );
}

/// **A machine that opens one opens it as it is.**
///
/// The other side of the same rule: recognising a photograph is only worth
/// anything if a machine with a viewer then shows it rather than explaining it.
#[test]
fn a_machine_with_a_viewer_opens_it() {
    let machine = ThisMachine::with_nothing()
        .opens(Kind::HeicPhoto)
        .expect("a machine may open a photograph");
    assert_eq!(
        outcome_of(&the_file(THE_PHOTO), &machine),
        Outcome::OpensAsItIs {
            kind: Kind::HeicPhoto,
            macros: Macros::NoneSeen,
        }
    );
}

/// **The file is what the provenance says it is.**
///
/// The provenance is evidence, so it is checked rather than trusted: a README
/// describing a file that was later replaced would be a test passing against a
/// story. The brands are read out of the file's own `ftyp` box.
#[test]
fn the_file_matches_what_its_provenance_records() {
    let bytes = the_file(THE_PHOTO);
    assert_eq!(bytes.len(), 37_008, "the file is not the one recorded");
    assert_eq!(
        bytes.get(4..8),
        Some(&b"ftyp"[..]),
        "the file does not begin as an ISO media container"
    );
    assert_eq!(
        bytes.get(8..12),
        Some(&b"heic"[..]),
        "the file's major brand is not the one recorded"
    );

    let provenance = fs::read_to_string(the_files().join(THE_PROVENANCE))
        .expect("the provenance beside the files");
    assert!(
        provenance.contains(THE_PHOTO),
        "the provenance does not mention the file it is for"
    );
    assert!(
        provenance.contains("2bb53cff09dad7a4a31963e609cf2081081ceb225c82f18b39dcae3850b98368"),
        "the provenance does not record this file's digest"
    );
}

/// **A film in the same container is still a film.**
///
/// The rule separates two things that share a container, so it has to be shown
/// not to have swallowed the other one. The brands here are the ones a real
/// recording carries, checked against the same rule rather than against a file,
/// because what is being tested is the four bytes and nothing else.
#[test]
fn the_brands_a_recording_carries_are_still_films() {
    for brand in [&b"isom"[..], b"mp42", b"avc1", b"hevc", b"qt  "] {
        let mut bytes = vec![0, 0, 0, 0x18];
        bytes.extend_from_slice(b"ftyp");
        bytes.extend_from_slice(brand);
        bytes.extend_from_slice(&[0; 16]);
        assert_eq!(
            appears(&bytes),
            Appears::A(Kind::Mp4Video),
            "{} stopped being a film",
            String::from_utf8_lossy(brand)
        );
    }
    for brand in [&b"M4A "[..], b"M4B "] {
        let mut bytes = vec![0, 0, 0, 0x18];
        bytes.extend_from_slice(b"ftyp");
        bytes.extend_from_slice(brand);
        bytes.extend_from_slice(&[0; 16]);
        assert_eq!(
            appears(&bytes),
            Appears::A(Kind::Mp4Audio),
            "{} stopped being a sound recording",
            String::from_utf8_lossy(brand)
        );
    }
}
