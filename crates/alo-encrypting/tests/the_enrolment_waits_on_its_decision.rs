//! Nothing is enrolled on any disk, virtual or real, until the decision about
//! where this machine's key lives is accepted — **and once it is accepted, this
//! test fails**, which is the next worker's instruction to build what it
//! decided.
//!
//! Task 5 of `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md` is a decision
//! and the shape it leaves behind:
//! [ADR 0054](../../../docs/decisions/0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md),
//! proposed. Its constraint is *proposed, and marked so; tasks 6 and 7 wait on
//! it. No encryption is enrolled on any real disk by any test.* Two things are
//! held here for exactly as long as that is true: this crate takes no step of
//! [`alo_encrypting::THE_ROAD`] and could not, and the decision still says what
//! it says.
//!
//! The pattern is `crates/alo-brokerd/tests/the_updates_wait_on_their_decision.rs`'s,
//! and it is here for the reason it is there: a task that waits on somebody
//! else's answer leaves a green suite behind it, and a green suite is how a
//! decision gets forgotten.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_encrypting::{Step, THE_ROAD};

/// The decision this crate waits on.
const THE_DECISION: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/decisions/0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md"
);

/// **While the decision is proposed, nothing here enrols anything.** The day it
/// says accepted, this fails: turn [`THE_ROAD`] into the tested command sequence
/// against a virtual disk that task 6 asks for, and replace this test with the
/// tests of that.
#[test]
fn nothing_is_enrolled_while_its_decision_is_proposed() {
    let decision = std::fs::read_to_string(THE_DECISION).unwrap_or_default();
    assert!(!decision.is_empty(), "{THE_DECISION} could not be read");
    let Some(status) = decision
        .lines()
        .find(|line| line.starts_with("**Status:**"))
    else {
        panic!("ADR 0054 records no status")
    };
    assert!(
        status.contains("proposed"),
        "ADR 0054 is no longer proposed ({status}). Build what it decided — enrolment during \
         install, and the recovery — and replace this test with the tests of that"
    );

    // The road is named and nothing takes a step of it: no rented tool, no
    // device, no argument list anywhere in the crate.
    assert_eq!(THE_ROAD.len(), 6);
    assert_eq!(THE_ROAD.first(), Some(&Step::TheDiskIsMadeIntoAVolume));
    for (name, text) in the_source() {
        // Comments may name a rented tool — `road.rs` says which ones it is
        // deliberately not running, and a reader needs that. Code may not.
        let text: String = text
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<&str>>()
            .join("\n");
        for taking in [
            "cryptsetup",
            "systemd-cryptenroll",
            "luksFormat",
            "bootc",
            "/dev/",
            "--tpm2",
            "--recovery-key",
            "crypttab",
        ] {
            assert!(
                !text.contains(taking),
                "{name} names {taking}: this crate decides the road and takes none of it"
            );
        }
    }
}

/// Every Rust file under `src/`, as `(name, text)`.
fn the_source() -> Vec<(String, String)> {
    let mut files: Vec<(String, String)> =
        std::fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("src"))
            .into_iter()
            .flatten()
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
            .map(|path| {
                let name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_default();
                (name, std::fs::read_to_string(&path).unwrap_or_default())
            })
            .collect();
    files.sort();
    assert_eq!(files.len(), 9, "src/ could not be read");
    files
}
