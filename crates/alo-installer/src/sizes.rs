//! The sizes the installer works in, and how a size is said.
//!
//! Every size a person reads is a whole number of gigabytes, the unit Windows'
//! own Explorer shows a drive in (it counts in 1024s and writes GB). A size
//! the person is told is free is rounded **down** and a size they are told is
//! needed is rounded **up**, so the installer never promises space it does not
//! have.

/// A mebibyte.
pub const MIB: u64 = 1024 * 1024;

/// A gibibyte, which Windows writes GB.
pub const GIB: u64 = 1024 * MIB;

/// How big the installer's area is.
///
/// The environment `image/installing` builds is a kernel, an initramfs holding
/// the tools that write a disk, and the base's signed loaders — a few hundred
/// mebibytes. A gibibyte holds it with room for the environment to grow, and
/// `crate::environment` refuses a download whose files would not fit.
pub const THE_AREA: u64 = GIB;

/// How much free space Windows keeps after it is made smaller.
///
/// Windows needs room to update itself, and a Windows that cannot update
/// because alo OS took its space is a Windows made worse by installing alo OS.
/// Sixteen gibibytes is more than a feature update asks for.
pub const WINDOWS_KEEPS_FREE: u64 = 16 * GIB;

/// The smallest disk alo OS is installed onto.
///
/// `docs/booting.md`: the environment's test machine gives it a second disk of
/// at least 24 GB, which holds the operating system and the model it arrives
/// with.
pub const THE_LEAST_DISK: u64 = 24 * GIB;

/// A size a person is told they have.
#[must_use]
pub fn had(bytes: u64) -> String {
    (bytes / GIB).to_string()
}

/// A size a person is told is needed.
#[must_use]
pub fn needed(bytes: u64) -> String {
    bytes.div_ceil(GIB).to_string()
}

/// A size rounded down to a whole mebibyte, which is how Windows aligns
/// partitions.
#[must_use]
pub fn aligned(bytes: u64) -> u64 {
    bytes - bytes % MIB
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **What is had rounds down, and what is needed rounds up.**
    #[test]
    fn had_rounds_down_and_needed_rounds_up() {
        assert_eq!(had(GIB + GIB / 2), "1");
        assert_eq!(needed(GIB + 1), "2");
        assert_eq!(had(GIB), "1");
        assert_eq!(needed(GIB), "1");
        assert_eq!(aligned(3 * MIB + 5), 3 * MIB);
    }
}
