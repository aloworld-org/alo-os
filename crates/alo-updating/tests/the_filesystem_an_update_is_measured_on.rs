//! The two virtual machines this crate measures an update on are installed
//! onto the filesystem alo OS ships, and onto no other.
//!
//! `tests/an_update_keeps_the_persons_things.rs` and
//! `tests/back_to_yesterdays_machine.rs` each stand a real machine up with the
//! base's own installer, and until 2026-09-21 each spelt `ext4` in place.
//! [`alo_image::THE_ONLY_FILESYSTEM`] is `btrfs` and has been since the
//! filesystem became the whole of whether a machine can ever undo anything
//! (ADR 0045), so what those two proved about an update keeping a person's
//! things was proved about a machine nobody will own.
//!
//! **This is the half that survives the next decision.** Changing the two
//! spellings fixes today; reading them back through the guard `alo-image`
//! already ships is what carries them the next time the value moves, without
//! anybody having to remember these two files exist.
//!
//! Unlike the two tests it reads, this one needs no machine, no root and no
//! network: it reads their source.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_image::{THE_ONLY_FILESYSTEM, TheFilesystem};

/// The two tests that install a machine of their own.
const THEY_INSTALL_A_MACHINE: [&str; 2] = [
    "an_update_keeps_the_persons_things.rs",
    "back_to_yesterdays_machine.rs",
];

/// **Each of them names the one filesystem, and names it by the constant.**
///
/// Named by the constant rather than spelt, which is what
/// [`alo_image::THE_ONLY_FILESYSTEM`]'s own header asks of every writer: a value
/// that cannot be taken back after an install is one this repository keeps in a
/// single place.
#[test]
fn every_machine_this_crate_stands_up_is_installed_onto_the_one_filesystem() {
    for named in THEY_INSTALL_A_MACHINE {
        let text = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests")
                .join(named),
        )
        .expect("a test in this crate");
        let read = TheFilesystem::read(&text);

        assert!(
            read.names_one(),
            "{named} installs a machine and names no filesystem"
        );
        assert_eq!(
            read.anything_else(),
            Vec::<&str>::new(),
            "{named} installs onto a filesystem alo OS does not ship"
        );
        assert!(
            !text.contains(&format!("\"{THE_ONLY_FILESYSTEM}\"")),
            "{named} spells the filesystem where it could name the constant"
        );
    }
}
