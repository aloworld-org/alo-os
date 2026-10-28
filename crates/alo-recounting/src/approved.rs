//! Whether the person at this machine approved what ran.
//!
//! ADR 0001 §7 asks the record for four answers about every execution, and one
//! of them is *from which approval*. `alo_record::Happened::Ran` keeps it as
//! `from_approval`: the number of the approval the call was redeemed from, or
//! nothing for a read, which needs none. An approval is only ever given by the
//! person at this machine, on its own approval surface (ADR 0001 §5, and ADR
//! 0003 for a change that arrived from a paired machine), so an entry that
//! holds one is an entry somebody here said yes to.
//!
//! That is the whole of what this file says, and it says it from the entry and
//! from nothing else. It does not say **when** the person said yes: the record
//! keeps the moment the grants were asked as the change ran, not a separate
//! moment for the approval, and a time drawn here would be one the record does
//! not hold. Nor does it say **which grant** it ran under: the record keeps
//! each grant by its number, and a number is not something a person can match
//! to anything they were shown.
//!
//! Only an entry that **ran** carries the clause. A change refused at the last
//! moment had no approval spent on it, and a change handed to the broker
//! already says in its own clause that a person approved it — a second line
//! saying so would be two accounts of one fact.

use alo_record::Happened;
use alo_strings::{Filling, Said, Strings};

use crate::words::{self, Word};

/// The person at this machine was shown what ran and said yes to it.
///
/// Made from an entry and from nothing else, so an account can only say it of
/// a change the record says was approved.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Approved {
    /// Private, so that nothing outside this crate can make one.
    _from_the_record: (),
}

impl Approved {
    /// Whether this entry ran from an approval — [`None`] for a read, which
    /// needed none, and for everything that did not run.
    pub(crate) fn of(happened: &Happened) -> Option<Self> {
        match happened {
            Happened::Ran {
                from_approval: Some(_),
                ..
            } => Some(Self {
                _from_the_record: (),
            }),
            _ => None,
        }
    }

    /// The string this crate declares for it.
    #[must_use]
    pub fn word(self) -> Word {
        words::THE_PERSON_SAID_YES
    }

    /// The clause a person reads under what ran, in the language they read.
    ///
    /// Never fails and never panics, for [`crate::Outcome::said`]'s reason.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{
        archived, declined, in_english, ours_alone, ran_a_read, refused_at_the_moment,
        somewhere_of_our_own,
    };
    use crate::{AtMost, Recounting};
    use alo_record::Asking;

    /// **A record written before this clause existed says it too**, because the
    /// clause is read off a field every `ran` entry has carried since the
    /// record's first format. The line below is written by hand, in the shape
    /// the machine wrote on 2026-10-28 for the rename Demo 1 approved, and read
    /// back off a real disk the way the record window reads it.
    #[test]
    fn a_line_written_before_this_clause_existed_says_it_too() {
        let at = somewhere_of_our_own("before-the-clause");
        let line = concat!(
            r#"{"at":{"secs_since_epoch":1792000000,"nanos_since_epoch":0},"#,
            r#""happened":{"ran":{"agent":"alo","what":{"verb":"rename_file","#,
            r#""effect":"change","sentence":"rename /home/alo/Documents/march-invoice.pdf "#,
            r#"to invoice-march.pdf","arguments":{"file":{"path":"#,
            r#""/home/alo/Documents/march-invoice.pdf"},"to":{"name":"invoice-march.pdf"}}},"#,
            r#""from_approval":0,"against":[0]}}}"#,
        );
        std::fs::write(&at, format!("{{\"format\":1}}\n{line}\n")).unwrap();
        ours_alone(&at);

        let account = Recounting::kept_at(&at)
            .about(&Asking::anything(), AtMost::ONE_SITTING)
            .unwrap();
        let told = account.told();
        assert_eq!(told.len(), 1, "{account:?}");
        assert!(told[0].approved().is_some(), "{:?}", told[0]);
        assert!(
            told[0]
                .sentence()
                .is_some_and(|sentence| sentence.as_str().contains("invoice-march.pdf"))
        );
    }

    /// A change that ran from an approval says the person said yes, in the
    /// words this crate declares.
    #[test]
    fn a_change_that_ran_from_an_approval_says_the_person_said_yes() {
        let approved = Approved::of(archived().happened());
        assert!(approved.is_some());
        let said = approved.map(|approved| approved.said(&in_english()).into_text());
        assert_eq!(
            said.as_deref(),
            Some("the person at this machine was shown this and said yes before it happened")
        );
    }

    /// **A read was approved by nobody, and is not said to have been.**
    #[test]
    fn a_read_is_not_said_to_have_been_approved() {
        assert_eq!(Approved::of(ran_a_read().happened()), None);
    }

    /// Nothing that did not run is said to have been approved — not what the
    /// person declined, and not what the grants refused at the last moment.
    #[test]
    fn nothing_refused_is_said_to_have_been_approved() {
        assert_eq!(Approved::of(declined().happened()), None);
        assert_eq!(Approved::of(refused_at_the_moment().happened()), None);
    }
}
