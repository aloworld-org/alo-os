//! **When a new machine says, once, how to hear its screen**
//! ([ADR 0155](../../../docs/decisions/0155-the-screens-before-sign-in-speak.md),
//! the owner's answer of 2026-11-14: *"Off, with one spoken hint"*).
//!
//! The screen reader is off as a machine comes. So that somebody who cannot
//! see the first screen still learns how to turn it on, the first screen of a
//! new machine says one sentence — `crate::sign_in_words::HEAR_THE_SCREEN`,
//! *Press Control, Alt and S to hear the screen read aloud* — after a pause in
//! which nobody pressed anything. This file decides **when**; the reader on the
//! sign-in login says it (`crate::the_sign_in_screens_reader`), with the voice
//! alone and without starting the screen reader.
//!
//! # Once, and only where it helps
//!
//! - **Only on the first start**: a machine that already has an account shows
//!   the sign-in screen, and is never told it.
//! - **Never once anything was pressed** — a key or a click. A person who is
//!   already using the screen does not need telling how to hear it.
//! - **Never while the reader is reading**: it would be talking over itself.
//! - **Once**: here, once per screen; and the reader that says it says it once
//!   for as long as it runs, which is the whole boot, so a sign-in screen that
//!   is started again does not repeat it.

use std::time::{Duration, Instant};

/// **How long the first screen waits, with nothing pressed, before it says
/// how to hear it**: long enough that somebody looking at the screen has
/// started choosing, short enough that somebody who cannot see it is not left
/// in silence wondering whether the machine has started.
pub const THE_PAUSE: Duration = Duration::from_secs(10);

/// **Whether the hint is still to be said**, and from when the pause runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheSpokenHint {
    /// When the screen came up.
    began: Instant,
    /// Whether it can no longer be said: something was pressed, or it was.
    done: bool,
}

impl TheSpokenHint {
    /// A screen that came up at `began`, with nothing said and nothing pressed.
    #[must_use]
    pub const fn from(began: Instant) -> Self {
        Self { began, done: false }
    }

    /// **Something was pressed**: a key or a click. The hint is never said
    /// after that.
    pub const fn something_was_pressed(&mut self) {
        self.done = true;
    }

    /// **Whether to say it now**: the pause is over at `now`, nothing was
    /// pressed, the screen is the first start's (`on_the_first_start`) and
    /// the reader is not reading. Asked once a frame; says nothing about
    /// whether it was said — [`Self::said`] is told that.
    #[must_use]
    pub fn is_due(&self, now: Instant, on_the_first_start: bool, reading: bool) -> bool {
        !self.done
            && on_the_first_start
            && !reading
            && now.saturating_duration_since(self.began) >= THE_PAUSE
    }

    /// It was said, so it is never said again here.
    pub const fn said(&mut self) {
        self.done = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The moment the screen came up, and the moment the pause is over.
    fn the_pause() -> (Instant, Instant) {
        let began = Instant::now();
        (began, began + THE_PAUSE)
    }

    /// **Said after the pause and not before, and once.**
    #[test]
    fn it_is_said_after_the_pause_and_once() {
        let (began, over) = the_pause();
        let mut hint = TheSpokenHint::from(began);
        assert!(!hint.is_due(began + Duration::from_secs(1), true, false));
        assert!(hint.is_due(over, true, false));
        hint.said();
        assert!(!hint.is_due(over + THE_PAUSE, true, false), "said once");
    }

    /// **A key or a click before the pause is over means it is never said.**
    #[test]
    fn something_pressed_first_means_it_is_never_said() {
        let (began, over) = the_pause();
        let mut hint = TheSpokenHint::from(began);
        hint.something_was_pressed();
        assert!(!hint.is_due(over, true, false));
        assert!(!hint.is_due(over + THE_PAUSE * 10, true, false));
    }

    /// **Never on the sign-in screen of a machine that has an account**, and
    /// never over a reader that is already reading.
    #[test]
    fn not_on_the_sign_in_screen_and_not_over_the_reader() {
        let (began, over) = the_pause();
        let hint = TheSpokenHint::from(began);
        assert!(!hint.is_due(over, false, false), "the sign-in screen");
        assert!(!hint.is_due(over, true, true), "the reader is reading");
    }
}
