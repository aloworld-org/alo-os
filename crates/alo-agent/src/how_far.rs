//! How many requests one turn may make before it stops on its own.
//!
//! A model asked *what next* can answer *what next* for ever. Nothing in the
//! capability model stops it: every request it makes is validated, permitted
//! and recorded, so a thousand refused reads are a thousand correct refusals
//! and a machine doing nothing else. The bound is not a safety rail around the
//! verbs — those have their own — it is the answer to *when does a turn that is
//! getting nowhere end*.
//!
//! # Why the caller passes it
//!
//! Because it is a judgement about the work, not about the machine. *Tidy my
//! invoices* is two or three steps; *find every contract Anna sent before the
//! summer and archive them* is more. A constant here would be this crate
//! deciding how patient a person's machine is, which is the kind of number
//! `CLAUDE.md` says belongs where somebody can see it.
//!
//! [`HowFar::THE_ORDINARY`] is what a surface that has no opinion should pass,
//! and it is a decision rather than a measurement: **no multi-step turn has
//! ever been measured**, so there is no number to derive it from, and saying so
//! here is cheaper than letting a reader assume one.

/// How many rounds a turn may take.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HowFar {
    /// The most requests this turn may make, counting the first.
    rounds: usize,
}

impl HowFar {
    /// What a surface with no opinion passes.
    ///
    /// Eight: enough for a job of several steps with a refusal or two on the
    /// way, and few enough that a model going in circles stops while a person
    /// is still watching. **Not measured** — see this module's header.
    pub const THE_ORDINARY: Self = Self { rounds: 8 };

    /// This many rounds, and at least one.
    ///
    /// Zero is answered as one rather than refused: a turn that may take no
    /// requests is not a turn, and a caller that computed zero meant *the
    /// smallest a turn can be*.
    #[must_use]
    pub const fn of(rounds: usize) -> Self {
        Self {
            rounds: if rounds == 0 { 1 } else { rounds },
        }
    }

    /// The most requests this turn may make.
    #[must_use]
    pub const fn rounds(self) -> usize {
        self.rounds
    }
}

#[cfg(test)]
mod tests {
    use super::HowFar;

    #[test]
    fn a_turn_always_gets_at_least_one_round() {
        assert_eq!(HowFar::of(0).rounds(), 1);
        assert_eq!(HowFar::of(1).rounds(), 1);
        assert_eq!(HowFar::of(5).rounds(), 5);
    }

    #[test]
    fn the_ordinary_is_more_than_one_and_small_enough_to_watch() {
        assert!(
            HowFar::THE_ORDINARY.rounds() > 1,
            "a turn that cannot take a second step cannot finish a job of two"
        );
        assert!(
            HowFar::THE_ORDINARY.rounds() <= 12,
            "a turn nobody is still watching is a turn nobody asked for"
        );
    }
}
