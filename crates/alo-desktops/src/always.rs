//! The three things that are on every desktop at once, and cannot be put on one.
//!
//! **This is the reason this file exists rather than a flag on a window.** A
//! promise that lived on desktop 1 would be a promise nobody working on desktop
//! 3 could see, and the three surfaces below are each a promise this product is
//! sold on:
//!
//! - the **egress indicator** — law 1 of `CLAUDE.md`: *every network egress an
//!   agent causes is visible at the moment it happens*. Visible on the desktop
//!   a person happens to be looking at is not visible;
//! - the **approval surface** — *reads answer, changes wait*: a change waits for
//!   one approval, and a person who switched desktop while it waited would be a
//!   person who cannot find the thing they must answer;
//! - the **agent overlay** — the one road to the agent (v0.01), which has one
//!   key and is reachable wherever a person is.
//!
//! So a display's desktops cannot be made without all three
//! ([`Promises::of`]), there is no road that puts one of them on a single
//! desktop, and every one of the three roads that could move a window
//! — [`crate::OnADisplay::put_on`], [`crate::OnADisplay::on_every_desktop`] and
//! [`crate::OnADisplay::close`] — refuses a promise with
//! [`crate::Refused::APromise`]. The promise is therefore structural rather
//! than a rule somebody has to remember, which is what
//! `docs/autonomy/v0-5-hands-on-the-desktop-plan.md` task 3 asks a test to hold.
//!
//! Nothing here draws any of them. What is here is *which window is which
//! promise*, which the shell says once, and the refusals that keep them where
//! they were put.

use alo_dividing::WindowId;
use alo_strings::{Filling, Said, Strings};

use crate::words::{self, Word};

/// A surface that is on every desktop of a display at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Always {
    /// What is leaving this machine, as it leaves.
    EgressIndicator,
    /// What is waiting for the person's approval.
    ApprovalSurface,
    /// The agent, over whatever is in front.
    AgentOverlay,
}

impl Always {
    /// All three, in the order a settings panel would list them.
    pub const ALL: [Self; 3] = [
        Self::EgressIndicator,
        Self::ApprovalSurface,
        Self::AgentOverlay,
    ];

    /// The string this crate declares for it.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::EgressIndicator => words::THE_EGRESS_INDICATOR,
            Self::ApprovalSurface => words::THE_APPROVAL_SURFACE,
            Self::AgentOverlay => words::THE_AGENT_OVERLAY,
        }
    }

    /// What this is called, in the language the person reads.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

/// Which window is which of the three, on one display.
///
/// A display's desktops are made with these and never without them, so there is
/// no moment at which a display has desktops and one of the three is missing
/// from them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Promises {
    /// The egress indicator's window.
    egress_indicator: WindowId,
    /// The approval surface's window.
    approval_surface: WindowId,
    /// The agent overlay's window.
    agent_overlay: WindowId,
}

impl Promises {
    /// The three surfaces of one display.
    ///
    /// # Errors
    /// [`TwoPromises`] when two of them are the same window. One surface
    /// standing for two promises would mean a person who dismissed one lost
    /// the other, so it is refused where it is stated rather than discovered
    /// later.
    pub fn of(
        egress_indicator: WindowId,
        approval_surface: WindowId,
        agent_overlay: WindowId,
    ) -> Result<Self, TwoPromises> {
        let promises = Self {
            egress_indicator,
            approval_surface,
            agent_overlay,
        };
        for (first, second) in [
            (Always::EgressIndicator, Always::ApprovalSurface),
            (Always::EgressIndicator, Always::AgentOverlay),
            (Always::ApprovalSurface, Always::AgentOverlay),
        ] {
            if promises.the_surface(first) == promises.the_surface(second) {
                return Err(TwoPromises {
                    first,
                    second,
                    window: promises.the_surface(first),
                });
            }
        }
        Ok(promises)
    }

    /// The window this promise is drawn in.
    #[must_use]
    pub const fn the_surface(self, always: Always) -> WindowId {
        match always {
            Always::EgressIndicator => self.egress_indicator,
            Always::ApprovalSurface => self.approval_surface,
            Always::AgentOverlay => self.agent_overlay,
        }
    }

    /// All three windows, in the order of [`Always::ALL`].
    #[must_use]
    pub const fn every_surface(self) -> [WindowId; 3] {
        [
            self.egress_indicator,
            self.approval_surface,
            self.agent_overlay,
        ]
    }

    /// Which promise this window is, when it is one.
    #[must_use]
    pub fn which(self, window: WindowId) -> Option<Always> {
        Always::ALL
            .into_iter()
            .find(|always| self.the_surface(*always) == window)
    }
}

/// Two of a display's three promises were named as one window.
///
/// **English, and said to the shell that stated them.** It is alo OS's own bug
/// and there is nothing to ask a person about it; the refusals a person reads
/// are [`crate::Refused`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error(
    "{first:?} and {second:?} cannot both be window {}: one surface standing for two promises \
     would mean dismissing one dismissed the other",
    window.to_compositor()
)]
pub struct TwoPromises {
    /// The first of the two.
    first: Always,
    /// The second.
    second: Always,
    /// The window both were named as.
    window: WindowId,
}

impl TwoPromises {
    /// The two promises that were named as one window.
    #[must_use]
    pub const fn both(&self) -> (Always, Always) {
        (self.first, self.second)
    }

    /// The window they were both named as.
    #[must_use]
    pub const fn window(&self) -> WindowId {
        self.window
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, promises, window};
    use std::collections::BTreeSet;

    /// Each promise is its own window and is found again by it.
    #[test]
    fn each_promise_is_its_own_window() {
        let promises = promises();
        let mut seen = BTreeSet::new();
        for always in Always::ALL {
            let surface = promises.the_surface(always);
            assert!(seen.insert(surface));
            assert_eq!(promises.which(surface), Some(always));
        }
        assert_eq!(promises.every_surface().len(), Always::ALL.len());
        assert_eq!(promises.which(window(99)), None);
    }

    /// Two promises in one window is refused where it is stated, naming both.
    #[test]
    fn two_promises_cannot_be_one_window() {
        let one = window(101);
        let refused = Promises::of(one, window(102), one).unwrap_err();
        assert_eq!(
            refused.both(),
            (Always::EgressIndicator, Always::AgentOverlay)
        );
        assert_eq!(refused.window(), one);
        assert!(refused.to_string().contains("101"), "{refused}");
        assert!(Promises::of(one, one, window(103)).is_err());
        assert!(Promises::of(window(101), one, one).is_err());
    }

    /// Each of the three says what it is, in the language the person reads, and
    /// no two say the same thing.
    #[test]
    fn each_promise_says_what_it_is() {
        let strings = in_english();
        let mut said = BTreeSet::new();
        for always in Always::ALL {
            let row = always.said(&strings);
            assert!(!row.is_a_bug(), "{always:?}");
            assert!(row.unfilled().is_empty(), "{always:?}");
            assert!(said.insert(row.text().to_owned()), "two are both {row}");
        }
    }
}
