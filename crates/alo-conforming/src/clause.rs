//! One clause of the standard, as this repository holds it.

use crate::standing::{Checked, Standing};

/// **One clause**: its number, what it asks for in one sentence, where it
/// stands, and whether anybody has read it against the published text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Clause {
    /// The clause's number, as the standard writes it: `11.5.2.5`.
    pub number: &'static str,
    /// What it asks for, in one sentence in this repository's own words. Never
    /// the standard's text, which is somebody else's copyright — and never more
    /// than one sentence, because a reader comparing forty of these against a
    /// document needs to be able to read each one at a glance.
    pub requirement: &'static str,
    /// Where it stands here.
    pub standing: Standing,
    /// Whether a person has read it against the standard itself.
    pub checked: Checked,
}

impl Clause {
    /// Which of the standard's top-level clauses this one is under: the `11` of
    /// `11.5.2.5`.
    #[must_use]
    pub fn under(&self) -> &str {
        self.number.split('.').next().unwrap_or(self.number)
    }

    /// Whether this clause is met by something in this repository.
    #[must_use]
    pub const fn is_met(&self) -> bool {
        matches!(self.standing, Standing::Met { .. })
    }

    /// Whether this clause is waiting on work somebody has written down.
    #[must_use]
    pub const fn is_waiting(&self) -> bool {
        matches!(self.standing, Standing::NotYet { .. })
    }

    /// Whether this clause is one the standard has for machines this is not.
    #[must_use]
    pub const fn is_not_applicable(&self) -> bool {
        matches!(self.standing, Standing::NotApplicable { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::standing::Waiting;

    #[test]
    fn a_clause_knows_which_part_of_the_standard_it_is_under() {
        let clause = Clause {
            number: "11.5.2.5",
            requirement: "Software exposes what each thing on the screen is.",
            standing: Standing::NotYet {
                because: Waiting {
                    task: 4,
                    plan: crate::THE_PLAN,
                },
            },
            checked: Checked::NotAgainstTheText,
        };
        assert_eq!(clause.under(), "11");
        assert!(clause.is_waiting() && !clause.is_met() && !clause.is_not_applicable());
    }
}
