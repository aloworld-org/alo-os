//! The lengths a person may give a folder for, and how long each one is.
//!
//! [ADR 0001](../../../docs/decisions/0001-the-agent-is-the-interface.md) §3
//! says grants are *expiring by default* and that *there is no grant that
//! outlives the reason it was made*. It does not say how long, and
//! `alo_picking::Granting::of` refuses both zero and no end — so a length has
//! to be chosen, and a person meets it the first time they grant a folder.
//!
//! **The person chooses it, every time.** Asked on 2026-10-24, the owner chose
//! *ask me each time* over any default, which is law 5 read the way it is
//! written: a person decides what their own machine does, and how long the
//! agent may see their files is not a decision to make on their behalf once
//! and never mention again.
//!
//! # Three, and why these three
//!
//! They answer three different reasons for granting, which is what keeps them
//! from being a slider:
//!
//! - **an hour** is *do this one thing now* — a person who wants alo to sort
//!   out one folder this afternoon and has no wish to think about it again;
//! - **a day** is *I am working on this today*, and survives a restart;
//! - **a month** is *this is where I work*, for the folder somebody would
//!   otherwise re-grant every morning until they started resenting it.
//!
//! There is no *forever*, and that is not an omission: it is ADR 0001's own
//! sentence, and the code refuses a grant with no end before this file gets a
//! say. There is no *until I shut down* either, for a duller reason — a grant
//! is kept as the moment it expires, and a shutdown is not a moment anything
//! can name in advance.

use std::time::Duration;

use alo_strings::Word;

use crate::the_pickers_words::{FOR_A_DAY, FOR_A_MONTH, FOR_AN_HOUR};

/// How long a person is giving the agent a folder for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HowLong {
    /// Long enough for one piece of work.
    AnHour,
    /// A working day, surviving a restart.
    ADay,
    /// The folder somebody actually works in.
    AMonth,
}

impl HowLong {
    /// Every length a person is offered, shortest first.
    ///
    /// **Shortest first on purpose.** The row a person lands on is the first
    /// one, so the least they can give is the one that costs no decision —
    /// which is the right way round for a list whose other end hands over a
    /// month of access to their files.
    pub const ALL: [Self; 3] = [Self::AnHour, Self::ADay, Self::AMonth];

    /// How long the grant lasts.
    ///
    /// A month is 30 days rather than a calendar month, which is what every
    /// other *a month* in software means and what a person reading *a month*
    /// will not be surprised by. Nothing here reads a calendar.
    #[must_use]
    pub const fn lasting(self) -> Duration {
        match self {
            Self::AnHour => Duration::from_secs(60 * 60),
            Self::ADay => Duration::from_secs(24 * 60 * 60),
            Self::AMonth => Duration::from_secs(30 * 24 * 60 * 60),
        }
    }

    /// What the person reads on the row.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::AnHour => FOR_AN_HOUR,
            Self::ADay => FOR_A_DAY,
            Self::AMonth => FOR_A_MONTH,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shortest is first, because that is the row a person lands on.
    #[test]
    fn the_lengths_are_offered_shortest_first() {
        let mut sorted = HowLong::ALL;
        sorted.sort_by_key(|it| it.lasting());
        assert_eq!(sorted, HowLong::ALL);
    }

    /// `alo_picking::Granting::of` refuses zero and refuses no end, so every
    /// length here has to be a real stretch of time.
    #[test]
    fn no_length_is_zero_and_every_one_of_them_ends() {
        for how_long in HowLong::ALL {
            assert!(how_long.lasting() > Duration::ZERO, "{how_long:?}");
            assert!(how_long.lasting() < Duration::MAX, "{how_long:?}");
        }
    }

    /// Three different answers, not three names for one.
    #[test]
    fn no_two_lengths_are_the_same_length_or_the_same_word() {
        let mut lasting: Vec<Duration> = HowLong::ALL.iter().map(|it| it.lasting()).collect();
        lasting.sort_unstable();
        lasting.dedup();
        assert_eq!(lasting.len(), HowLong::ALL.len());

        let mut words: Vec<String> = HowLong::ALL
            .iter()
            .map(|it| it.word().key().to_string())
            .collect();
        words.sort_unstable();
        words.dedup();
        assert_eq!(words.len(), HowLong::ALL.len());
    }

    /// The words a person reads say the same thing the code does, which is the
    /// one way this file can lie to somebody.
    #[test]
    fn each_length_is_the_length_its_own_word_claims() {
        assert_eq!(HowLong::AnHour.lasting(), Duration::from_secs(3_600));
        assert_eq!(HowLong::ADay.lasting(), Duration::from_secs(86_400));
        assert_eq!(HowLong::AMonth.lasting(), Duration::from_secs(2_592_000));
    }
}
