//! **Once at low, once at nearly gone, and never again.**
//!
//! A machine that tells a person about their battery every thirty seconds has
//! taught them to dismiss the thing that says their battery is going, which
//! means the message that mattered arrives in a queue of messages that did not.
//!
//! So this holds what somebody has already been told, and the whole of its
//! behaviour is:
//!
//! - **once** at [`Charge::LOW`], and **once** at [`Charge::CRITICAL`];
//! - nothing at all while the machine is on the mains;
//! - and plugging in **forgets both**, so the next time the battery runs down
//!   the person is told again. That is the one case where saying it twice is
//!   right, and it is the reason this is a value that changes rather than a flag
//!   set at boot.

use crate::reading::{Charge, Reading};
use crate::words::{self, Word};

/// **What a person has already been told about this run down the battery.**
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Telling {
    /// Whether they have been told it is low.
    told_low: bool,
    /// Whether they have been told it is nearly gone.
    told_critical: bool,
}

impl Telling {
    /// Nobody has been told anything yet.
    #[must_use]
    pub fn nothing_yet() -> Self {
        Self::default()
    }

    /// **What to say about this reading, if anything** — and what has been said
    /// afterwards.
    #[must_use]
    pub fn about(mut self, reading: &Reading) -> (Self, Option<Word>) {
        if !reading.charging().is_on_the_battery() {
            // On the mains, and everything is forgotten: the next run down is a
            // new one and a person hears about it.
            return (Self::nothing_yet(), None);
        }
        if reading.charge().is_critical() && !self.told_critical {
            self.told_critical = true;
            self.told_low = true;
            return (self, Some(words::NEARLY_GONE));
        }
        if reading.charge().is_low() && !self.told_low {
            self.told_low = true;
            return (self, Some(words::GETTING_LOW));
        }
        (self, None)
    }

    /// Whether they have been told it is low.
    #[must_use]
    pub const fn told_low(self) -> bool {
        self.told_low
    }

    /// Whether they have been told it is nearly gone.
    #[must_use]
    pub const fn told_critical(self) -> bool {
        self.told_critical
    }

    /// What a person is shown beside a battery that is neither.
    #[must_use]
    pub const fn nothing_to_say(charge: Charge) -> Option<Word> {
        if charge.is_low() {
            return Some(words::GETTING_LOW);
        }
        None
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::reading::Charging;
    use std::time::SystemTime;

    fn at(per_cent: u8, charging: Charging) -> Reading {
        Reading::taken(
            Charge::reported(per_cent).expect("a charge"),
            charging,
            Some(-1_000),
            SystemTime::UNIX_EPOCH,
        )
    }

    /// **Once at low, and then not again** however many times it is asked.
    #[test]
    fn a_person_is_told_once_that_it_is_getting_low() {
        let (telling, said) = Telling::nothing_yet().about(&at(20, Charging::Discharging));
        assert_eq!(
            said.map(|word| word.named()),
            Some(words::GETTING_LOW.named())
        );

        let mut telling = telling;
        for per_cent in [19, 18, 17, 16] {
            let (next, said) = telling.about(&at(per_cent, Charging::Discharging));
            assert!(said.is_none(), "told again at {per_cent} per cent");
            telling = next;
        }
        assert!(telling.told_low() && !telling.told_critical());
    }

    /// **Once at nearly gone**, which is a second thing to be told rather than
    /// the same thing said louder.
    #[test]
    fn nearly_gone_is_its_own_sentence_and_is_also_said_once() {
        let (telling, _) = Telling::nothing_yet().about(&at(20, Charging::Discharging));
        let (telling, said) = telling.about(&at(5, Charging::Discharging));
        assert_eq!(
            said.map(|word| word.named()),
            Some(words::NEARLY_GONE.named())
        );

        let (telling, said) = telling.about(&at(4, Charging::Discharging));
        assert!(said.is_none());
        assert!(telling.told_critical());
    }

    /// **A battery that runs down fast is still told about**: a person who went
    /// from plugged in to four per cent without passing twenty gets the sentence
    /// that matters, not neither.
    #[test]
    fn a_battery_that_fell_past_both_says_the_one_that_matters() {
        let (telling, said) = Telling::nothing_yet().about(&at(3, Charging::Discharging));
        assert_eq!(
            said.map(|word| word.named()),
            Some(words::NEARLY_GONE.named())
        );
        assert!(
            telling.told_low(),
            "the low sentence would arrive after the critical one, which reads as things getting \
             better"
        );
    }

    /// **Nothing at all on the mains, and plugging in forgets both**, so the
    /// next run down the battery is told about from the beginning.
    #[test]
    fn plugging_in_says_nothing_and_forgets_what_was_said() {
        let (telling, _) = Telling::nothing_yet().about(&at(4, Charging::Discharging));
        assert!(telling.told_critical());

        let (telling, said) = telling.about(&at(4, Charging::Charging));
        assert!(
            said.is_none(),
            "a person was told about a battery that is filling"
        );
        assert_eq!(telling, Telling::nothing_yet());

        let (_, said) = telling.about(&at(19, Charging::Discharging));
        assert_eq!(
            said.map(|word| word.named()),
            Some(words::GETTING_LOW.named())
        );
    }
}
