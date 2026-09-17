//! The way from the person's service to the boundary the loader pinned.
//!
//! ADR 0018 splits the boundary in two: `alo-boundaryd`, root at boot, loads the
//! programme and pins it beneath `/sys/fs/bpf/alo`, handing the map of turns to
//! the agent's group; `alo-agentd`, the person holding nothing, opens that map.
//! Every file of that split was checked here, and none of them was the reason
//! the first disk installed under Secure Boot came up with `alo-agentd` failed:
//! **systemd mounts the BPF file system 0700 root:root**, so the directory the
//! loader gave the agent's group sat behind one only root could pass through,
//! and the daemon said there was no boundary while one was pinned
//! (`docs/quirks.md`, *systemd mounts the BPF filesystem so only root can pass
//! through it*).
//!
//! So two promises, one [`Wrong`] each when broken:
//!
//! - **the loader waits for the file system** it pins in, which is where the
//!   mount point is named at all;
//! - **the image lets the agent's group through that mount point and does
//!   nothing else** — a `tmpfiles.d` adjustment to exactly
//!   [`THE_PASSAGES_MODE`], owned by root, in the group the loader runs in. Not
//!   a directory made there, which would stand in for a mount; not read, which
//!   would let the group list every pin on the machine; not write; and nobody
//!   outside the group.

use std::path::Path;

use crate::image::Image;
use crate::service::ROOT;
use crate::wrong::Wrong;

/// Where systemd mounts the BPF file system, which the boundary is pinned in.
pub const THE_BPF_FILESYSTEM: &str = "/sys/fs/bpf";

/// The mode the way through it is given: everything to root, search and nothing
/// more to the agent's group, nothing to anybody else.
pub const THE_PASSAGES_MODE: u32 = 0o710;

/// What a unit with no group says, in a sentence.
const NOTHING: &str = "-";

/// Everything wrong with the way from the agent's service to the boundary.
pub(crate) fn everything_wrong_with_the_way_to_the_boundary(image: &Image, wrong: &mut Vec<Wrong>) {
    the_loader_waits_for_its_file_system(image, wrong);
    the_agents_group_may_pass_and_nothing_more(image, wrong);
}

/// **The loader waits for the file system it pins the boundary in.**
fn the_loader_waits_for_its_file_system(image: &Image, wrong: &mut Vec<Wrong>) {
    if !image
        .loader()
        .waits_for_mounts()
        .contains(&THE_BPF_FILESYSTEM)
    {
        wrong.push(Wrong::TheLoaderDoesNotWaitForItsFileSystem {
            loader: image.loader().called().to_owned(),
            mount: THE_BPF_FILESYSTEM.to_owned(),
        });
    }
}

/// **The agent's group may pass through the BPF file system's root, and do
/// nothing else there**, and nobody else may pass at all.
fn the_agents_group_may_pass_and_nothing_more(image: &Image, wrong: &mut Vec<Wrong>) {
    let at = Path::new(THE_BPF_FILESYSTEM);
    let agents_group = image.loader().in_group().unwrap_or(NOTHING);
    match image.adjusted_at(at) {
        None => wrong.push(Wrong::TheBoundaryIsOutOfTheAgentsReach {
            at: at.to_owned(),
            group: agents_group.to_owned(),
        }),
        Some(passage) => {
            if passage.mode() != THE_PASSAGES_MODE
                || passage.owner() != ROOT
                || passage.group() != agents_group
            {
                wrong.push(Wrong::TheWayToTheBoundaryIsNotWhatWasDecided {
                    at: at.to_owned(),
                    mode: passage.mode(),
                    owner: passage.owner().to_owned(),
                    group: passage.group().to_owned(),
                    expected_mode: THE_PASSAGES_MODE,
                    agents_group: agents_group.to_owned(),
                });
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::checking::everything_wrong_with;
    use crate::testing::{THE_LOADERS_UNIT, THE_TMPFILES, a_copy_of_the_image, edited, image_at};

    /// The line the image ships.
    const THE_PASSAGE: &str = "z /sys/fs/bpf 0710 root alo-agent -";

    /// What is wrong with the way to the boundary in a copy of the image with
    /// one file edited.
    fn wrong_after(what: &str, file: &str, from: &str, to: &str) -> Vec<Wrong> {
        let root = a_copy_of_the_image(what);
        edited(&root, file, from, to);
        let image = image_at(&root);
        let mut wrong = Vec::new();
        everything_wrong_with_the_way_to_the_boundary(&image, &mut wrong);
        drop(std::fs::remove_dir_all(root.parent().unwrap_or(&root)));
        wrong
    }

    /// **The image this repository ships lets the agent's group through, and
    /// only it** — the line booted on 2026-09-16, after which `alo-agentd` ran
    /// under Secure Boot — and the whole image still says one thing with it.
    #[test]
    fn the_image_lets_the_agents_group_reach_the_boundary() {
        let root = a_copy_of_the_image("the-way-to-the-boundary");
        let image = image_at(&root);
        let mut wrong = Vec::new();
        everything_wrong_with_the_way_to_the_boundary(&image, &mut wrong);
        assert!(wrong.is_empty(), "{wrong:?}");
        let passage = image
            .adjusted_at(Path::new(THE_BPF_FILESYSTEM))
            .expect("the image adjusts the BPF file system's root");
        assert_eq!(passage.mode(), THE_PASSAGES_MODE);
        assert_eq!(
            passage.group(),
            image.loader().in_group().unwrap_or(NOTHING)
        );
        assert!(
            everything_wrong_with(&image).is_empty(),
            "{:?}",
            everything_wrong_with(&image)
        );
        drop(std::fs::remove_dir_all(root.parent().unwrap_or(&root)));
    }

    /// **An image that leaves the BPF file system as systemd mounts it is
    /// caught** — which is exactly the image of release 0.0.1, whose installed
    /// disk booted with the boundary pinned and `alo-agentd` failed.
    #[test]
    fn a_boundary_behind_roots_door_is_caught() {
        let wrong = wrong_after("no-passage", THE_TMPFILES, THE_PASSAGE, "");
        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheBoundaryIsOutOfTheAgentsReach { at, group }
                    if at == Path::new(THE_BPF_FILESYSTEM) && group == "alo-agent"
            )),
            "{wrong:?}"
        );
    }

    /// **A directory made there is not a way through.** `d` would make a
    /// directory where a mount belongs on a machine that had none, and it is not
    /// the line that adjusts the mount.
    #[test]
    fn a_directory_made_where_the_mount_belongs_is_caught() {
        let wrong = wrong_after(
            "made-not-adjusted",
            THE_TMPFILES,
            THE_PASSAGE,
            "d /sys/fs/bpf 0710 root alo-agent -",
        );
        assert!(
            wrong
                .iter()
                .any(|it| matches!(it, Wrong::TheBoundaryIsOutOfTheAgentsReach { .. })),
            "{wrong:?}"
        );
    }

    /// **Every way of opening it wider is caught**: anybody may pass, the group
    /// may list every pin, the group may write, and the sticky world-writable
    /// root the kernel would otherwise leave.
    #[test]
    fn a_way_through_wider_than_passing_is_caught() {
        for (named, mode) in [
            ("anybody-passes", 0o711),
            ("group-lists", 0o750),
            ("group-writes", 0o730),
            ("world-writable", 0o1777),
        ] {
            let wrong = wrong_after(
                named,
                THE_TMPFILES,
                THE_PASSAGE,
                &format!("z /sys/fs/bpf {mode:04o} root alo-agent -"),
            );
            assert!(
                wrong.iter().any(|it| matches!(
                    it,
                    Wrong::TheWayToTheBoundaryIsNotWhatWasDecided { mode: said, .. }
                        if *said == mode
                )),
                "{mode:04o}: {wrong:?}"
            );
        }
    }

    /// **A way through given to anybody but the agent's group, or owned by
    /// anybody but root, is caught** — the person's own group, and the agent's
    /// group made the owner, which could then change the mode itself.
    #[test]
    fn a_way_through_given_to_somebody_else_is_caught() {
        for (named, line) in [
            ("the-persons-group", "z /sys/fs/bpf 0710 root alo -"),
            (
                "owned-by-the-agent",
                "z /sys/fs/bpf 0710 alo-agent alo-agent -",
            ),
        ] {
            let wrong = wrong_after(named, THE_TMPFILES, THE_PASSAGE, line);
            assert!(
                wrong
                    .iter()
                    .any(|it| matches!(it, Wrong::TheWayToTheBoundaryIsNotWhatWasDecided { .. })),
                "{line}: {wrong:?}"
            );
        }
    }

    /// **The way through follows the group the loader hands the boundary to**,
    /// so a loader moved to another group without this line moving with it is
    /// caught rather than left with a passage for a group that holds nothing.
    #[test]
    fn a_loader_moved_to_another_group_is_caught() {
        let wrong = wrong_after(
            "loader-moved",
            THE_LOADERS_UNIT,
            "Group=alo-agent",
            "Group=alo-greeter",
        );
        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheWayToTheBoundaryIsNotWhatWasDecided { agents_group, .. }
                    if agents_group == "alo-greeter"
            )),
            "{wrong:?}"
        );
    }

    /// **A loader that does not wait for the file system is caught.**
    #[test]
    fn a_loader_that_does_not_wait_for_its_file_system_is_caught() {
        let wrong = wrong_after(
            "loader-does-not-wait",
            THE_LOADERS_UNIT,
            "RequiresMountsFor=/sys/fs/bpf",
            "",
        );
        assert!(
            wrong.iter().any(|it| matches!(
                it,
                Wrong::TheLoaderDoesNotWaitForItsFileSystem { mount, .. }
                    if mount == THE_BPF_FILESYSTEM
            )),
            "{wrong:?}"
        );
    }

    /// **The sentences say what to change and why**: the mode in octal, the
    /// group, and the decision they answer to.
    #[test]
    fn the_sentences_name_the_mode_the_group_and_the_decision() {
        let said = Wrong::TheWayToTheBoundaryIsNotWhatWasDecided {
            at: Path::new(THE_BPF_FILESYSTEM).to_owned(),
            mode: 0o711,
            owner: ROOT.to_owned(),
            group: "alo-agent".to_owned(),
            expected_mode: THE_PASSAGES_MODE,
            agents_group: "alo-agent".to_owned(),
        }
        .to_string();
        assert!(
            said.contains("0711") && said.contains("0710 root:alo-agent"),
            "{said}"
        );
        assert!(said.contains("ADR 0018"), "{said}");
        let said = Wrong::TheBoundaryIsOutOfTheAgentsReach {
            at: Path::new(THE_BPF_FILESYSTEM).to_owned(),
            group: "alo-agent".to_owned(),
        }
        .to_string();
        assert!(
            said.contains("/sys/fs/bpf") && said.contains("alo-agent"),
            "{said}"
        );
    }
}
