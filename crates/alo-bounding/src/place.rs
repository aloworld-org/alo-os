//! A folder on this machine, as the two numbers the kernel knows it by.
//!
//! The grant a person made is a path. The boundary is a filesystem and an
//! inode, because that is what a BPF program on `file_open` has to compare
//! against. This file is the one conversion between them, and it happens once,
//! when the turn begins.
//!
//! # `st_dev` is not `s_dev`, and nothing says so
//!
//! The number `stat` reports for a file's device and the number the kernel
//! keeps in `super_block->s_dev` are **two different packings of the same major
//! and minor numbers**, and neither is labelled. Comparing one against the
//! other does not fail loudly — it simply never matches, so every file is
//! outside every grant and the boundary looks like it is working perfectly.
//!
//! ```text
//! stat reports    minor & 0xff | major << 8 | (minor & ~0xff) << 12
//! the kernel has  major << 20 | minor
//! ```
//!
//! So the major and the minor are taken apart and put back together the
//! kernel's way. `docs/quirks.md` records it, because it is exactly the kind of
//! thing that is obvious for ten minutes and then costs somebody an afternoon.

use std::{os::linux::fs::MetadataExt, path::Path};

use alo_bounding_map::Place;

use crate::failing::NotBounded;

/// The place the kernel knows a path by.
///
/// Whatever the path names is resolved by the machine, so a granted folder
/// reached through a symbolic link is the folder rather than the link — which
/// is `alo-files`' rule about resolving before asking, arriving here for free
/// because a filesystem has no other way to answer.
pub fn place_of(path: &Path) -> Result<Place, NotBounded> {
    let known = path.metadata().map_err(|why| NotBounded::NotAPlace {
        path: path.display().to_string(),
        why,
    })?;
    Ok(Place::of(
        as_the_kernel_keeps_it(known.st_dev()),
        known.st_ino(),
    ))
}

/// A device number as `stat` reports it, as the kernel keeps it.
///
/// Two packings of one pair of numbers; see this file's own documentation for
/// why the conversion exists at all.
#[must_use]
pub fn as_the_kernel_keeps_it(reported: u64) -> u64 {
    let major = (reported >> 8) & 0xfff;
    let minor = (reported & 0xff) | ((reported >> 12) & 0xffff_ff00);
    (major << 20) | minor
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The ordinary disks: `stat` reports one packing and the kernel keeps
    /// another, and a boundary that skipped this would match nothing at all.
    #[test]
    fn a_device_number_is_repacked_the_way_the_kernel_keeps_it() {
        // 8:2, an ordinary partition. `stat` reports 0x802; the kernel keeps
        // (8 << 20) | 2.
        assert_eq!(as_the_kernel_keeps_it(0x802), (8 << 20) | 2);
        // 0:27, which is what an overlay or a tmpfs looks like.
        assert_eq!(as_the_kernel_keeps_it(0x1b), 27);
        // A minor above 255, which is where the two packings visibly differ.
        let reported = (300 & 0xff) | (8 << 8) | ((300 & !0xff) << 12);
        assert_eq!(as_the_kernel_keeps_it(reported), (8 << 20) | 300);
    }

    /// The one this crate really depends on: a real folder answers with a place
    /// whose inode is the one the machine reports.
    #[test]
    fn a_real_folder_is_a_place() {
        let here = Path::new("/");
        let place = place_of(here).expect("the root of the machine can be looked at");
        assert_eq!(
            place.inode(),
            here.metadata().expect("root exists").st_ino()
        );
        assert_ne!(place, Place::of(0, 0));
    }

    /// A path that is not there is a refusal with the path in it, rather than a
    /// place made of zeroes — which would be a grant to whichever filesystem
    /// happens to be numbered nothing.
    #[test]
    fn a_folder_that_is_not_there_is_not_a_place() {
        assert!(matches!(
            place_of(Path::new("/nowhere-in-particular-at-all")),
            Err(NotBounded::NotAPlace { .. })
        ));
    }
}
