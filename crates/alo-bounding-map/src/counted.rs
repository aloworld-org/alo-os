//! What both halves must agree on about the one counter
//! ([ADR 0148](../../../docs/decisions/0148-the-kernel-counts-what-an-application-sends-during-a-persons-command.md)):
//! how many control groups it can count at once, and which packets are not
//! counted because nothing left.
//!
//! The kernel half adds a packet's length to the entry for the sending socket's
//! control group, **only where the desktop already put an entry**. It never makes
//! one. So the bound here is how many control groups the desktop may ask about
//! at once, and the desktop refuses a count that would need more.

/// How many control groups can be counted at once.
///
/// Eight commands counting at once, each with up to eight control groups of
/// their application, which is more than a person typing commands reaches; the
/// desktop says *cannot count* rather than counting part of an application.
pub const THE_MOST_COUNTED: u32 = 64;

/// The interface every network namespace's loopback has.
///
/// The kernel numbers loopback first in every namespace (`LOOPBACK_IFINDEX`),
/// and a packet that leaves on it has not left the machine.
pub const THE_LOOPBACK_INTERFACE: u32 = 1;

/// Whether a packet leaving on `interface` is one the counter counts.
///
/// Everything but loopback: a packet on any other interface is going somewhere
/// that is not this machine, which is what the indicator is for.
#[must_use]
pub const fn is_counted(interface: u32) -> bool {
    interface != THE_LOOPBACK_INTERFACE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_is_not_counted_and_every_other_interface_is() {
        assert!(!is_counted(THE_LOOPBACK_INTERFACE));
        assert!(is_counted(2));
        assert!(is_counted(u32::MAX));
        // An interface the kernel could not name is zero, and a packet with no
        // interface is still a packet the program saw leave a socket: counted,
        // because undercounting is the failure the indicator cannot afford.
        assert!(is_counted(0));
    }
}
