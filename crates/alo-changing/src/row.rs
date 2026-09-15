//! One row of the one list of what has been granted to what.
//!
//! `docs/features.md` promises grants and pairings in **one list, revoked the
//! same way**. A list a surface draws is therefore a list of [`Row`], and
//! [`crate::Changing::revoked`] takes any of them and answers the same
//! [`crate::Gone`] — so no surface holds two revocations as two code paths,
//! and the day a third kind of row exists the compiler names every place
//! that has to learn about it.
//!
//! Each arm carries the one shape that cannot name something the machine does
//! not hold: `alo_granted::Seen` for a grant, [`SeenPairing`] for a pairing.
//! There is no arm for *everything*: a revocation is one row a person chose.

use alo_granted::Seen;

use crate::seen_pairing::SeenPairing;

/// A row of the one list, of whichever kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Row {
    /// A grant a person made to an agent.
    Grant(Seen),
    /// A pairing two people made between this machine and another.
    Pairing(SeenPairing),
}

impl From<Seen> for Row {
    fn from(row: Seen) -> Self {
        Self::Grant(row)
    }
}

impl From<SeenPairing> for Row {
    fn from(row: SeenPairing) -> Self {
        Self::Pairing(row)
    }
}
