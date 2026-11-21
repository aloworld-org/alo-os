//! What opening the disk does, read without a machine.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on a missing step is the failure being reported"
)]

use super::{BEFORE_THE_PROMPT, THE_DISK_PASSPHRASE, the_disk_is_opened};
use crate::step::{Since, Step, Walk};
use crate::the_first_walk;

/// **The disk is opened in this order**: the initramfs is seen to start, the
/// walk waits, types the passphrase without saying it, presses Enter, and
/// sees the root mount on the opened device — each line read from the boot.
#[test]
fn the_passphrase_is_typed_after_the_initramfs_and_the_root_mounts_after_it() {
    let walk = the_disk_is_opened(Walk::named("opening"));
    let steps: Vec<&Step> = walk.steps().iter().skip(1).collect();
    match steps.as_slice() {
        [
            Step::ExpectLine {
                phrase: started,
                since: Since::Boot,
                ..
            },
            Step::Wait(waited),
            Step::TypeSecret(variable),
            Step::Press(enter),
            Step::ExpectLine {
                phrase: mounted,
                since: Since::Boot,
                ..
            },
        ] => {
            assert_eq!(started, "Run /init");
            assert_eq!(*waited, BEFORE_THE_PROMPT);
            assert_eq!(*variable, THE_DISK_PASSPHRASE);
            assert_eq!(enter.the_key(), "ret");
            assert_eq!(mounted, "BTRFS info (device dm-");
        }
        other => panic!("opening the disk is {other:?}"),
    }
}

/// **Nothing of it runs without the passphrase, and only it is gated**: the
/// first step is a gate on the variable covering every step after it, so a
/// disk written without encryption boots as before.
#[test]
fn it_runs_only_with_the_passphrase_and_gates_only_itself() {
    let walk = the_disk_is_opened(Walk::named("opening"));
    let steps = walk.steps();
    match steps.first() {
        Some(Step::OnlyWithFor {
            variable,
            steps: covered,
        }) => {
            assert_eq!(*variable, THE_DISK_PASSPHRASE);
            assert_eq!(*covered, steps.len() - 1);
        }
        other => panic!("the first step is {other:?}"),
    }
}

/// **The first walk opens the disk before it waits for the sign-in screen**,
/// since that screen is drawn only after the root is mounted.
#[test]
fn the_first_walk_opens_the_disk_first() {
    let walk = the_first_walk::the_first_walk();
    assert!(
        matches!(walk.steps().first(),
            Some(Step::OnlyWithFor { variable, .. }) if *variable == THE_DISK_PASSPHRASE),
        "the first walk's first step is {:?}",
        walk.steps().first()
    );
}
