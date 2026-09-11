//! What the image's recipe says about the disk a machine boots from.
//!
//! An image is not a disk. `image/Containerfile` builds a bootable container
//! ([ADR 0011](../../../docs/decisions/0011-the-base-is-rented-and-the-image-is-a-container.md)),
//! and something has to write that container onto a partitioned disk before any
//! machine can be pointed at it. The three `alo.disk.*` labels in the recipe are
//! where the image says which tool does that, which firmware it is installed
//! for, and what the resulting file is called.
//!
//! # Why the declaration is in the recipe rather than beside it
//!
//! A second file would be a second recipe, and a divergence between them is the
//! defect this module exists to prevent: an image built one way and a disk
//! declared another. Labels also travel — they are on the image itself, so
//! somebody holding only the image can ask it what disk it expects, and the two
//! cannot be separated far enough to disagree.
//!
//! # The tool is not pinned here, because it is already pinned
//!
//! `bootc install to-disk` is run **out of the base image**, so the version of
//! the tool is `THE_BASE`'s digest in the same file. That is why
//! [`TheDisk::written_by_the_base`] asks whether the base is pinned by digest at
//! all: an unpinned base is an unpinned partitioner, and
//! [ADR 0011](../../../docs/decisions/0011-the-base-is-rented-and-the-image-is-a-container.md)
//! rents the base rather than trusting whatever `:42` meant this morning.
//!
//! # Read leniently, judged strictly
//!
//! [`TheDisk::read`] never refuses, which is `crate::runtime`'s shape: a recipe
//! that declares no disk reads as one that declares none, and `crate::checking`
//! turns each absence into a [`Wrong`](crate::Wrong) with the decision it
//! breaks. A reader that refused the wrong states could never report them.

/// The label naming the tool that writes the disk.
pub const THE_TOOL: &str = "alo.disk.tool";

/// The label naming the firmware the disk is installed for.
pub const THE_FIRMWARE: &str = "alo.disk.firmware";

/// The label naming the file the disk is written to.
pub const THE_FILE: &str = "alo.disk.file";

/// The only tool this repository writes a disk with: the base image's own,
/// installing the container it is running as.
///
/// Named here rather than only in the recipe because the whole of the promise
/// is that it is *this* and not something somebody assembled — a check that
/// accepted whatever the label said would accept `sfdisk`.
pub const THE_ONLY_TOOL: &str = "bootc install to-disk";

/// Every way of laying out a disk by hand, which is the thing this repository
/// does not do.
///
/// A partitioner in the recipe or in `docs/booting.md` is somebody who found
/// the upstream tool inconvenient, and an alo OS disk assembled here is a disk
/// whose layout nobody upstream ever tested — on the one part of the system
/// whose mistakes only ever appear on somebody else's machine.
pub const NO_PARTITIONER: [&str; 5] = ["sfdisk", "fdisk", "parted", "mkfs", "dd if="];

/// The build argument that names the base image.
const THE_BASE_ARG: &str = "ARG THE_BASE=";

/// What a base pinned by content looks like.
const BY_DIGEST: &str = "@sha256:";

/// How many hexadecimal characters a sha256 digest has.
const A_WHOLE_DIGEST: usize = 64;

/// What the image's recipe says about the disk it becomes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheDisk {
    /// The tool the recipe names, or [`None`] where it names none.
    tool: Option<String>,
    /// The firmware the recipe names, or [`None`] where it names none.
    firmware: Option<String>,
    /// The file the recipe names, or [`None`] where it names none.
    file: Option<String>,
    /// Whether the base the tool comes out of is pinned by content.
    base_is_pinned: bool,
    /// The first partitioner the recipe names, where it names one.
    by_hand: Option<String>,
}

impl TheDisk {
    /// What this Containerfile says about the disk, read off its text.
    #[must_use]
    pub fn read(containerfile: &str) -> Self {
        let mut tool = None;
        let mut firmware = None;
        let mut file = None;
        let mut base_is_pinned = false;
        let mut by_hand = None;

        for line in containerfile.lines() {
            let line = line.trim();
            if line.starts_with('#') {
                continue;
            }
            if by_hand.is_none() {
                by_hand = NO_PARTITIONER
                    .into_iter()
                    .find(|it| line.contains(it))
                    .map(ToOwned::to_owned);
            }
            if let Some(base) = line.strip_prefix(THE_BASE_ARG) {
                base_is_pinned = pinned_by_content(base.trim());
            }
            if let Some((label, said)) = labelled(line) {
                match label {
                    THE_TOOL => tool = Some(said),
                    THE_FIRMWARE => firmware = Some(said),
                    THE_FILE => file = Some(said),
                    _ => {}
                }
            }
        }

        Self {
            tool,
            firmware,
            file,
            base_is_pinned,
            by_hand,
        }
    }

    /// The tool the recipe names, or [`None`] where it names none.
    #[must_use]
    pub fn tool(&self) -> Option<&str> {
        self.tool.as_deref()
    }

    /// The firmware the recipe names, or [`None`] where it names none.
    #[must_use]
    pub fn firmware(&self) -> Option<&str> {
        self.firmware.as_deref()
    }

    /// The file the recipe names, or [`None`] where it names none.
    #[must_use]
    pub fn file(&self) -> Option<&str> {
        self.file.as_deref()
    }

    /// Whether the disk is written by the base image's own tool, pinned by the
    /// digest that pins the base.
    ///
    /// Both halves are the same promise: the tool has to be `bootc install
    /// to-disk`, and the base it is run out of has to be a base somebody chose
    /// rather than a tag that moved.
    #[must_use]
    pub fn written_by_the_base(&self) -> bool {
        self.tool() == Some(THE_ONLY_TOOL) && self.base_is_pinned
    }

    /// Whether the base the tool comes out of is pinned by content.
    #[must_use]
    pub const fn on_a_pinned_base(&self) -> bool {
        self.base_is_pinned
    }

    /// The partitioner this recipe names, where it names one.
    #[must_use]
    pub fn laid_out_by_hand(&self) -> Option<&str> {
        self.by_hand.as_deref()
    }
}

/// The label and the value on this line, where it is one.
fn labelled(line: &str) -> Option<(&str, String)> {
    let said = line.strip_prefix("LABEL ")?;
    let (label, value) = said.split_once('=')?;
    Some((label.trim(), value.trim().trim_matches('"').to_owned()))
}

/// Whether a base image is named by content rather than by a tag that moves.
fn pinned_by_content(base: &str) -> bool {
    let Some((_, digest)) = base.split_once(BY_DIGEST) else {
        return false;
    };
    digest.len() == A_WHOLE_DIGEST && digest.bytes().all(|b| b.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A recipe holding exactly the lines a test names.
    fn saying(lines: &[&str]) -> TheDisk {
        TheDisk::read(&lines.join("\n"))
    }

    /// A base pinned the way this repository pins it.
    fn a_pinned_base() -> String {
        format!(
            "ARG THE_BASE=quay.io/fedora/fedora-bootc:42@sha256:{}",
            "0f".repeat(32)
        )
    }

    /// **The whole declaration reads off a recipe that carries it**, so the
    /// refusals below are about what is missing rather than about a reader that
    /// never worked.
    #[test]
    fn a_recipe_that_declares_its_disk_reads_as_one() {
        let read = saying(&[
            &a_pinned_base(),
            "LABEL alo.disk.tool=\"bootc install to-disk\"",
            "LABEL alo.disk.firmware=\"uefi\"",
            "LABEL alo.disk.file=\"alo-os.raw\"",
        ]);

        assert_eq!(read.tool(), Some(THE_ONLY_TOOL));
        assert_eq!(read.firmware(), Some("uefi"));
        assert_eq!(read.file(), Some("alo-os.raw"));
        assert!(read.written_by_the_base());
    }

    /// **A recipe that says nothing about a disk reads as one that declares
    /// none**, rather than as a refusal — the wrong states are the ones the
    /// checker has to be able to see.
    #[test]
    fn a_recipe_with_no_disk_in_it_declares_nothing() {
        let read = saying(&["FROM somewhere", "COPY image/etc/alo/ /etc/alo/"]);

        assert_eq!(read.tool(), None);
        assert_eq!(read.firmware(), None);
        assert_eq!(read.file(), None);
        assert!(!read.written_by_the_base());
    }

    /// **A tool that is not the base's own is not the base's own.** The check
    /// names the tool rather than accepting whatever the label says, because a
    /// label that read `sfdisk` would otherwise pass the promise that no disk
    /// here is laid out by hand.
    #[test]
    fn a_disk_written_by_something_else_is_not_written_by_the_base() {
        let read = saying(&[
            &a_pinned_base(),
            "LABEL alo.disk.tool=\"sfdisk --wipe always\"",
        ]);

        assert!(!read.written_by_the_base());
    }

    /// **A base on a tag is an unpinned partitioner.** The tool comes out of the
    /// base, so a base that moved is a disk written by a tool nobody chose.
    #[test]
    fn a_base_on_a_moving_tag_is_not_a_pinned_tool() {
        for moving in [
            "quay.io/fedora/fedora-bootc:42",
            "quay.io/fedora/fedora-bootc:latest",
            "quay.io/fedora/fedora-bootc:42@sha256:0f0f",
            "quay.io/fedora/fedora-bootc:42@sha256:zz",
            "",
        ] {
            let read = saying(&[
                &format!("ARG THE_BASE={moving}"),
                "LABEL alo.disk.tool=\"bootc install to-disk\"",
            ]);
            assert!(
                !read.written_by_the_base(),
                "`{moving}` was read as a pinned base"
            );
        }
    }

    /// **A partitioner in the recipe is found and named.** It is the shape the
    /// mistake arrives in: not a second recipe, one `RUN` added to this one to
    /// make a layout come out the way somebody wanted.
    #[test]
    fn a_recipe_that_lays_out_a_disk_itself_is_read_as_doing_so() {
        let read = saying(&[
            &a_pinned_base(),
            "RUN parted --script /dev/loop0 mklabel gpt",
        ]);
        assert_eq!(read.laid_out_by_hand(), Some("parted"));

        let clean = saying(&[&a_pinned_base(), "LABEL alo.disk.firmware=\"uefi\""]);
        assert_eq!(clean.laid_out_by_hand(), None);
    }

    /// **A commented-out declaration declares nothing.** It is the shape a
    /// half-finished edit really leaves behind.
    #[test]
    fn a_commented_label_declares_nothing() {
        let read = saying(&["# LABEL alo.disk.firmware=\"uefi\""]);

        assert_eq!(read.firmware(), None);
    }
}
