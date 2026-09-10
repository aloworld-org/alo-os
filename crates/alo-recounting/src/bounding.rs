//! How much of a record is answered at once.
//!
//! A record is the whole of what a machine did, and on a machine that has been
//! working for a year that is a great many lines. Something has to say how many
//! of them an answer holds, and the choice this crate makes is that **the bound
//! is not optional**: [`crate::Recounting::about`] takes an [`AtMost`], and
//! there is no door beside it that answers with everything. A surface that could
//! ask for a year would ask for one, once, on the machine with the longest
//! record — which is the machine whose owner has the most reason to be asking.
//!
//! # It keeps the newest, and tells them oldest first
//!
//! *What has my machine been doing* is a question about the recent past, so an
//! account bounded to two hundred entries holds the **last** two hundred that
//! answered the question. They are still read oldest first, because that is the
//! order they happened in and the order somebody reads a day in.
//!
//! Keeping the first two hundred instead would answer *what did this machine do
//! when it was new*, and would go on answering it for the rest of the machine's
//! life — a bound that quietly stops the account moving is worse than no account
//! at all, because it looks like one.
//!
//! # Nothing is hidden by it
//!
//! [`crate::Account::how_many_answered`] is how many entries matched the
//! question altogether, beside the ones being shown, and
//! [`crate::Account::said`] adds a sentence when the two differ. A bound nobody
//! is told about is indistinguishable from a machine that did nothing else.

/// How many entries an account may hold.
///
/// ```
/// use alo_recounting::AtMost;
///
/// // A number a surface picked.
/// assert_eq!(AtMost::entries(50).map(AtMost::how_many), Some(50));
///
/// // None of them is not a smaller question; it is no question at all.
/// assert_eq!(AtMost::entries(0), None);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AtMost(usize);

impl AtMost {
    /// What a surface asks for when nobody has said otherwise.
    ///
    /// A number here rather than in every shell, so that two surfaces on one
    /// machine cannot disagree about how much of a record is one sitting — and
    /// so that the number is somewhere a person can be shown it.
    pub const ONE_SITTING: Self = Self(200);

    /// This many entries, or [`None`] for none at all.
    ///
    /// Zero is refused rather than answered with an empty account: an account
    /// of nothing and a machine that did nothing are the one confusion this
    /// crate exists to prevent, and a bound of zero would manufacture it.
    #[must_use]
    pub fn entries(how_many: usize) -> Option<Self> {
        (how_many > 0).then_some(Self(how_many))
    }

    /// How many.
    #[must_use]
    pub fn how_many(self) -> usize {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A bound is a number of entries, and it is the number it was made from.
    #[test]
    fn a_bound_is_the_number_it_was_made_from() {
        assert_eq!(AtMost::entries(1).map(AtMost::how_many), Some(1));
        assert_eq!(AtMost::ONE_SITTING.how_many(), 200);
    }

    /// **No account is bounded to nothing.** A surface that asked for none
    /// would be shown an empty list, which is the one thing this crate refuses
    /// to let mean *nothing happened*.
    #[test]
    fn nothing_at_all_is_not_a_bound() {
        assert_eq!(AtMost::entries(0), None);
    }

    /// Bounds compare as the numbers they are, so a surface can hold one
    /// against another without reaching inside.
    #[test]
    fn a_bigger_bound_is_bigger() {
        assert!(AtMost::entries(10) < AtMost::entries(11));
        assert!(AtMost::entries(1000) > Some(AtMost::ONE_SITTING));
    }
}
