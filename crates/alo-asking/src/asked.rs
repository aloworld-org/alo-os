//! An answer, and the departure it came with.
//!
//! # The departure comes back, and that is the decision this file exists for
//!
//! Law 1 has two halves: *visible at the moment it happens **and afterwards in
//! a record***. The first is `alo-egress`', and this crate satisfies it by
//! being unable to open a socket without an `alo_egress::Departing`. The second
//! is `alo-record`'s, and `alo_record::Entry::left` is made from a `Departing`
//! and from nothing else — which is what makes an egress the indicator never
//! showed an entry nobody can write.
//!
//! So a crate that took the departure off the indicator itself would leave the
//! record of what left **impossible to write**, in the one crate that actually
//! causes the largest egress this product has. It hands the departure back
//! instead: `alo-files`' rule from item 6a — *the authorisation comes back
//! either way* — met one crate on, about a departure rather than about an
//! authorisation.
//!
//! ```text
//! let asked = asking.to_a_provider(&question, &hosted, &mut indicator, now)?;
//! record.keep(Entry::left(asked.departing()));
//! let answer = asked.ended(&mut indicator);
//! ```
//!
//! **And this crate still reaches `alo-record` from nowhere.** Nothing in this
//! workspace does: the record observes, and is reachable from none of the
//! crates it observes. Handing back the evidence is how that stays true while
//! the evidence still gets written.
//!
//! # Why the line stays up until then
//!
//! Between the answer arriving and [`Asked::ended`] the indicator shows an
//! egress that has just finished, which is a moment of showing something that
//! is over. The alternative is a moment of *not* showing something that is
//! happening, and only one of those two is a lie law 1 cares about.

use alo_egress::{Departing, Indicator};

use crate::answer::Answer;

/// An answer that came back, and the departure that brought it.
///
/// Not `Clone`, because a `Departing` is not: a departure that could be copied
/// could be ended twice, and the second ending would take somebody else's line
/// off the indicator.
#[derive(Debug)]
pub struct Asked {
    /// What left, still on the indicator and still to be written down.
    departing: Departing,
    /// What came back.
    answer: Answer,
}

impl Asked {
    /// Made by [`crate::Asking::to_a_provider`] and by nothing else.
    pub(crate) fn new(departing: Departing, answer: Answer) -> Self {
        Self { departing, answer }
    }

    /// What left this machine — for `alo_record::Entry::left`, which is the
    /// only way what left is written down.
    #[must_use]
    pub fn departing(&self) -> &Departing {
        &self.departing
    }

    /// The answer, before the line comes off the indicator.
    ///
    /// Borrowed rather than taken, so that keeping the record and reading the
    /// answer are not two orders a caller has to get right.
    #[must_use]
    pub fn answer(&self) -> &Answer {
        &self.answer
    }

    /// Take the line off the indicator, and keep the answer.
    ///
    /// Consumes the departure, which is `alo_egress::Indicator::ended`'s rule:
    /// one connection can never take two lines off the indicator.
    #[must_use]
    pub fn ended(self, indicator: &mut Indicator) -> Answer {
        indicator.ended(self.departing);
        self.answer
    }
}
