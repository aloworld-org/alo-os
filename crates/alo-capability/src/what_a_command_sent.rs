//! **Whether a command alo ran sent anything off this computer**, so the line
//! a person reads afterwards never says *Done* as though Put back could take
//! all of it back.
//!
//! [ADR 0110](../../../docs/decisions/0110-the-agent-writes-the-command-and-only-the-person-runs-it.md)
//! §2: *"A snapshot rewinds this machine's files. It does not recall a push,
//! un-send an email, or undo anything that already left — and the surface says
//! so."* Until 2026-11-13 a `git push` alo ran was followed by *Done* beside a
//! kept snapshot, which reads as a change Put back can take back whole. Since
//! #1212 the kernel counts what each command alo runs sends (ADR 0148, as
//! amended 2026-11-13), so the line can say which of three things is true:
//!
//! | the count | the line |
//! |---|---|
//! | `0` | *Done.*, as before — nothing left, so Put back's promise is whole |
//! | more than `0` | Put back returns the files here; what it sent can't be recalled |
//! | no count at all | the same, said as *if it sent anything*, and that this computer could not count |
//!
//! **Only where a snapshot was kept.** A line for a run with no snapshot
//! already says it can't be put back, so it overstates nothing.

use crate::words::{
    DONE_AFTER_YOUR_OK_AND_IT_SENT, DONE_AFTER_YOUR_OK_AND_NOT_COUNTED,
    DONE_AT_FULL_ACCESS_AND_IT_SENT, DONE_AT_FULL_ACCESS_AND_NOT_COUNTED,
    DONE_WITHOUT_ASKING_AND_IT_SENT, DONE_WITHOUT_ASKING_AND_NOT_COUNTED,
};
use alo_strings::Word;

/// What a command sent off this computer, where that changes what a person is
/// told about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatItSent {
    /// The kernel counted bytes leaving.
    Something,
    /// Nothing counted it, so whether anything left is not known.
    NotCounted,
}

impl WhatItSent {
    /// What `sent` — bytes counted, or [`None`] where nothing counted — means
    /// for the line afterwards, or [`None`] for a command that sent nothing.
    #[must_use]
    pub const fn of(sent: Option<u64>) -> Option<Self> {
        match sent {
            Some(0) => None,
            Some(_) => Some(Self::Something),
            None => Some(Self::NotCounted),
        }
    }

    /// The line after a command run at full access, around `{sentence}`.
    #[must_use]
    pub const fn done_at_full_access(self) -> Word {
        match self {
            Self::Something => DONE_AT_FULL_ACCESS_AND_IT_SENT,
            Self::NotCounted => DONE_AT_FULL_ACCESS_AND_NOT_COUNTED,
        }
    }

    /// The line after a command the person said yes to.
    #[must_use]
    pub const fn done_after_your_ok(self) -> Word {
        match self {
            Self::Something => DONE_AFTER_YOUR_OK_AND_IT_SENT,
            Self::NotCounted => DONE_AFTER_YOUR_OK_AND_NOT_COUNTED,
        }
    }

    /// The line after a command the terminal's own switch let run.
    #[must_use]
    pub const fn done_without_asking(self) -> Word {
        match self {
            Self::Something => DONE_WITHOUT_ASKING_AND_IT_SENT,
            Self::NotCounted => DONE_WITHOUT_ASKING_AND_NOT_COUNTED,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::WhatItSent;
    use alo_strings::{Filling, Strings};

    /// **Only a count of nothing keeps plain *Done*.**
    #[test]
    fn only_a_command_counted_as_sending_nothing_keeps_plain_done() {
        assert_eq!(WhatItSent::of(Some(0)), None);
        assert_eq!(WhatItSent::of(Some(1)), Some(WhatItSent::Something));
        assert_eq!(WhatItSent::of(Some(4_096)), Some(WhatItSent::Something));
        assert_eq!(WhatItSent::of(None), Some(WhatItSent::NotCounted));
    }

    /// **Every line says Put back can't recall what left**, and none of them
    /// is only *Done*.
    #[test]
    fn every_line_says_what_put_back_cannot_reach() {
        let strings = Strings::of(crate::words::capability_words().unwrap());
        for what in [WhatItSent::Something, WhatItSent::NotCounted] {
            for word in [
                what.done_at_full_access(),
                what.done_after_your_ok(),
                what.done_without_asking(),
            ] {
                let said = strings.say(
                    &word.key(),
                    &Filling::of("sentence", "run this command: git push"),
                );
                assert!(!said.is_a_bug(), "{said:?}");
                assert!(said.text().starts_with("Done"), "{said:?}");
                assert!(
                    said.text()
                        .contains("Put back can return your files on this computer"),
                    "{said:?}"
                );
                // Each says what Put back cannot reach: what the command sent
                // off this computer, counted or not.
                assert!(said.text().contains("the command sent"), "{said:?}");
            }
        }
    }
}
