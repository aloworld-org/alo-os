//! What a machine remembers so that a proof it accepted once is not accepted
//! twice.
//!
//! A proof's tag is unique per message (`proof.rs`), so a tag seen before is a
//! message seen before — and a message replayed by whoever recorded it off the
//! wire is the one attack a correct tag does not stop by itself. [`Seen`] is
//! the memory, bounded two ways: a tag is forgotten once it is older than
//! [`WHILE_A_PROOF_STANDS`], after which `proven.rs` refuses it as being from
//! another moment anyway, and the memory holds at most [`AT_MOST`] tags.
//!
//! # Why the cap evicts rather than refuses
//!
//! Filling the memory takes [`AT_MOST`] proofs that verified within two
//! minutes, and anything that can make that many holds the pairing key
//! already, at which point replay is the least of what it can do. Refusing at
//! the cap would let anybody with the key stop every other paired machine
//! being heard, which is a worse property than forgetting the oldest.

use std::collections::HashSet;
use std::time::{Duration, SystemTime};

use crate::proof::A_TAG;

/// How long a proof stands, either side of the receiving machine's clock.
///
/// Two minutes. Long enough for two office machines whose clocks disagree by
/// what NTP leaves between them; short enough that what has to be remembered
/// to refuse a replay is small.
pub const WHILE_A_PROOF_STANDS: Duration = Duration::from_secs(120);

/// The most tags remembered at once.
pub const AT_MOST: usize = 65_536;

/// The tags accepted within the last [`WHILE_A_PROOF_STANDS`].
#[derive(Debug, Default)]
pub struct Seen {
    /// Every tag remembered, for the one question asked of every proof.
    tags: HashSet<[u8; A_TAG]>,
    /// The same tags with the moment each proof claimed, in the order they
    /// were accepted, for forgetting.
    order: Vec<([u8; A_TAG], SystemTime)>,
}

impl Seen {
    /// Nothing seen yet, which is what a machine starts with.
    #[must_use]
    pub fn nothing() -> Self {
        Self::default()
    }

    /// Remember a tag claimed at `at`, forgetting what has aged out at `now`
    /// once the memory is full.
    ///
    /// Answers `false` if the tag was already remembered — which is a replay —
    /// and remembers nothing in that case.
    pub(crate) fn remember(&mut self, tag: &[u8; A_TAG], at: SystemTime, now: SystemTime) -> bool {
        if self.tags.contains(tag) {
            return false;
        }
        if self.order.len() >= AT_MOST {
            self.forget_before(now.checked_sub(WHILE_A_PROOF_STANDS));
            if self.order.len() >= AT_MOST {
                let (oldest, _) = self.order.remove(0);
                self.tags.remove(&oldest);
            }
        }
        self.tags.insert(*tag);
        self.order.push((*tag, at));
        true
    }

    /// Forget every tag whose proof claimed a moment before `before`.
    fn forget_before(&mut self, before: Option<SystemTime>) {
        let Some(before) = before else {
            return;
        };
        self.order.retain(|(_, when)| *when >= before);
        self.tags = self.order.iter().map(|(tag, _)| *tag).collect();
    }

    /// How many tags are remembered, for a test that checks the bound.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.order.len()
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{AT_MOST, Seen, WHILE_A_PROOF_STANDS};
    use crate::testing::a_moment;

    /// A tag numbered, so a test can make as many distinct ones as it likes.
    fn tag(n: usize) -> [u8; 32] {
        let mut tag = [0_u8; 32];
        let numbered = u64::try_from(n).unwrap_or_default().to_be_bytes();
        if let Some(first_eight) = tag.get_mut(..8) {
            first_eight.copy_from_slice(&numbered);
        }
        tag
    }

    /// A tag remembered once is refused the second time.
    #[test]
    fn a_tag_seen_before_is_refused() {
        let mut seen = Seen::nothing();
        assert!(seen.remember(&tag(1), a_moment(), a_moment()));
        assert!(!seen.remember(&tag(1), a_moment(), a_moment()));
        assert!(seen.remember(&tag(2), a_moment(), a_moment()));
        assert_eq!(seen.how_many(), 2);
    }

    /// **The memory is bounded.** Once full, what has aged out of the window
    /// is forgotten first, and only then the oldest — and a tag inside the
    /// window is still refused after the forgetting.
    #[test]
    fn the_memory_is_bounded_and_forgets_what_has_aged_out_first() {
        let mut seen = Seen::nothing();
        let long_ago = a_moment() - WHILE_A_PROOF_STANDS - Duration::from_secs(1);
        assert!(seen.remember(&tag(0), long_ago, a_moment()));
        for n in 1..AT_MOST {
            assert!(seen.remember(&tag(n), a_moment(), a_moment()));
        }
        assert_eq!(seen.how_many(), AT_MOST);

        // Full. The next one makes room by forgetting the one that aged out.
        assert!(seen.remember(&tag(AT_MOST), a_moment(), a_moment()));
        assert_eq!(seen.how_many(), AT_MOST);
        assert!(!seen.remember(&tag(1), a_moment(), a_moment()));

        // Full again with nothing aged out: the oldest goes.
        assert!(seen.remember(&tag(AT_MOST + 1), a_moment(), a_moment()));
        assert_eq!(seen.how_many(), AT_MOST);
    }
}
