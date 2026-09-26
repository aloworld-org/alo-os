//! The one act: offered, then approved once or declined.
//!
//! ADR 0045 point 5 asks that what an undo keeps be *visible and forgettable*,
//! and that forgetting it be **one act**. This is the offering of that act and
//! nothing else — the sentence a person approves, and a value that exists only
//! because they did.
//!
//! # One approval, one act, held by the types
//!
//! [`Offered`] is consumed by approving or declining, so a person cannot approve
//! the same offer twice. [`Approved`] is neither [`Clone`] nor [`Copy`] and is
//! consumed by whoever carries the act out, so one approval cannot become two
//! acts. Neither is constructible from outside this crate: there is no way to
//! arrive at an [`Approved`] except through an [`Offered`] that somebody said yes
//! to, which is the guarantee ADR 0045 wants and the kind a comment cannot make.
//!
//! # Declining does nothing, and *nothing* is the whole promise
//!
//! [`Offered::declined`] returns no value at all. There is nothing for a caller
//! to act on, nothing written, and every snapshot is where it was — not because
//! this crate was careful, but because declining produces nothing that any road
//! accepts. A refusal that had to be checked for would be a refusal somebody
//! could forget to check for.
//!
//! # What this cannot do, deliberately
//!
//! It cannot forget anything. The act is `alo_letting_go`'s, this crate ships no
//! dependency on that one (its own header says why), and an [`Approved`] is a
//! person's warrant rather than a performance of it. The caller is the session
//! the person is sitting at; it holds the warrant, and it takes the road.

use std::time::SystemTime;

use alo_keeping_up::WhatWasKept;
use alo_strings::{Said, Strings};

/// The one act, offered and not yet answered.
///
/// Carries nothing: what is being offered is the same act every time, and an
/// offer that carried the size or the window would be an offer that went stale
/// between being made and being taken.
#[derive(Debug)]
#[must_use]
pub struct Offered {
    /// Constructible only in this crate, and only from a machine that keeps.
    made: (),
}

/// One approval, and therefore one act.
///
/// Not [`Clone`] and not [`Copy`], so that a caller cannot get two acts out of
/// one person saying yes once.
#[derive(Debug)]
#[must_use]
pub struct Approved {
    /// When they approved it — which is what the act itself records.
    approved: SystemTime,
}

impl Offered {
    /// Offered, which only `crate::WhatIsKept::may_forget` may decide.
    pub(crate) const fn new() -> Self {
        Self { made: () }
    }

    /// **The sentence a person approves**, and there is one.
    ///
    /// `alo_keeping_up::WhatWasKept::forgetting` and nothing else. It already
    /// says both halves ADR 0045 point 5 asks for — that nothing the agent
    /// changed can be put back afterwards, and that the space comes back — and a
    /// pane that worded it again would be a second wording of one moment, which
    /// is the thing this repository refuses everywhere else.
    #[must_use]
    pub fn sentence(strings: &Strings) -> Said {
        WhatWasKept::forgetting(strings)
    }

    /// They approved it, at `now`.
    ///
    /// Consumes the offer, so this offer cannot be approved a second time. The
    /// warrant it returns is itself `#[must_use]`, so an approval that is taken
    /// and dropped is a warning rather than a person approving into nothing.
    pub fn approved(self, now: SystemTime) -> Approved {
        let Self { made: () } = self;
        Approved { approved: now }
    }

    /// They declined.
    ///
    /// Returns nothing, because nothing is to be done: no value reaches a
    /// caller, so there is nothing for one to carry out and every snapshot is
    /// where it was. The offer is consumed, so a decline cannot be revisited
    /// into an approval — a person who changes their mind is offered again,
    /// which is a new decision and reads as one.
    pub fn declined(self) {
        let Self { made: () } = self;
    }
}

impl Approved {
    /// The moment they approved it, which the act records as its own.
    #[must_use]
    pub const fn approved(&self) -> SystemTime {
        self.approved
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::{Holding, WhatIsKept};
    use alo_keeping_up::HowFarBack;
    use std::time::Duration;

    /// A machine that keeps, holding something.
    fn a_machine_that_keeps() -> WhatIsKept {
        let node = alo_measuring::Node {
            name: "undo".to_owned(),
            at: std::path::PathBuf::from("/var/lib/alo/undo"),
            kind: alo_files::Kind::Folder,
            own: 0,
            size: 4096,
            counted: alo_measuring::Counted::Whole,
            children: Vec::new(),
        };
        WhatIsKept::keeping(HowFarBack::AS_SHIPPED, Holding::measured(&node))
    }

    /// Every sentence a person might meet here.
    fn strings() -> Strings {
        let mut vocabulary = alo_strings::Vocabulary::empty();
        crate::words::declare_into(&mut vocabulary).unwrap();
        alo_keeping_up::declare_into(&mut vocabulary).unwrap();
        alo_measuring::declare_into(&mut vocabulary).unwrap();
        Strings::of(vocabulary)
    }

    /// **The sentence is the one that already existed**, word for word, and not
    /// a wording of this crate's.
    #[test]
    fn the_sentence_is_the_one_alo_keeping_up_already_had() {
        let strings = strings();
        assert_eq!(
            Offered::sentence(&strings).text(),
            WhatWasKept::forgetting(&strings).text()
        );
    }

    /// **One approval yields one warrant**, carrying the moment it was given.
    #[test]
    fn one_approval_yields_one_warrant_at_the_moment_it_was_given() {
        let at = SystemTime::UNIX_EPOCH + Duration::from_secs(1_790_000_000);
        let approved = a_machine_that_keeps().may_forget().unwrap().approved(at);
        assert_eq!(approved.approved(), at);
    }

    /// **An act they declined produces nothing at all.** Not an empty value, not
    /// a refusal to inspect — nothing reaches a caller, so there is nothing that
    /// any road would accept and every snapshot is where it was.
    #[test]
    fn an_act_declined_produces_nothing_a_caller_could_carry_out() {
        let offered = a_machine_that_keeps().may_forget().unwrap();
        let nothing: () = offered.declined();
        assert_eq!(nothing, ());
    }

    /// **Nothing happens until they give it.** Offering is not approving: the
    /// pane can be read, the sentence shown, and until somebody calls
    /// [`Offered::approved`] no warrant exists anywhere.
    #[test]
    fn offering_is_not_approving() {
        let kept = a_machine_that_keeps();
        let strings = strings();
        let offered = kept.may_forget().unwrap();
        // The sentence can be read as many times as a surface likes.
        assert!(!Offered::sentence(&strings).text().is_empty());
        assert!(!Offered::sentence(&strings).text().is_empty());
        // And the offer is still an offer, not a warrant.
        offered.declined();
    }
}
