//! **The real verifier, on a real Craft signature** (ADR 0158).
//!
//! `fixtures/signed-release/` holds PhotoCraft 0.23.1's `SHA256SUMS.txt` and
//! its detached signature `SHA256SUMS.txt.asc`, exactly as the release served
//! them on 2026-11-17 — the smallest file that maker signs. These run gpgv
//! itself, with the keyring this crate builds from the pinned key, so they
//! check the thing the machine runs rather than a stand-in for it:
//!
//! - the pinned key verifies the maker's own signature, and the verdict names
//!   the pinned fingerprint;
//! - one byte changed, and it is refused;
//! - another file's signature, and it is refused;
//! - no signature at all, and it is refused.
//!
//! **Linux only, and gpgv must be there.** These run on every build on the
//! Ubuntu runner, which has it. A Linux machine without `/usr/bin/gpgv` fails
//! here rather than passing over nothing — a skipped check is a check that
//! reads as green.

#![cfg(target_os = "linux")]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_software::signature::THE_VERIFIER;
use alo_software::{Maker, TheRentedVerifier, Verifier};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/signed-release")
        .join(name)
}

/// A folder holding the keyring built from the pinned key.
fn the_keyring() -> (tempfile::TempDir, PathBuf) {
    assert!(
        Path::new(THE_VERIFIER).exists(),
        "{THE_VERIFIER} is not on this machine, so nothing here can be checked"
    );
    let folder = tempfile::tempdir().unwrap();
    let keyring = folder.path().join("maker.gpg");
    std::fs::write(
        &keyring,
        Maker::named("storytold").unwrap().keyring().unwrap(),
    )
    .unwrap();
    (folder, keyring)
}

#[test]
fn the_makers_own_signature_names_the_pinned_key() {
    let (_folder, keyring) = the_keyring();
    let signer = TheRentedVerifier::on_this_machine()
        .signer(
            &fixture("SHA256SUMS.txt"),
            &fixture("SHA256SUMS.txt.asc"),
            &keyring,
        )
        .unwrap();
    assert_eq!(
        signer.as_deref(),
        Some(Maker::named("storytold").unwrap().fingerprint())
    );
}

#[test]
fn one_byte_changed_is_refused() {
    let (folder, keyring) = the_keyring();
    let changed = folder.path().join("SHA256SUMS.txt");
    let mut bytes = std::fs::read(fixture("SHA256SUMS.txt")).unwrap();
    let first = bytes.first_mut().unwrap();
    *first ^= 1;
    std::fs::write(&changed, bytes).unwrap();
    let signer = TheRentedVerifier::on_this_machine()
        .signer(&changed, &fixture("SHA256SUMS.txt.asc"), &keyring)
        .unwrap();
    assert_eq!(signer, None);
}

#[test]
fn another_files_signature_is_refused() {
    let (folder, keyring) = the_keyring();
    let other = folder.path().join("other.txt");
    std::fs::write(&other, b"not what was signed\n").unwrap();
    let signer = TheRentedVerifier::on_this_machine()
        .signer(&other, &fixture("SHA256SUMS.txt.asc"), &keyring)
        .unwrap();
    assert_eq!(signer, None);
}

#[test]
fn no_signature_is_refused() {
    let (folder, keyring) = the_keyring();
    let empty = folder.path().join("empty.asc");
    std::fs::write(&empty, b"").unwrap();
    let signer = TheRentedVerifier::on_this_machine()
        .signer(&fixture("SHA256SUMS.txt"), &empty, &keyring)
        .unwrap();
    assert_eq!(signer, None);
    let absent = TheRentedVerifier::on_this_machine()
        .signer(
            &fixture("SHA256SUMS.txt"),
            &folder.path().join("never-written.asc"),
            &keyring,
        )
        .unwrap();
    assert_eq!(absent, None);
}
