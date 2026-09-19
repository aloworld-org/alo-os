//! Nothing is enrolled on any disk, virtual or real, until it is decided **how
//! an enrolment is shown to work** — and once that is decided, this test fails,
//! which is the next worker's instruction to build what it decided.
//!
//! # Which decision this waits on, and why it changed
//!
//! It waited on
//! [ADR 0054](../../../docs/decisions/0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md),
//! *the disk is sealed to this machine and opened with a PIN*, which the owner
//! **accepted** on 2026-09-19. That is the road, and it is settled: the order,
//! PCR 7, the PIN's length, the recovery key and its consumption are not
//! reopened by anything here.
//!
//! What stopped task 6 of `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`
//! was not the road but its acceptance. That acceptance asks for the sequence to
//! be run *with a software TPM in a virtual machine*, and **no machine in this
//! fleet can present a TPM device to anything**: the Linux the gates run in is
//! built with `CONFIG_TCG_VTPM_PROXY` unset, so there is no `/dev/vtpmx` for a
//! software chip to appear through, and the host has neither `vmx` nor `svm`, so
//! every virtual machine on it is emulated — the one this repository already
//! owns took 3779 seconds. So the question *what does a virtual disk show about
//! encryption, and what only a machine with a chip can* is
//! [ADR 0056](../../../docs/decisions/0056-a-sealed-disks-promise-is-shown-on-a-machine-with-a-chip.md),
//! **proposed**, and this test now waits on that instead. The measurements are
//! in it rather than repeated here.
//!
//! # Two things held, for exactly as long as both statuses say what they say
//!
//! - the road is decided and this crate still takes no step of it — no rented
//!   tool, no device, no argument list anywhere in its code;
//! - the decision about how it is shown exists, is one somebody wrote, and names
//!   both halves it splits the showing into.
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

/// The decision that settled the road, and is accepted.
const THE_ROADS_DECISION: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/decisions/0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md"
);

/// The decision that settles how an enrolment is shown to work, and is not.
const THE_SHOWINGS_DECISION: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../docs/decisions/0056-a-sealed-disks-promise-is-shown-on-a-machine-with-a-chip.md"
);

/// **While it is undecided how an enrolment is shown to work, nothing here
/// enrols anything.** The day ADR 0056 says accepted, this fails: build the
/// sequence and the virtual-disk acceptance it decides, and replace this test
/// with the tests of that.
///
/// It also fails if ADR 0054 ever stops being accepted, because a crate that
/// waits on *how to show* a road nobody has decided is waiting on the wrong
/// thing.
#[test]
fn nothing_is_enrolled_while_how_it_is_shown_is_undecided() {
    let road = status_of(THE_ROADS_DECISION);
    assert!(
        road.contains("accepted"),
        "ADR 0054 no longer says the road is accepted ({road}). This crate waits on how that \
         road is shown to work, which is a question that only exists once there is a road"
    );

    let showing = status_of(THE_SHOWINGS_DECISION);
    assert!(
        showing.contains("proposed"),
        "ADR 0056 is no longer proposed ({showing}). Build what it decided — the enrolment \
         sequence, the sentences handed to the installer plan, and the virtual-disk acceptance \
         of everything that needs no chip — and replace this test with the tests of that"
    );

    // The road is named and nothing takes a step of it: no rented tool, no
    // device, no argument list anywhere in the crate.
    assert_eq!(THE_ROAD.len(), 6);
    assert_eq!(THE_ROAD.first(), Some(&Step::TheDiskIsMadeIntoAVolume));
    assert_eq!(
        THE_ROAD.last(),
        Some(&Step::TheInstallersFirstKeyIsWipedAway)
    );
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

/// **The decision this crate waits on is one somebody wrote**, and it says what
/// the two halves of showing an enrolment are.
///
/// A crate that waits on a number is waiting on nothing. `alo-citing` holds
/// every pointer in the repository to a file that exists; this holds the one
/// pointer that decides whether there is any code in this crate at all to the
/// thing it has to say.
#[test]
fn the_decision_about_showing_it_names_both_halves() {
    let decision = read(THE_SHOWINGS_DECISION);

    assert!(
        decision.contains("ADR 0054") || decision.contains("0054-the-disk-is-sealed"),
        "ADR 0056 does not cite the road it is about showing"
    );
    for half in [
        "a virtual disk",
        "a machine with a chip",
        "CONFIG_TCG_VTPM_PROXY",
    ] {
        assert!(
            decision.contains(half),
            "ADR 0056 does not say `{half}`, and the split it is asking for is exactly that \
             line: what a virtual disk shows, what only a chip can, and the measurement that \
             says why"
        );
    }
    assert!(
        decision.contains("## The recommendation"),
        "a decision with no recommendation is a description, and the owner cannot accept one"
    );
}

/// A decision's text, or a failure naming the file that could not be read.
fn read(decision: &str) -> String {
    let text = std::fs::read_to_string(decision).unwrap_or_default();
    assert!(!text.is_empty(), "{decision} could not be read");
    text
}

/// A decision's status line, which every decision in this repository records.
fn status_of(decision: &str) -> String {
    let text = read(decision);
    let Some(status) = text.lines().find(|line| line.starts_with("**Status:**")) else {
        panic!("{decision} records no status")
    };
    status.to_owned()
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
