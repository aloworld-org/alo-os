//! The Secure Boot state the Release's notes promise, held to the state this
//! installer really accepts.
//!
//! The installer plan's task 5: *the Release notes say … which Secure Boot
//! state it accepts.* The notes are `image/release-notes.md` and `alo-image`
//! reads them; what actually decides is `alo_installer::decide`, and the two
//! live in crates that cannot both depend on each other — `alo-installing`
//! already depends on `alo-image`, so `alo-image` asking this crate what it
//! accepts would be a cycle.
//!
//! So the rule has one home, `alo_image::ACCEPTED`, and the question is asked
//! from this end: a machine is put into each of the three states Windows can
//! report — on, off, and *could not be found out* — and what the notes promise
//! is held to which of them this program refuses. A person who reads *Secure
//! Boot: off* and restarts a computer the installer was never going to install
//! on is the failure being prevented, and it is a green build in both crates.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_image::{ACCEPTED, THE_IMAGE, THE_NOTES, THE_SECURE_BOOT, TheNotes};
use alo_installer::{BitLocker, Found, Refusal, SecurityChip, Starting, decide};

/// What the notes of the Release this repository publishes say.
fn the_notes() -> TheNotes {
    let at = Path::new(THE_IMAGE).join(THE_NOTES);
    match std::fs::read_to_string(&at) {
        Ok(notes) => TheNotes::read(&notes),
        Err(why) => panic!("{} did not read: {why}", at.display()),
    }
}

/// A computer that starts with UEFI, whose Secure Boot is in this state and
/// about which nothing else was read.
///
/// Nothing else has to be: every one of the three answers below is decided
/// before a disk is looked at, which is itself the thing worth holding — a
/// refusal for Secure Boot happens before this program has read, let alone
/// changed, anything.
fn a_machine_whose_secure_boot_is(secure_boot: Option<bool>) -> Found {
    Found {
        starting: Starting {
            uefi: Some(true),
            secure_boot,
        },
        chip: SecurityChip::NotRead,
        bitlocker: BitLocker::NotRead,
        memory: None,
        windows: None,
        disks: None,
        an_entry_is_named_alo_os: None,
    }
}

/// **The state the notes promise is the one state this installer does not
/// refuse**, and the other two are refused for being what they are.
///
/// *Could not be found out* is refused as its own answer rather than read as
/// *off* (ADR 0033 §4), and the notes say *off* because that is the only state
/// `decide` lets through.
#[test]
fn the_notes_promise_the_state_this_installer_accepts() {
    assert_eq!(
        the_notes().says(THE_SECURE_BOOT),
        Some(ACCEPTED),
        "the Release notes promise a Secure Boot state other than the one in `alo_image::ACCEPTED`"
    );
    assert_eq!(
        ACCEPTED, "off",
        "the state this release accepts moved, and every sentence about it has to move with it"
    );

    // Off: whatever this machine is refused for, it is not Secure Boot.
    assert!(
        !matches!(
            decide(&a_machine_whose_secure_boot_is(Some(false))),
            Err(Refusal::SecureBootOn | Refusal::SecureBootNotRead)
        ),
        "Secure Boot off is refused for being Secure Boot, and the notes promise it installs"
    );

    // On, and not known: each refused, and each for itself.
    assert!(
        matches!(
            decide(&a_machine_whose_secure_boot_is(Some(true))),
            Err(Refusal::SecureBootOn)
        ),
        "Secure Boot on is not refused, and the notes say this release does not install with it"
    );
    assert!(
        matches!(
            decide(&a_machine_whose_secure_boot_is(None)),
            Err(Refusal::SecureBootNotRead)
        ),
        "a Secure Boot state Windows would not report is read as off somewhere"
    );
}

/// **Nothing a person reads about it names a key, a password or a signature**
/// (ADR 0036, as the owner accepted it, and ADR 0046 for the installer's own).
///
/// The notes are the one document in this release written for the person
/// downloading rather than for this repository, and the sentence about the
/// download being genuine is the one somebody will one day want to explain with
/// the word *signature* in it.
#[test]
fn the_notes_name_no_key_no_password_and_no_signature() {
    let at = Path::new(THE_IMAGE).join(THE_NOTES);
    let notes = match std::fs::read_to_string(&at) {
        Ok(notes) => notes.to_lowercase(),
        Err(why) => panic!("{} did not read: {why}", at.display()),
    };

    for never in [
        "signature",
        "password",
        "public key",
        "private key",
        "cosign",
    ] {
        assert!(
            !notes.contains(never),
            "the Release notes say `{never}`, and a person never sees one"
        );
    }
}

/// **And the notes never tell anybody to change the firmware setting** (ADR
/// 0033 §4), however helpfully it could be phrased.
#[test]
fn the_notes_never_ask_anybody_to_change_the_setting() {
    let at = Path::new(THE_IMAGE).join(THE_NOTES);
    let notes = match std::fs::read_to_string(&at) {
        Ok(notes) => notes.to_lowercase(),
        Err(why) => panic!("{} did not read: {why}", at.display()),
    };

    for advice in [
        "turn it off",
        "turn off secure boot",
        "switch it off",
        "switch off secure boot",
        "disable secure boot",
    ] {
        assert!(
            !notes.contains(advice),
            "the Release notes say `{advice}` — a person is never asked to alter a firmware \
             setting"
        );
    }
}
