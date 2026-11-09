//! **A window's controls fade in and out** rather than appearing and
//! vanishing — and change at once for a person who asked for less motion.
//!
//! The owner's ruling of 2026-11-09 (`docs/design/the-window-controls.md`):
//! *no wait, continuous path*. The controls start to appear the moment the
//! pointer reaches the band and start to go the moment it truly leaves; there
//! is **no delay before either**, which is the standing promise *no reveal or
//! hide delays*. What the ruling adds is how the change looks: **about 120 ms
//! in, about 160 ms out, immediate under reduced motion.**
//!
//! # A fade is how a change is shown, never whether it happened
//!
//! Whether the controls are there is `crate::the_window_edge_reveals`' answer,
//! and **only that answer decides what a press reaches**: a control fading out
//! has already stopped answering, because `crate::window_edge` lays out none
//! once the band is concealed. So the specification's *invisible controls are
//! never clickable* holds through the fade, and a person is never surprised by
//! a press landing on something half gone.
//!
//! # A duration, so a long way is not a long wait
//!
//! A fade turned round part-way — the pointer leaves and comes straight back —
//! carries on from how far it had got, taking only the share of its length that
//! is left, so it never jumps and never takes longer than its own length.

use std::time::{Duration, Instant};

/// How long the controls take to appear. The owner's *fade-in ~120 ms*.
pub(crate) const THEY_COME_IN: Duration = Duration::from_millis(120);

/// How long they take to go. The owner's *fade-out ~160 ms*.
pub(crate) const THEY_GO_IN: Duration = Duration::from_millis(160);

/// Fully shown, in thousandths.
pub(crate) const ALL_THE_WAY: u16 = 1000;

/// One window's controls on their way in or out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TheFade {
    /// Whether they are coming in (true) or going (false).
    towards_shown: bool,
    /// How shown they were when this direction began, in thousandths.
    from: u16,
    /// When this direction began.
    began: Instant,
}

impl TheFade {
    /// Controls that have been as they are for as long as anybody can tell.
    #[must_use]
    pub(crate) const fn settled(shown: bool, now: Instant) -> Self {
        Self {
            towards_shown: shown,
            from: if shown { ALL_THE_WAY } else { 0 },
            began: now,
        }
    }

    /// How shown they are at `now`, in thousandths: 0 is gone, 1000 is there.
    ///
    /// **Under reduced motion, wherever they are heading is where they are.**
    #[must_use]
    pub(crate) fn how_shown(&self, now: Instant, less_motion: bool) -> u16 {
        let goal = if self.towards_shown { ALL_THE_WAY } else { 0 };
        if less_motion {
            return goal;
        }
        let length = if self.towards_shown {
            THEY_COME_IN
        } else {
            THEY_GO_IN
        };
        let distance = u32::from(goal.abs_diff(self.from));
        // The share of the full length this distance needs.
        let needs = length.mul_f64(f64::from(distance) / f64::from(ALL_THE_WAY));
        let gone = now.saturating_duration_since(self.began);
        if needs.is_zero() || gone >= needs {
            return goal;
        }
        let covered = gone.as_secs_f64() / needs.as_secs_f64();
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a share of at most 1000, rounded, is well inside a u16 and never negative"
        )]
        let moved = (f64::from(distance) * covered).round().clamp(0.0, 1000.0) as u16;
        if self.towards_shown {
            self.from.saturating_add(moved).min(ALL_THE_WAY)
        } else {
            self.from.saturating_sub(moved)
        }
    }

    /// They are now heading `shown`: unchanged if they already were, and
    /// turned round from where they had got to if not.
    #[must_use]
    pub(crate) fn heading(self, shown: bool, now: Instant, less_motion: bool) -> Self {
        if shown == self.towards_shown {
            return self;
        }
        Self {
            towards_shown: shown,
            from: self.how_shown(now, less_motion),
            began: now,
        }
    }
}

#[cfg(test)]
#[path = "the_controls_fade_tests.rs"]
mod tests;
