//! Where the one departure a check makes is written down.
//!
//! `alo_looking::Noting` is the door; this is what stands behind it on a booted
//! machine. `alo_record::Entry::left_on_its_own` is the only entry that can be
//! made for an errand, and it can only be made from the `alo_egress::Underway`
//! the indicator itself produced — so there is no way to write down a check
//! that was never shown, and none to make a check without writing it down.
//!
//! # It cannot refuse, so it keeps what went wrong instead
//!
//! `alo_looking::Noting::the_check_left` answers nothing, for
//! `alo_record::Record::keep`'s reason: a record with a way to decline would be
//! a record with a way to be silent about exactly the entry that mattered. A
//! disk that will not take the line is still a fact somebody has to be told, so
//! it is kept here and read back by [`IntoTheRecord::what_went_wrong`] after
//! the check, and the program says so and stops with the answer unkept — a
//! machine whose record cannot account for what it did has not finished the act.

use alo_egress::Underway;
use alo_keeping::{NotKept, Writing};
use alo_looking::Noting;
use alo_record::Entry;

/// The machine's own record of what it checked, open to be added to.
#[derive(Debug)]
pub struct IntoTheRecord<'a> {
    /// The open record file.
    writing: &'a mut Writing,
    /// What went wrong writing the one line, where it did.
    wrong: Option<NotKept>,
}

impl<'a> IntoTheRecord<'a> {
    /// Write the check into this record.
    pub fn writing_into(writing: &'a mut Writing) -> Self {
        Self {
            writing,
            wrong: None,
        }
    }

    /// What went wrong writing the check down, if anything did.
    ///
    /// Consuming, because there is exactly one check and therefore exactly one
    /// answer to this: a caller that could ask twice would be a caller that
    /// could ask before the check.
    #[must_use]
    pub fn what_went_wrong(self) -> Option<NotKept> {
        self.wrong
    }
}

impl Noting for IntoTheRecord<'_> {
    /// Written and synced before this answers, or kept as the failure it was.
    fn the_check_left(&mut self, underway: &Underway) {
        if let Err(why) = self.writing.keep(&Entry::left_on_its_own(underway)) {
            self.wrong = Some(why);
        }
    }
}
