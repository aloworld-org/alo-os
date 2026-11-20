//! **The check as `every-try.yml` runs it**, with no provider key: which way
//! it goes is decided before anybody is asked, so each way can be seen here
//! without leaving the machine.
//!
//! A change that owns nothing exits 0 and says so; a change that owns a
//! request goes on to ask the model (and here stops at the missing key); a
//! base declaration that does not read, or a count that is not one, is not a
//! measurement and never exits 0.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::path::PathBuf;
use std::process::{Command, Output};

/// The committed declaration, where the workflow's checkout has it.
fn the_committed() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("what-the-model-is-offered.txt")
}

/// A file of this test's own, so that two runs on one machine never share
/// one.
fn a_file(named: &str) -> PathBuf {
    std::env::temp_dir().join(format!("alo-every-try-{}-{named}", std::process::id()))
}

/// The check, with nothing from the environment it was run in and no key.
fn the_check() -> Command {
    let mut check = Command::new(env!("CARGO_BIN_EXE_alo-every-try"));
    for name in [
        "ALO_PROVIDER_KEY",
        "ALO_EVERY_TRY_BEFORE",
        "ALO_EVERY_TRY_TRIES",
        "ALO_EVERY_TRY_OTHER_TRIES",
        "ALO_EVERY_TRY_ONLY",
        "ALO_EVERY_TRY_SUMMARY",
        "ALO_EVERY_TRY_SHOWN",
        "ALO_EVERY_TRY_LEVEL",
    ] {
        check.env_remove(name);
    }
    check
}

fn ran(check: &mut Command) -> Output {
    check.output().expect("the check runs")
}

fn said(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// The committed declaration with one request's line changed, as a change to
/// that request would leave the base's beside it.
fn a_base_where_listing_a_folder_was_otherwise() -> PathBuf {
    let committed = std::fs::read_to_string(the_committed()).expect("it reads");
    let changed: String = committed
        .lines()
        .map(|line| {
            if line.starts_with("request\tlist a folder\t") {
                line.replacen(
                    "request\tlist a folder\t",
                    "request\tlist a folder\tchanged",
                    1,
                )
            } else {
                line.to_owned()
            }
        })
        .map(|line| line + "\n")
        .collect();
    let to = a_file("before-changed.txt");
    std::fs::write(&to, changed).expect("it is written");
    to
}

/// **A change that owns nothing passes without a key**, and its summary says
/// why.
#[test]
fn a_change_that_owns_nothing_passes_without_asking_anybody() {
    let summary = a_file("summary-nothing.md");
    let _ = std::fs::remove_file(&summary);
    let output = ran(the_check()
        .env("ALO_EVERY_TRY_BEFORE", the_committed())
        .env("ALO_EVERY_TRY_SUMMARY", &summary));
    assert_eq!(output.status.code(), Some(0), "{}", said(&output));
    let written = std::fs::read_to_string(&summary).expect("the summary is written");
    assert!(
        written.contains("alters nothing the model is offered"),
        "{written}"
    );
    let _ = std::fs::remove_file(&summary);
}

/// **A change that owns a request goes on to ask the model**: here it stops
/// at the key, which proves it did not pass on nothing.
#[test]
fn a_change_that_owns_a_request_goes_on_to_ask_the_model() {
    let before = a_base_where_listing_a_folder_was_otherwise();
    let output = ran(the_check().env("ALO_EVERY_TRY_BEFORE", &before));
    assert_eq!(output.status.code(), Some(2), "{}", said(&output));
    assert!(
        said(&output).contains("ALO_PROVIDER_KEY is not set"),
        "{}",
        said(&output)
    );
    // And it is the change's own that it is choosing, not the whole set: the
    // two ways of choosing are refused together before anything is asked.
    let output = ran(the_check()
        .env("ALO_EVERY_TRY_BEFORE", &before)
        .env("ALO_EVERY_TRY_ONLY", "list a folder"));
    assert_eq!(output.status.code(), Some(2), "{}", said(&output));
    assert!(said(&output).contains("two ways"), "{}", said(&output));
    let _ = std::fs::remove_file(&before);
}

/// **The refusal cases**: a base that does not read, and counts that are not
/// counts, are not measurements and never pass.
#[test]
fn what_cannot_be_read_never_passes() {
    let bad = a_file("before-bad.txt");
    std::fs::write(&bad, "nonsense\n").expect("it is written");
    let output = ran(the_check().env("ALO_EVERY_TRY_BEFORE", &bad));
    assert_eq!(output.status.code(), Some(2), "{}", said(&output));
    assert!(said(&output).contains("does not read"), "{}", said(&output));
    let _ = std::fs::remove_file(&bad);

    let output = ran(the_check().env("ALO_EVERY_TRY_TRIES", "ten"));
    assert_eq!(output.status.code(), Some(2), "{}", said(&output));
    assert!(
        said(&output).contains("ALO_EVERY_TRY_TRIES is a whole number"),
        "{}",
        said(&output)
    );
    let output = ran(the_check().env("ALO_EVERY_TRY_TRIES", "0"));
    assert_eq!(output.status.code(), Some(2), "{}", said(&output));
    assert!(said(&output).contains("above zero"), "{}", said(&output));
}

/// **`declare` writes what the code offers**, which is what is committed.
#[test]
fn declare_writes_what_the_code_offers() {
    let to = a_file("declared.txt");
    let output = ran(the_check().arg("declare").arg(&to));
    assert_eq!(output.status.code(), Some(0), "{}", said(&output));
    assert_eq!(
        std::fs::read_to_string(&to).expect("it was written"),
        std::fs::read_to_string(the_committed()).expect("it reads")
    );
    let _ = std::fs::remove_file(&to);
}
