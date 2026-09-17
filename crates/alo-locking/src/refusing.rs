//! What a person at a locked machine is told when they reach for something
//! only the signed-in person may do.
//!
//! One refusal, and it says only what the lock screen already says. The two
//! things it answers — the agent's key, and an answer to a waiting change —
//! are refused for the same reason (ADR 0001: approval is the signed-in
//! person's act, and the agent is the signed-in person's), and a sentence that
//! told them apart would be the lock screen admitting that a change is waiting.

use alo_strings::{Filling, Said, Strings};

use crate::words::{self, Word};

/// This is the signed-in person's to do, and the machine is locked.
///
/// There is no `Display`: the only road to words is [`NotWhileLocked::said`],
/// in the language the person reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotWhileLocked;

impl NotWhileLocked {
    /// The string this crate declares for it.
    #[must_use]
    pub const fn word(self) -> Word {
        words::LOCKED
    }

    /// What it says: that the machine is locked, and nothing more.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// **The refusal is the lock screen's own sentence**, so refusing something
    /// adds nothing to what a stranger at the desk can read.
    #[test]
    fn a_refusal_says_only_that_the_machine_is_locked() {
        let said = NotWhileLocked.said(&in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert_eq!(said.text(), "This machine is locked");
    }
}
