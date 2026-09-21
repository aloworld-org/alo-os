//! One filesystem is named on the road to a person's disk, and it is the one a
//! snapshot can be taken on.
//!
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md) rewinds
//! *undo what the agent did* from the base's own read-only snapshot, and **a
//! filesystem is chosen at install and cannot be converted afterwards**. So the
//! argument `bootc install to-disk --filesystem` is given is not a preference:
//! a machine that got the wrong one can only be given an undo by being
//! reinstalled, which on somebody's only computer is not a fix.
//!
//! The value lives in one place, [`alo_image::THE_ONLY_FILESYSTEM`], and every
//! writer takes it from there. What this test adds is the thing a shared
//! constant cannot do on its own: it counts. A **second** `--filesystem`
//! appearing later — added beside the first for a case somebody had — would
//! install some machines onto a filesystem with no snapshots, and there would
//! be nobody to notice. Here it is a failing test.
//!
//! # The two places, and why only these two
//!
//! `crates/alo-installing/src/writing.rs` is what the boot environment runs on
//! a person's real disk. `docs/booting.md` is what a person types to make a
//! development disk, and a document that drifts from the program is how a lane
//! spends an afternoon measuring a filesystem the product does not install.
//!
//! Nothing else in the tree is read, on purpose. A test elsewhere that installs
//! a scratch disk on `ext4` to measure something about `ext4` is measuring
//! `ext4` deliberately, and a guard that swept the whole repository would call
//! that a defect and be wrong.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_image::{THE_ONLY_FILESYSTEM, TheDocument, TheFilesystem};

/// The installer program that writes a person's real disk.
const THE_INSTALLERS_WRITE: &str = "crates/alo-installing/src/writing.rs";

/// The document a person follows to make a disk by hand.
const THE_DOCUMENT: &str = "docs/booting.md";

/// Every file that decides what filesystem a machine alo OS installs gets.
const EVERY_PLACE: [&str; 2] = [THE_INSTALLERS_WRITE, THE_DOCUMENT];

/// The repository's root, from this crate's own directory — a test's working
/// directory is the package root, which is not something to rely on.
fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// **Each place that names a filesystem names one, and it is the one an undo
/// can be taken on.** A second one anywhere on this road is the defect.
#[test]
fn every_place_that_installs_a_machine_names_one_filesystem_and_it_is_btrfs() {
    for place in EVERY_PLACE {
        let text = std::fs::read_to_string(repository().join(place))
            .unwrap_or_else(|why| panic!("{place} could not be read: {why}"));
        // The document is read as the commands a person types, not as prose:
        // it quotes the base's own `Probing bootupd --filesystem support`
        // while explaining a failure, and a sentence about an argument is not
        // an argument.
        let read = if place == THE_DOCUMENT {
            TheFilesystem::read(&TheDocument::read(&text).commands().join("\n"))
        } else {
            TheFilesystem::read(&text)
        };

        assert!(
            read.names_one(),
            "{place} no longer names a filesystem at all; a `bootc install to-disk` with no \
             `--filesystem` takes whatever the base's default is, which nobody here chose"
        );
        assert!(
            read.anything_else().is_empty(),
            "{place} names {:?}, and the only filesystem a machine alo OS installs may get is \
             `{THE_ONLY_FILESYSTEM}` — ADR 0045, and a filesystem cannot be converted after an \
             install. If a second one is genuinely needed, that is a decision and not an argument.",
            read.anything_else()
        );
    }
}

/// **The one filesystem is one that has snapshots at all**, which is the whole
/// reason for the value.
///
/// It reads as tautological until the day somebody changes the constant to make
/// a build pass: `xfs` and `ext4` are the other two the pinned `bootc` accepts,
/// and neither has a subvolume or a snapshot (ADR 0045, measured). A machine
/// installed on either can never undo what an agent did.
#[test]
fn the_one_filesystem_is_not_one_without_snapshots() {
    for without_snapshots in ["ext4", "xfs", "vfat"] {
        assert_ne!(
            THE_ONLY_FILESYSTEM, without_snapshots,
            "`{without_snapshots}` has no subvolume and no snapshot, so a machine installed onto \
             it can never undo what an agent did (ADR 0045)"
        );
    }
}
