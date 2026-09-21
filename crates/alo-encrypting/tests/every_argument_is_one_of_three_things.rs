//! **No argument of any run is a free string**, which is what
//! [ADR 0056](../../../docs/decisions/0056-a-sealed-disks-promise-is-shown-on-a-machine-with-a-chip.md)
//! asks of the sequence and what makes the difference between a description of
//! two rented tools and a road from something somebody typed into the argument
//! list of a program that runs as root.
//!
//! It is held three ways, because any one of them alone would be a green test
//! that proves the wrong thing:
//!
//! - **every argument of every run of every sequence** is a flag written out in
//!   `src/sequence.rs`, the volume's own path, or one of the four named
//!   secrets' paths — walked rather than argued;
//! - **nothing that builds a sequence takes text**, read off the crate's own
//!   public surface, so a road from typed text to an argument does not exist to
//!   be walked;
//! - **no argument carries a shell, a space or a way out of a directory**, so
//!   that a tool started without one is the same tool as a tool started with
//!   one.
//!
//! The first of the three is the one that would catch an argument somebody
//! added; the second is the one that would catch a constructor somebody added.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_encrypting::{
    ASecretOnItsWay, IT_OPENS_AS, Run, TheDisk, TheSequence, TheVolume, WhatToAskFor,
};

/// Every flag and sub-command `src/sequence.rs` writes out, and there is no
/// other. A flag added there and not here fails this test, which is where the
/// argument for adding it is made.
const EVERY_FLAG_THIS_CRATE_WRITES: [&str; 14] = [
    "luksFormat",
    "luksAddKey",
    "luksRemoveKey",
    "open",
    "close",
    "--type",
    "luks2",
    "--batch-mode",
    "--recovery-key",
    "--tpm2-device=auto",
    "--tpm2-with-pin=yes",
    "--tpm2-pcrs=7",
    "--unlock-tpm2-device=auto",
    "--wipe-slot=tpm2",
];

/// The volume every sequence here is about.
fn a_volume() -> TheVolume {
    let Ok(disk) = TheDisk::named("virtio-alo-target") else {
        panic!("that is a disk's own name")
    };
    match TheVolume::the_partition_of(&disk, 4) {
        Ok(volume) => volume,
        Err(why) => panic!("4 is a partition: {why}"),
    }
}

/// Every sequence this crate can build, so that none of them is checked by
/// having been forgotten.
fn every_sequence() -> Vec<TheSequence> {
    let volume = a_volume();
    let mut all = vec![
        TheSequence::enrolling_at_install(&volume, WhatToAskFor::APassphrase),
        TheSequence::enrolling_at_install(&volume, WhatToAskFor::APin),
        TheSequence::closing(),
        TheSequence::changing_what_the_person_unlocks_with(&volume, WhatToAskFor::APassphrase),
        TheSequence::changing_what_the_person_unlocks_with(&volume, WhatToAskFor::APin),
    ];
    for secret in ASecretOnItsWay::ALL {
        all.push(TheSequence::opening(&volume, secret));
    }
    all
}

/// Whether one argument is one of the three things an argument may be.
fn one_of_the_three(argument: &str, volume: &TheVolume) -> bool {
    if EVERY_FLAG_THIS_CRATE_WRITES.contains(&argument) || argument == IT_OPENS_AS {
        return true;
    }
    if argument == volume.where_it_is() {
        return true;
    }
    match argument.split_once('=') {
        Some((flag, names)) => {
            ["--key-file", "--new-keyfile", "--unlock-key-file"].contains(&flag)
                && ASecretOnItsWay::ALL
                    .iter()
                    .any(|secret| secret.where_it_is() == names)
        }
        None => false,
    }
}

/// **Every argument of every run of every sequence is one of the three.**
#[test]
fn every_argument_is_a_flag_the_volume_or_a_named_secret() {
    let volume = a_volume();
    let mut seen = 0;
    for sequence in every_sequence() {
        for run in sequence.runs() {
            assert!(!run.arguments().is_empty(), "a run with no arguments");
            for argument in run.arguments() {
                seen += 1;
                assert!(
                    one_of_the_three(argument, &volume),
                    "{argument} is none of: a flag this crate writes, the volume, a named secret"
                );
            }
        }
    }
    assert!(seen > 40, "only {seen} arguments were walked");
}

/// **Nothing that builds a sequence takes text.** Read off the public surface,
/// because a test that walks the sequences cannot say there is not a fourth
/// constructor three lines further down that takes a `String`.
#[test]
fn nothing_that_builds_a_run_takes_text() {
    let source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("sequence.rs"),
    )
    .unwrap_or_default();
    assert!(!source.is_empty(), "src/sequence.rs could not be read");
    let public: Vec<&str> = source
        .lines()
        .map(str::trim)
        .take_while(|line| *line != "#[cfg(test)]")
        .filter(|line| line.starts_with("pub fn ") || line.starts_with("pub const fn "))
        .collect();
    assert!(!public.is_empty(), "no public function was found");
    for line in public {
        let Some(arguments) = line
            .split_once('(')
            .and_then(|(_, rest)| rest.split_once(')'))
        else {
            panic!("{line} is not a function this test can read")
        };
        for takes in ["&str", "String", "Path", "OsStr", "u8"] {
            assert!(
                !arguments.0.contains(takes),
                "{line} takes {takes}, and an argument built from one is a free string"
            );
        }
    }
}

/// **No argument carries a shell, a space, or a way out of a directory.**
#[test]
fn no_argument_carries_a_shell_or_a_way_out_of_a_directory() {
    for sequence in every_sequence() {
        for argument in sequence.runs().iter().flat_map(Run::arguments) {
            for never in [
                ' ', '\t', '\n', '\r', ';', '|', '&', '$', '`', '>', '<', '*', '?', '\'', '"',
            ] {
                assert!(
                    !argument.contains(never),
                    "{argument} carries {never:?}, which is a shell where an argument was meant"
                );
            }
            assert!(!argument.contains(".."), "{argument} leaves its directory");
            assert!(argument.is_ascii(), "{argument} is not what udev writes");
        }
    }
}

/// **Every run that needs a secret names which of the four it needs**, so that
/// a caller writes one file rather than guessing, and every run that names one
/// also names it in an argument.
#[test]
fn a_run_that_needs_a_secret_names_which_one() {
    for sequence in every_sequence() {
        for run in sequence.runs() {
            let Some(secret) = run.given() else { continue };
            assert!(
                run.arguments()
                    .iter()
                    .any(|argument| argument.ends_with(secret.where_it_is())),
                "{run:?} says it needs {secret} and never names its file"
            );
        }
    }
}
