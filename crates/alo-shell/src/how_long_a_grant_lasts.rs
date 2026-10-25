//! The lengths a person may give a folder for, and what each one means.
//!
//! [ADR 0001](../../../docs/decisions/0001-the-capability-model.md) §3
//! says grants are *expiring by default*. It does not say how long, so a
//! length has to be chosen, and a person meets it the first time they grant a
//! folder.
//!
//! **The person chooses it, every time.** Asked on 2026-10-24, the owner chose
//! *ask me each time* over any default, which is law 5 read the way it is
//! written: a person decides what their own machine does, and how long the
//! agent may see their files is not a decision to make on their behalf once
//! and never mention again.
//!
//! # Four, and why these four
//!
//! They answer four different reasons for granting, which is what keeps them
//! from being a slider:
//!
//! - **an hour** is *do this one thing now* — a person who wants alo to sort
//!   out one folder this afternoon and has no wish to think about it again;
//! - **a day** is *I am working on this today*, and survives a restart;
//! - **a month** is *this is where I work*, for the folder somebody would
//!   otherwise re-grant every morning until they started resenting it;
//! - **until I remove it** is *this is mine and alo works in it*, for the
//!   person who does not want to be asked again at all.
//!
//! # The fourth one was added, and what it replaced
//!
//! This file offered three until 2026-10-25 and said outright that *there is
//! no forever, and that is not an omission*. The owner removed the clause it
//! rested on: *I can give an agent full access or remove it but there is no
//! time limit … we should add where the user can put so as he does not need
//! to come back every one hour or month to change.* ADR 0001 is amended to
//! match, and *expiring by default* still stands — the default is a length,
//! and the one that does not end is the last row rather than the first.
//!
//! What the amendment moved is **where the protection lives**. A grant that
//! ends protects a person by running out; this one cannot, so all of its
//! protection is that they can find it in Settings and take it away. That is
//! `alo_granted::Lasts::UntilRemoved`'s whole reason for being a row.
//!
//! There is still no *until I shut down*, for a duller reason — a grant is
//! kept as the moment it expires, and a shutdown is not a moment anything can
//! name in advance.

use std::time::Duration;

use alo_picking::Granting;
use alo_strings::Word;

use crate::the_pickers_words::{FOR_A_DAY, FOR_A_MONTH, FOR_AN_HOUR, UNTIL_I_REMOVE_IT};

/// An hour, in seconds.
const AN_HOUR: u64 = 60 * 60;
/// A day, in seconds.
const A_DAY: u64 = 24 * AN_HOUR;
/// A month, in seconds — thirty days, which is what *a month* means
/// everywhere else in software. Nothing here reads a calendar.
const A_MONTH: u64 = 30 * A_DAY;

/// How long a person is giving the agent a folder for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HowLong {
    /// Long enough for one piece of work.
    AnHour,
    /// A working day, surviving a restart.
    ADay,
    /// The folder somebody actually works in.
    AMonth,
    /// Until the person takes it away, and not before.
    ///
    /// **Added 2026-10-25 by the owner's decision**, in these words: *I can
    /// give an agent full access or remove it but there is no time limit … we
    /// should add where the user can put so as he does not need to come back
    /// every one hour or month to change.* ADR 0001 is amended to match it.
    ///
    /// It is **last** and not first. A grant that never ends is the one a
    /// person forgets, so it is a deliberate reach rather than the row the
    /// ring happens to land on — the same argument the order of the three
    /// above makes, carried one step further.
    UntilIRemoveIt,
}

impl HowLong {
    /// Every length a person is offered, shortest first and the endless one
    /// last.
    ///
    /// **Shortest first on purpose.** The row a person lands on is the first
    /// one, so the least they can give is the one that costs no decision —
    /// which is the right way round for a list whose other end hands over a
    /// folder until they come back for it.
    pub const ALL: [Self; 4] = [
        Self::AnHour,
        Self::ADay,
        Self::AMonth,
        Self::UntilIRemoveIt,
    ];

    /// The grant this length means, to this agent.
    ///
    /// **The one place that decides which kind of grant a row makes**, so
    /// that a caller cannot hold a length and get the kind wrong. A
    /// `Duration` with nothing to put in it for *until I remove it* would be
    /// an `Option` meaning *for ever* in one place and *already over* in
    /// another, which is the confusion `alo_capability::Until` exists to
    /// prevent.
    #[must_use]
    pub fn granting_to(self, agent: &str) -> Granting {
        match self {
            Self::AnHour => Granting::to(agent, Duration::from_secs(AN_HOUR)),
            Self::ADay => Granting::to(agent, Duration::from_secs(A_DAY)),
            Self::AMonth => Granting::to(agent, Duration::from_secs(A_MONTH)),
            Self::UntilIRemoveIt => Granting::to_until_removed(agent),
        }
    }

    /// Whether this is the one that does not end.
    #[must_use]
    pub const fn is_until_removed(self) -> bool {
        matches!(self, Self::UntilIRemoveIt)
    }

    /// What the person reads on the row.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::AnHour => FOR_AN_HOUR,
            Self::ADay => FOR_A_DAY,
            Self::AMonth => FOR_A_MONTH,
            Self::UntilIRemoveIt => UNTIL_I_REMOVE_IT,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_picking::ForHowLong;

    /// Every test here grants to this agent.
    const HERS: &str = "@files";

    /// The lengths that end, in the order they are offered.
    fn the_lengths_that_end() -> Vec<Duration> {
        HowLong::ALL
            .iter()
            .filter_map(|it| match it.granting_to(HERS).lasting() {
                ForHowLong::Lasting(lasting) => Some(lasting),
                ForHowLong::UntilRemoved => None,
            })
            .collect()
    }

    /// The shortest is first, because that is the row a person lands on, and
    /// the one that does not end is last because it is the most to give.
    #[test]
    fn the_lengths_are_offered_shortest_first_and_the_endless_one_last() {
        let ending = the_lengths_that_end();
        let mut sorted = ending.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, ending);

        assert!(HowLong::ALL.last().unwrap().is_until_removed());
        assert_eq!(
            HowLong::ALL
                .iter()
                .filter(|it| it.is_until_removed())
                .count(),
            1
        );
    }

    /// `alo_capability::Grant::checked` refuses a zero length and refuses one
    /// with no end, so every length that is a duration has to be a real
    /// stretch of time rather than a number standing for *never*.
    #[test]
    fn no_length_is_zero_and_none_of_them_stands_in_for_never() {
        let ending = the_lengths_that_end();
        assert_eq!(ending.len(), HowLong::ALL.len() - 1);
        for lasting in ending {
            assert!(lasting > Duration::ZERO, "{lasting:?}");
            assert!(lasting < Duration::MAX, "{lasting:?}");
        }
    }

    /// Four different answers, not four names for one.
    #[test]
    fn no_two_lengths_are_the_same_length_or_the_same_word() {
        let mut lasting: Vec<ForHowLong> = HowLong::ALL
            .iter()
            .map(|it| it.granting_to(HERS).lasting())
            .collect();
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

    /// The words a person reads say the same thing the code does, which is
    /// the one way this file can lie to somebody.
    #[test]
    fn each_length_is_the_length_its_own_word_claims() {
        for (how_long, expected) in [
            (HowLong::AnHour, Duration::from_secs(3_600)),
            (HowLong::ADay, Duration::from_secs(86_400)),
            (HowLong::AMonth, Duration::from_secs(2_592_000)),
        ] {
            assert_eq!(
                how_long.granting_to(HERS).lasting(),
                ForHowLong::Lasting(expected),
                "{how_long:?}"
            );
        }
        assert_eq!(
            HowLong::UntilIRemoveIt.granting_to(HERS).lasting(),
            ForHowLong::UntilRemoved
        );
    }

    /// Every length grants to the agent it was asked about, which is the
    /// other half a caller could get wrong.
    #[test]
    fn every_length_grants_to_the_agent_it_was_given() {
        for how_long in HowLong::ALL {
            assert_eq!(how_long.granting_to(HERS).grantee(), HERS, "{how_long:?}");
        }
    }
}
