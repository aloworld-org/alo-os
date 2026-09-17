//! **The recovery key is shown once and is never stored on the disk it
//! recovers**, and neither is the PIN or the passphrase.
//!
//! ADR 0054 says the key *exists as a value in one process, for the length of
//! one screen*. A sentence like that is worth nothing unless the crate holding
//! the value is incapable of doing anything else with it, so this test measures
//! the incapability three ways:
//!
//! - **it depends on nothing**, so there is no serialiser, no client and no
//!   logger in reach;
//! - **its source names no file, no program and no socket**, so even `std` is
//!   not being used to keep one;
//! - **no secret appears in any line it can print**, held against the values
//!   rather than against the source.
//!
//! A dependency added to this crate fails the first of those, and the argument
//! for adding it then has to be made here, in front of a reader, rather than in
//! a manifest nobody opens.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_encrypting::{Enrolment, HowItUnlocks, Passphrase, Pin, RecoveryKey, TheChip};

/// What the rented tool printed on a virtual disk in the pinned base.
const THE_KEY: &str = "lrhdhrji-vtdlgdkr-ulukcjff-rgckhvrd-kguhibgc-gvihrfgc-ghedrdlr-uihctkkt";

/// A PIN nobody else uses, so that finding it in a line is unambiguous.
const THE_PIN: &str = "zwqx-pin-4417";

/// A passphrase nobody else uses, for the same reason.
const THE_PASSPHRASE: &str = "zwqx passphrase 4417";

/// This crate's directory.
fn here() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

/// Every Rust file under `src/`, as `(name, text)`.
fn the_source() -> Vec<(String, String)> {
    let mut files: Vec<(String, String)> = std::fs::read_dir(here().join("src"))
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

/// **This crate depends on nothing.** Not a serialiser, not a client, not a
/// logger, not a crate of ours — so a secret it holds has nowhere to go that is
/// not `std`, and the next test says what it does with that.
#[test]
fn this_crate_depends_on_nothing() {
    let manifest = std::fs::read_to_string(here().join("Cargo.toml")).unwrap_or_default();
    assert!(!manifest.is_empty(), "the manifest could not be read");
    let named: Vec<&str> = manifest
        .lines()
        .map(str::trim)
        .filter(|line| !line.starts_with('#') && !line.starts_with('['))
        .filter_map(|line| line.split_once('='))
        .map(|(name, _)| name.trim())
        .filter(|name| {
            ![
                "name",
                "version",
                "description",
                "edition.workspace",
                "rust-version.workspace",
                "license.workspace",
                "repository.workspace",
                "workspace",
            ]
            .contains(name)
        })
        .collect();
    assert!(
        named.is_empty(),
        "a dependency was added to a crate that holds a recovery key: {named:?}"
    );
    assert!(
        !manifest.contains("[dependencies]")
            && !manifest.contains("[dev-dependencies]")
            && !manifest.contains("[build-dependencies]"),
        "{manifest}"
    );
}

/// **Nothing here opens a file, runs a program or reaches a network.** There is
/// therefore no road from a value in this crate to the disk it recovers, to a
/// log, or to anywhere else.
#[test]
fn nothing_here_opens_a_file_or_runs_a_program_or_reaches_a_network() {
    for (name, text) in the_source() {
        for reach in [
            "std::fs",
            "fs::write",
            "File::",
            "OpenOptions",
            "std::process",
            "Command",
            "std::net",
            "TcpStream",
            "UdpSocket",
            "UnixStream",
            "std::env",
            "env::var",
            "println!",
            "eprintln!",
            "Serialize",
            "Deserialize",
            "serde",
            "to_json",
            "unsafe",
        ] {
            assert!(
                !text.contains(reach),
                "{name} names {reach}, and a crate that holds a recovery key may not"
            );
        }
    }
}

/// **Every secret this crate holds has a `Debug` that says nothing**, written by
/// hand rather than derived, and a test beside each. Derived, one of them would
/// put a person's PIN in every line that ever printed an enrolment.
#[test]
fn no_secret_is_in_any_line_this_crate_can_print() {
    let key = match RecoveryKey::as_printed(&format!("{THE_KEY}\n")) {
        Ok(key) => key,
        Err(why) => panic!("the tool's own output is a recovery key: {why}"),
    };
    let said = format!("{key:?}");
    assert!(!said.contains("lrhdhrji"), "{said}");

    let kept = match key.written_back(THE_KEY) {
        Ok(kept) => kept,
        Err(again) => panic!("the key typed back is the key: {:?}", again.why()),
    };

    let pin = match Pin::typed(THE_PIN, THE_PIN) {
        Ok(pin) => pin,
        Err(why) => panic!("{THE_PIN} is a PIN: {why}"),
    };
    assert!(!format!("{pin:?}").contains(THE_PIN));

    let passphrase = match Passphrase::typed(THE_PASSPHRASE, THE_PASSPHRASE) {
        Ok(passphrase) => passphrase,
        Err(why) => panic!("{THE_PASSPHRASE} is a passphrase: {why}"),
    };
    assert!(!format!("{passphrase:?}").contains("zwqx"));

    let how = match HowItUnlocks::sealed_to_the_chip(TheChip::ReadyToUse, pin) {
        Ok(how) => how,
        Err(why) => panic!("a chip that is ready can hold a key: {why}"),
    };
    assert!(!format!("{how:?}").contains("zwqx"));
    assert!(!format!("{:?}", HowItUnlocks::a_passphrase(passphrase)).contains("zwqx"));

    let enrolment = Enrolment::made(how, kept);
    let whole = format!("{enrolment:?}");
    assert!(!whole.contains("zwqx"), "{whole}");
    assert!(!whole.contains("lrhdhrji"), "{whole}");
}

/// **No secret has a `Display`**, so none of them can reach a string by being
/// written into one, and `to_string` on one does not compile. Held by reading
/// the source, because a trait that is not implemented cannot be tested for.
#[test]
fn no_secret_can_be_written_into_a_sentence() {
    for (file, secret) in [
        ("recovery_key.rs", "RecoveryKey"),
        ("pin.rs", "Pin"),
        ("passphrase.rs", "Passphrase"),
    ] {
        let text = the_source()
            .into_iter()
            .find(|(name, _)| name == file)
            .map(|(_, text)| text)
            .unwrap_or_default();
        assert!(
            !text.contains(&format!("impl fmt::Display for {secret}"))
                && !text.contains(&format!("impl std::fmt::Display for {secret}")),
            "{secret} can be written into a sentence"
        );
        assert!(
            text.contains(&format!("impl fmt::Debug for {secret}")),
            "{secret} has no hand-written Debug, so it has a derived one"
        );
        let declared = format!("pub struct {secret} {{");
        let before = text
            .split(&declared)
            .next()
            .map(|before| before.rsplit("\n\n").next().unwrap_or_default().to_owned())
            .unwrap_or_default();
        assert!(
            !before.contains("#[derive("),
            "{file} derives something onto {secret}, and what it derives is a line with the \
             secret in it"
        );
    }
}
