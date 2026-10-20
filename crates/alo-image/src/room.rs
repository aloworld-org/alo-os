//! How much room alo OS needs on a disk, in one place, for both roads.
//!
//! **This is an image's fact, not an installer's**, which is why it lives here:
//! how much disk the operating system needs is a property of what the image
//! contains, and the two programs that install it are both consumers of that
//! number rather than authors of it.
//!
//! # Why it moved, and what it was doing wrong
//!
//! It lived in `alo_installer::sizes` — the **Windows** half — and
//! `crates/alo-installer/tests/the_installer_walked_on_a_real_windows.rs` had
//! already written down what that cost, in its own words:
//!
//! > The check lives in `crate::sizes::THE_LEAST_DISK` on the **Windows** side,
//! > and it is the only thing standing between a too-small disk and
//! > `bootc install to-disk --wipe` … `crate::disks` in the environment has no
//! > size check at all … So there is exactly one guard here, on the other side
//! > of a reboot from the thing it protects.
//!
//! **On the road from Windows that is survivable**: the Windows half refuses
//! before it restarts, so a person with a 16 GB disk is told on the machine
//! they are still using.
//!
//! **On the road from a stick there is no Windows half.** A person boots the
//! stick, is offered a disk too small to hold alo OS, types its name, and
//! `bootc install to-disk --wipe` runs — one program, so the wipe and the write
//! are the same invocation. The disk is erased and the install then fails. That
//! is the worst shape a fault can take in this product, and it is reachable by
//! a person following the interface exactly.
//!
//! So the number is here, where both roads can ask it, rather than copied —
//! because **two answers to one question is how one of them is wrong later**,
//! and the one that would have been wrong is the one with a `--wipe` behind it.
//!
//! # What this module does not do
//!
//! It does not check anything. Which disks a person may be offered is the
//! installer's to decide on each road, and the environment's own answer belongs
//! with the rest of what it offers. This is the number they both read.

/// A mebibyte.
const MIB: u64 = 1024 * 1024;

/// A gibibyte.
const GIB: u64 = 1024 * MIB;

/// The least disk alo OS can be installed onto.
///
/// Held to `alo_installer::sizes::THE_LEAST_DISK` by a test in that crate, for
/// as long as that constant exists, so the two cannot drift apart while both
/// are read.
///
/// **24 GB and not a measurement of the image.** The image is about 3.55 GiB
/// compressed; the rest is the room a person needs to use the machine
/// afterwards, and `docs/design/figma-snapshot`'s `S6 · No disk can be used`
/// says the number to a person in as many words: *alo OS needs 24 GB.* A
/// number a screen already states is a number this code may not disagree with.
pub const THE_LEAST_DISK: u64 = 24 * GIB;

/// Whether a disk of this many bytes can hold alo OS.
///
/// A function rather than a comparison at each call site, so that *too small*
/// has one definition and a road that forgets the check is a road that never
/// called anything.
#[must_use]
pub const fn can_hold_alo_os(bytes: u64) -> bool {
    bytes >= THE_LEAST_DISK
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The boundary, both sides of it**, which is the one place an
    /// off-by-one in a destructive check could hide.
    #[test]
    fn a_disk_of_exactly_the_least_can_hold_it_and_one_byte_less_cannot() {
        assert!(can_hold_alo_os(THE_LEAST_DISK));
        assert!(can_hold_alo_os(THE_LEAST_DISK + 1));
        assert!(!can_hold_alo_os(THE_LEAST_DISK - 1));
        assert!(!can_hold_alo_os(0));
    }

    /// **The number a screen says to a person.**
    ///
    /// `S6 · No disk can be used` says *alo OS needs 24 GB*. If this constant
    /// moves, that sentence becomes false and a person is told a number the
    /// machine does not use.
    #[test]
    fn it_is_the_twenty_four_gigabytes_the_interface_says() {
        assert_eq!(THE_LEAST_DISK, 24 * 1024 * 1024 * 1024);
    }

    /// **A 16 GB disk is refused**, which is the disk `S6` was drawn around and
    /// the disk the walk on a real Windows used.
    #[test]
    fn the_disk_the_interface_refuses_is_refused() {
        assert!(!can_hold_alo_os(16 * GIB));
    }
}
