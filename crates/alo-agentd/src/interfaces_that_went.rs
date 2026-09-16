//! Which interfaces the kernel said were deleted, out of what one round of the
//! service read on the routing socket.
//!
//! *Machines find each other with zero configuration.* Every set of sockets
//! that follows this machine's networks — discovery's responders
//! (`crate::responding`), the IPv6 joins (`crate::joining`) and the port's
//! listeners (`crate::listeners`) — matches what it holds to what the kernel
//! reports **by the interface's number**, and a number is not an interface: the
//! kernel gives a number that is free again to the next interface that asks for
//! it, and an interface can ask for a particular one. A cable deleted and laid
//! again at the same number **between two readings of the interfaces** — a
//! dock re-enumerating, a namespace rebuilt — reads, before and after, as the
//! same number under the same name. What did not survive is a multicast
//! membership: the kernel takes the interface out of every group when it is
//! deleted and leaves each socket's own record of the membership behind at the
//! number (`docs/quirks.md`), so a responder kept because its number is still
//! reported answers questions that never reach it.
//!
//! # How a re-laid interface is told from the one that went
//!
//! Three readings were weighed:
//!
//! - **The kernel's own message that the interface went** — this file.
//!   `RTM_DELLINK` arrives on the routing socket the service already waits on,
//!   in the order it happened, however many other messages are read with it in
//!   the same round, and it is sent exactly when the memberships went. Nothing
//!   more is asked of the kernel and nothing wakes that did not already.
//! - **Whether the interface is in the group now**, read out of
//!   `/proc/net/igmp`. It answers for the interface and not for the socket: an
//!   interface another process joined is in the group, and when that process
//!   leaves the group is gone with no routing message at all, so a responder
//!   that asked once would be deaf again with nothing to wake it.
//! - **Taking every membership afresh on every notification.** Always right,
//!   and a leave and a report on every network each time any address anywhere
//!   changes — an IPv6 router's advertisement renewing a lifetime is one — which
//!   is a cost a machine whose networks did not change should not pay.
//!
//! # And when what the kernel said cannot be read
//!
//! A routing socket nobody read in time is told `ENOBUFS` and the messages are
//! gone; a message cut short cannot be read. Either way which interfaces went
//! is not known, and [`Went`] then says **every** interface may have: every
//! membership is taken afresh once. That costs a report on each network on the
//! rare round it happens, and it never leaves a machine counted joined where it
//! is not.

use std::collections::BTreeSet;

use crate::route_messages::links_deleted_in;

/// The interfaces the kernel said were deleted in one round of the service.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Went {
    /// The numbers of the interfaces it said were deleted.
    numbered: BTreeSet<u32>,
    /// Whether something it said was lost or unreadable, so any interface may
    /// have gone.
    unknown: bool,
}

impl Went {
    /// No interface went.
    #[must_use]
    pub fn nothing() -> Self {
        Self::default()
    }

    /// Any interface may have gone: what the kernel said was lost, or could
    /// not be read.
    #[must_use]
    pub fn any_of_them() -> Self {
        Self {
            numbered: BTreeSet::new(),
            unknown: true,
        }
    }

    /// Read one datagram the kernel sent on the routing socket.
    ///
    /// One that cannot be read is [`lost`](Self::lost): which interfaces it
    /// said went is not known.
    pub fn heard(&mut self, datagram: &[u8]) {
        match links_deleted_in(datagram) {
            Ok(deleted) => self.numbered.extend(deleted),
            Err(_) => self.lost(),
        }
    }

    /// The kernel dropped messages nobody read in time.
    pub const fn lost(&mut self) {
        self.unknown = true;
    }

    /// Whether the interface numbered `index` went — or may have.
    #[must_use]
    pub fn includes(&self, index: u32) -> bool {
        self.unknown || self.numbered.contains(&index)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::Went;

    /// A link deleted, as the kernel says it: `RTM_DELLINK` in the unspecified
    /// family, for the interface numbered `index`.
    fn deleted(index: u32) -> Vec<u8> {
        let mut message = Vec::new();
        message.extend_from_slice(&32_u32.to_ne_bytes());
        message.extend_from_slice(&17_u16.to_ne_bytes());
        message.extend_from_slice(&[0; 10]);
        message.extend_from_slice(&[0, 0, 0, 0]);
        message.extend_from_slice(&index.to_ne_bytes());
        message.extend_from_slice(&[0; 8]);
        message
    }

    /// **Nothing heard is nothing gone.**
    #[test]
    fn nothing_heard_is_nothing_gone() {
        let went = Went::nothing();
        assert!(!went.includes(1));
        assert!(!went.includes(40));
    }

    /// **An interface the kernel said was deleted went, and only that one** —
    /// across every datagram of the round.
    #[test]
    fn an_interface_said_deleted_went_and_only_that_one() {
        let mut went = Went::nothing();
        went.heard(&deleted(40));
        went.heard(&[deleted(44), deleted(45)].concat());
        assert!(went.includes(40));
        assert!(went.includes(44));
        assert!(went.includes(45));
        assert!(!went.includes(41), "{went:?}");
    }

    /// **What was lost, or cannot be read, is any interface having gone**:
    /// never nothing having gone.
    #[test]
    fn what_was_lost_or_cannot_be_read_is_any_interface_having_gone() {
        let mut lost = Went::nothing();
        lost.lost();
        assert!(lost.includes(7));
        assert_eq!(lost, Went::any_of_them());

        let mut unreadable = Went::nothing();
        let whole = deleted(40);
        unreadable.heard(whole.get(..whole.len() - 4).unwrap());
        assert!(unreadable.includes(41), "{unreadable:?}");
    }
}
