//! **Every date this machine shows goes through this crate.**
//!
//! Task 6 of `docs/autonomy/v0-5-access-and-language-plan.md`. A date written
//! anywhere else is a date written the way whoever typed it writes dates, which
//! is English and American about half the time and is never Maltese. So this
//! reads the shipped source of every crate and fails on a date built by hand.
//!
//! **What counts as building one by hand**: a `strftime`-style pattern, or a
//! format string that puts the parts of a date together with slashes, dots or
//! dashes. Test code is not read — a test may write `2026-09-17` and mean it —
//! and neither is this crate, which is where the real one is built.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// The ways a date gets written by hand.
const A_DATE_BY_HAND: [&str; 6] = [
    "%Y-%m-%d", "%d/%m/%Y", "%m/%d/%Y", "%d.%m.%Y", "%Y/%m/%d", "%d-%m-%Y",
];

/// **Crates that write a date that is not shown to a person**, and why each is
/// here — the list a lane that trips this test adds to, when its date is a wire
/// format rather than something somebody reads.
///
/// - `alo-record`: what happened on this machine, read back by a later reader
///   and by another machine, which must agree about when.
/// - `alo-keeping`: the same, for what is kept.
/// - `alo-models`: a catalogue grade says the day it was earned, and two
///   machines compare those days.
/// - `alo-driving`: a measurement says when it ran, for the same reason.
///
/// All four write ISO 8601, which is nobody's regional format and is not meant
/// to be read as one. **A date on a screen is a different thing**, and belongs
/// in `alo-formats` — the question that decides it is who reads this.
const NOT_SHOWN_TO_A_PERSON: [&str; 4] = ["alo-record", "alo-keeping", "alo-models", "alo-driving"];

#[test]
fn no_crate_writes_a_date_by_hand() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the crates folder")
        .to_owned();
    let mut written_by_hand: Vec<String> = Vec::new();
    for source in every_shipped_source(&crates) {
        let text = std::fs::read_to_string(&source).unwrap_or_default();
        for pattern in A_DATE_BY_HAND {
            if text.contains(pattern) {
                written_by_hand.push(format!("{} writes {pattern}", source.display()));
            }
        }
    }
    assert!(
        written_by_hand.is_empty(),
        "A date is written by hand rather than through `alo-formats`, so it is written the way \
         whoever typed it writes dates — English and American for everybody who is neither.\n\n\
         {written_by_hand:#?}\n\n\
         If this is a date **a person reads**, build it with `alo_formats::Regionally`, which \
         asks CLDR how that person's language writes it.\n\
         If it is a date **written down for a machine** — a record entry, a grade, anything a \
         later reader or another machine must agree with — it is a wire format and this rule \
         does not apply to it: add the crate to NOT_SHOWN_TO_A_PERSON below, with the reason, \
         in the same change. Four crates are there already: alo-record, alo-keeping, alo-models \
         and alo-driving, all of which write ISO 8601 on purpose.\n\
         The question that decides it is *who reads this*, and nothing else."
    );
}

/// Every `src` file of every crate but this one, tests excluded.
fn every_shipped_source(crates: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(crates) else {
        return found;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "alo-formats" || NOT_SHOWN_TO_A_PERSON.contains(&name.as_str()) {
            continue;
        }
        walk(&entry.path().join("src"), &mut found);
    }
    found
}

/// Every `.rs` under a folder, but not a file whose name says it is tests.
fn walk(folder: &Path, found: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, found);
        } else if path.extension().is_some_and(|kind| kind == "rs")
            && !path
                .file_name()
                .is_some_and(|name| name.to_string_lossy().contains("test"))
        {
            found.push(path);
        }
    }
}
