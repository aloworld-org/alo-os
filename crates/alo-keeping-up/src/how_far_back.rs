//! How far back an undo reaches, and what falls outside it.
//!
//! The owner's first accepted term on
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md), and the
//! question behind it was the right one: *if the system keeps what a folder was
//! before every turn, will the machines not get fuller with them?* What is kept
//! costs nothing at the moment it is taken and everything the person later
//! changes or deletes, so keeping it without a bound is a disk that fills
//! quietly. **The bound is this value**, and it is one value rather than a
//! habit for the same reason [`crate::THE_RULE`] is: a promise about a person's
//! disk that lives in whoever remembered to write it is not a promise.
//!
//! # Seven days or fifty changing turns, whichever ends first
//!
//! [`HowFarBack::AS_SHIPPED`], and `whichever ends first` is the `&&` in
//! [`HowFarBack::still_reaches`]. A machine somebody uses hard reaches the fifty
//! first; a machine somebody uses on Fridays reaches the seven first; neither
//! keeps more than the person would recognise as *what I did recently*.
//!
//! # A stated default, not a default that decides for anybody
//!
//! `CLAUDE.md`: *where alo OS lets an organisation set a rule, the rule is
//! theirs to name — we ship the mechanism, never a default that decides for
//! them.* So this is a value with a shipped default that a person changes where
//! their settings are, and that an organisation may name on a machine it
//! manages (ADR 0004). **There is no maximum**, deliberately: the disk is
//! already protected by the owner's second term — under pressure the oldest go
//! first, and the machine never fills a disk to preserve an undo — so a cap
//! here would be this repository deciding for somebody what their own disk is
//! for.
//!
//! **Zero is refused.** A window that reaches nothing is undo switched off by
//! arithmetic, quietly, in a settings file; the honest way to hold nothing is
//! [`crate::WhatWasKept::forgetting`], which is one act with a sentence on it.
//!
//! # Nothing here reads a clock
//!
//! `days_ago` arrives counted, from whoever holds the moments — this crate
//! names no time, and [`crate::before`] has the reason. Counted **down**: a
//! change made six days and twenty-three hours ago is six days ago, and inside
//! a window of seven.

use serde::{Deserialize, Deserializer, Serialize};

use alo_strings::{Filling, Said, Strings};

use crate::words;

/// How far back an undo reaches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct HowFarBack {
    /// How many days back it reaches.
    days: u32,
    /// How many changing turns back it reaches.
    turns: u32,
}

/// Why a pair of numbers is not a window.
///
/// One sentence covers both, because what a person is told is the rule and not
/// which half of it they missed. Which half it was is in the value, for
/// whoever is repairing the settings it came out of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotAWindow {
    /// It reaches back no days at all.
    NoDays,
    /// It reaches back no changing turns at all.
    NoTurns,
}

impl HowFarBack {
    /// What the machine ships with: seven days, or fifty changing turns,
    /// whichever ends first.
    pub const AS_SHIPPED: Self = Self { days: 7, turns: 50 };

    /// A window of `days` days and `turns` changing turns.
    ///
    /// # Errors
    /// [`NotAWindow`] when either is zero.
    pub fn of(days: u32, turns: u32) -> Result<Self, NotAWindow> {
        if days == 0 {
            return Err(NotAWindow::NoDays);
        }
        if turns == 0 {
            return Err(NotAWindow::NoTurns);
        }
        Ok(Self { days, turns })
    }

    /// How many days back it reaches.
    #[must_use]
    pub fn days(self) -> u32 {
        self.days
    }

    /// How many changing turns back it reaches.
    #[must_use]
    pub fn turns(self) -> u32 {
        self.turns
    }

    /// Whether a change made `days_ago` days ago, with `turns_since` changing
    /// turns made after it, is still inside the window.
    #[must_use]
    pub fn still_reaches(self, days_ago: u32, turns_since: u32) -> bool {
        days_ago < self.days && turns_since < self.turns
    }

    /// How many of a machine's changing turns the window still reaches, given
    /// how many days ago each was, **newest first**.
    ///
    /// Everything after that count is outside the window and is let go — and
    /// the order is the same order the disk takes them in when it needs the
    /// room (ADR 0045, the owner's second term): the oldest first, which is the
    /// end of this list.
    #[must_use]
    pub fn how_many_it_still_reaches(self, days_ago_newest_first: &[u32]) -> usize {
        let mut reached = 0;
        for (since, days_ago) in days_ago_newest_first.iter().enumerate() {
            let Ok(since) = u32::try_from(since) else {
                break;
            };
            if !self.still_reaches(*days_ago, since) {
                break;
            }
            reached += 1;
        }
        reached
    }
}

impl Default for HowFarBack {
    fn default() -> Self {
        Self::AS_SHIPPED
    }
}

impl<'de> Deserialize<'de> for HowFarBack {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        /// The two numbers as they are written down, before either is checked.
        #[derive(Deserialize)]
        struct AsWritten {
            /// How many days back it claims to reach.
            days: u32,
            /// How many changing turns back it claims to reach.
            turns: u32,
        }
        let written = AsWritten::deserialize(deserializer)?;
        Self::of(written.days, written.turns).map_err(|why| {
            serde::de::Error::custom(format!(
                "{} days and {} changing turns is not a window: {why:?}",
                written.days, written.turns
            ))
        })
    }
}

impl NotAWindow {
    /// Every way a window is refused.
    pub const EVERY: [Self; 2] = [Self::NoDays, Self::NoTurns];

    /// What a person reads when what they set was not kept.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&words::NOT_A_WINDOW.key(), &Filling::nothing())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// **What the machine ships is seven days or fifty changing turns**, and it
    /// is what a machine nobody has configured uses.
    #[test]
    fn what_the_machine_ships_is_seven_days_or_fifty_changing_turns() {
        let window = HowFarBack::default();
        assert_eq!(window, HowFarBack::AS_SHIPPED);
        assert_eq!(window.days(), 7);
        assert_eq!(window.turns(), 50);
    }

    /// **Whichever ends first ends it.** Either bound alone puts a change
    /// outside, which is what *whichever ends first* means and what a window
    /// that took the longer of the two would not do.
    #[test]
    fn whichever_bound_ends_first_ends_the_window() {
        let window = HowFarBack::AS_SHIPPED;
        assert!(window.still_reaches(0, 0));
        assert!(window.still_reaches(6, 49));
        // Old enough, though hardly anything has happened since.
        assert!(!window.still_reaches(7, 0));
        // Recent, and fifty changing turns have happened since.
        assert!(!window.still_reaches(0, 50));
        assert!(!window.still_reaches(7, 50));
    }

    /// **A machine used hard reaches the turns first, and one used rarely
    /// reaches the days first** — read off a real list of turns rather than
    /// asserted about the arithmetic.
    #[test]
    fn a_busy_machine_runs_out_of_turns_and_a_quiet_one_runs_out_of_days() {
        let window = HowFarBack::of(7, 3).unwrap();
        let all_today = [0, 0, 0, 0, 0];
        assert_eq!(window.how_many_it_still_reaches(&all_today), 3);

        let one_a_week = [0, 7, 14];
        assert_eq!(window.how_many_it_still_reaches(&one_a_week), 1);

        assert_eq!(window.how_many_it_still_reaches(&[]), 0);
    }

    /// **A window that reaches nothing is refused**, in both halves, and what a
    /// person reads is the rule.
    #[test]
    fn a_window_that_reaches_nothing_is_refused() {
        assert_eq!(HowFarBack::of(0, 50), Err(NotAWindow::NoDays));
        assert_eq!(HowFarBack::of(7, 0), Err(NotAWindow::NoTurns));
        let strings = in_english();
        for why in NotAWindow::EVERY {
            let said = why.said(&strings);
            assert!(!said.is_a_bug(), "{said}");
            assert!(said.text().contains("at least one day"), "{said}");
        }
    }

    /// **A window read back off a disk goes through the same door**, so a
    /// settings file saying *nothing* is refused rather than read as a machine
    /// with undo quietly switched off.
    #[test]
    fn a_window_read_back_is_checked_the_way_one_made_here_is() {
        let written = serde_json::to_string(&HowFarBack::of(30, 200).unwrap()).unwrap();
        assert_eq!(
            serde_json::from_str::<HowFarBack>(&written).unwrap(),
            HowFarBack::of(30, 200).unwrap()
        );
        for refused in [r#"{"days":0,"turns":50}"#, r#"{"days":7,"turns":0}"#] {
            assert!(
                serde_json::from_str::<HowFarBack>(refused).is_err(),
                "{refused} was read as a window"
            );
        }
    }

    /// **There is no maximum**, because the disk is bounded by what goes first
    /// under pressure and not by this number — an organisation naming a long
    /// window is naming their own rule.
    #[test]
    fn a_long_window_an_organisation_names_is_kept() {
        let long = HowFarBack::of(365, 10_000).unwrap();
        assert!(long.still_reaches(364, 9_999));
        assert!(!long.still_reaches(365, 0));
    }
}
