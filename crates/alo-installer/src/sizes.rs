//! The sizes the installer works in, and how a size is said.
//!
//! Every size a person reads is a whole number of gigabytes, the unit Windows'
//! own Explorer shows a drive in (it counts in 1024s and writes GB). Which way
//! it rounds depends on what the number is about, and there are three:
//!
//! - a size the person is told is **free** rounds down ([`had`]);
//! - a size they are told is **needed** rounds up ([`needed`]);
//!
//! so that the installer never promises space it does not have. And:
//!
//! - a size that was or is about to be **taken** rounds to the nearest
//!   ([`taken`]).
//!
//! The third is not a promise about free space, it is a description of
//! something that happened, so neither of the other two rules fits it. Through
//! `needed` a shrink that gave up a gibibyte and an alignment sliver reads *2
//! GB*, and the installer would say it was taking two while taking one.
//! Through `had`, giving up sixty and a half gigabytes reads *60*. Nearest is
//! the only one of the three that is never wrong by more than half a gigabyte
//! in either direction, which is what a description owes a person.

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
/// at least 24 GB.
///
/// **The reason written here used to be *the operating system and the model it
/// arrives with*, and there is no model any more** — ADR 0095 took the weights
/// out of the release, so a machine arrives with none. The number has not moved
/// with the reason, deliberately: a refusal threshold is a decision about
/// somebody's only computer, 24 GB was never only the model's size, and
/// lowering it to let more machines through is a thing to do on purpose with a
/// measurement behind it rather than as a side effect of another change.
/// `docs/booting.md` carries the same note beside the 17 GB floor.
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

/// A size a person is told was taken from them, or is about to be.
///
/// Rounded to the nearest gigabyte, for the reason at the top of this module:
/// it describes an amount rather than promising free space, and the two sizes
/// it is used for - what the shrink is about to take, and what a failed install
/// left missing - have to be the same number on the same run. They were not:
/// one was a constant gibibyte and the other was whatever happened.
#[must_use]
pub fn taken(bytes: u64) -> String {
    ((bytes + GIB / 2) / GIB).to_string()
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

    /// **What is had rounds down, what is needed rounds up, and what was
    /// taken rounds to the nearest.**
    #[test]
    fn each_kind_of_size_rounds_its_own_way() {
        assert_eq!(had(GIB + GIB / 2), "1");
        assert_eq!(needed(GIB + 1), "2");
        assert_eq!(had(GIB), "1");
        assert_eq!(needed(GIB), "1");
        assert_eq!(aligned(3 * MIB + 5), 3 * MIB);

        // **The case the installer actually meets**: the shrink is aligned
        // down to a mebibyte, so what it frees is the area and a sliver more.
        // A person watching is told one gigabyte, which is what went.
        assert_eq!(taken(THE_AREA + MIB - 1), "1");
        assert_eq!(taken(GIB), "1");
        // And it does not understate a large one, which `had` would.
        assert_eq!(taken(60 * GIB + GIB / 2 + 1), "61");
        assert_eq!(taken(60 * GIB + GIB / 2 - 1), "60");
        // Nothing taken is nothing said, not a rounded-up one.
        assert_eq!(taken(0), "0");
    }
}
