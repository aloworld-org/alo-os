//! **The walk machine's disk is encrypted, the way an installed one is.**
//!
//! ADR 0054 encrypts the root filesystem, and on 2026-11-21 the owner said an
//! install is encrypted unless the person turns it off. The walk machine is the
//! one alo OS that is booted on every merge. If its disk is plain, every walk
//! passes on a machine that is not the product, and the unlock at boot, which is
//! the first thing a person meets, is never tried. So this reads
//! `.github/workflows/walk.yml` the way `the_walk_workflow_publishes_nothing.rs`
//! does: by its lines that are not comments.
//!
//! **What this cannot show**, and the walk itself does: that the disk opens at
//! boot. This holds the workflow to asking for an encrypted disk. A machine that
//! then cannot open it fails the walk, which is the check that can.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on an unreadable workflow is the failure being reported"
)]

/// The workflow, from this repository.
const THE_WORKFLOW: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../.github/workflows/walk.yml"
);

/// The workflow's lines that are not comments, trimmed.
fn said() -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(THE_WORKFLOW) else {
        panic!("{THE_WORKFLOW} could not be read");
    };
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

/// Whether any line says this.
fn says(lines: &[String], words: &str) -> bool {
    lines.iter().any(|line| line.contains(words))
}

/// **The root is made LUKS2 before anything is written to it**, and the
/// install is handed the opened volume rather than the disk. `to-disk` makes
/// its own table and a plain root, which is what this replaces.
#[test]
fn the_root_is_luks2_and_the_install_is_handed_the_opened_volume() {
    let lines = said();
    assert!(
        says(&lines, "cryptsetup luksFormat --type luks2"),
        "no LUKS2 root is made"
    );
    assert!(
        says(&lines, "bootc install to-filesystem"),
        "the install is not handed a root that was opened"
    );
    assert!(
        !says(&lines, "bootc install to-disk"),
        "the disk is still written plain by to-disk"
    );
}

/// **A recovery key is enrolled, as on every install** (ADR 0054: no road
/// enrols encryption without one), and the initramfs is told which volume to
/// open.
#[test]
fn a_recovery_key_is_enrolled_and_the_boot_is_told_what_to_open() {
    let lines = said();
    assert!(
        says(&lines, "systemd-cryptenroll --recovery-key"),
        "no recovery key is enrolled"
    );
    assert!(
        says(&lines, "--karg rd.luks.uuid="),
        "the boot is not told which volume to open"
    );
}

/// **The walk is given the secret to type at boot**, by the one name
/// `alo-walking` reads, and the disk is made with that same secret.
#[test]
fn the_walk_is_given_the_secret_the_disk_was_made_with() {
    let lines = said();
    assert!(
        says(&lines, "ALO_WALK_DISK_PASSPHRASE:"),
        "the walk is not told the disk's secret"
    );
    let made_with_it = lines
        .iter()
        .any(|line| line.contains("luksFormat") && line.contains("ALO_WALK_DISK_PASSPHRASE"));
    assert!(
        made_with_it,
        "the disk is not made with the secret the walk types"
    );
}
