//! A change waiting for one approval: what it would do, and how long the
//! question stands.
//!
//! ADR 0001 §5: a read answers inside the turn, a change comes back as a
//! proposal carrying the sentence describing exactly what it will do. This is
//! that proposal, and three of its rules are refusals made where it is built
//! rather than checks somebody downstream has to remember:
//!
//! - **a read is never proposed.** Asking a person to approve a question is how
//!   approving becomes a reflex, and a reflex is not consent;
//! - **a change the grants already refuse is never put to a person.** An
//!   approval that leads to "actually, no" teaches people to click through;
//! - **the question lapses.** A change proposed this morning describes a machine
//!   that has moved on, so how long it stands is named when it is made and zero
//!   is refused — the same rule a grant lives by, for the same reason.
//!
//! What a proposal is *not* is authority. Holding one — or the [`Call`] inside
//! it — is not permission to run anything: [`crate::Authorised`] is the only
//! type in this crate that means may-run, and the only way to one for a change
//! is through an approval that is spent reaching it.

use std::time::{Duration, SystemTime};

use alo_strings::{Filling, Said, Strings};
use serde::Serialize;

use crate::call::Call;
use crate::grant::Grantee;
use crate::grants::Grants;
use crate::refusing::NotGranted;
use crate::words;

/// Why a change could not be proposed.
///
/// Each of these is a refusal before anybody was asked anything, so nothing ran
/// and nobody was interrupted. They are recorded like every other refusal
/// (ADR 0001 §7), which is why the caller keeps hold of the call it offered.
///
/// **No `Display`**, and the grants' half is the grants' own refusal rather
/// than a copy of its words: what a person reads about a change that was never
/// proposed is the same sentence they would read about one refused at the
/// moment it ran.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProposalError {
    /// A read, which does not wait for anybody.
    ReadDoesNotWait {
        /// The verb that was offered.
        verb: String,
    },
    /// Something the grants do not permit.
    NotGranted(NotGranted),
    /// A question that stands for no time at all.
    NoTime,
    /// A question standing for longer than this machine can represent.
    NoEnd,
}

impl ProposalError {
    /// What this says, in the language the person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::NotGranted(why) => why.said(strings),
            Self::ReadDoesNotWait { verb } => strings.say(
                &words::READ_DOES_NOT_WAIT.key(),
                &Filling::of("verb", verb.clone()),
            ),
            Self::NoTime => strings.say(&words::PROPOSAL_NO_TIME.key(), &Filling::nothing()),
            Self::NoEnd => strings.say(&words::PROPOSAL_NO_END.key(), &Filling::nothing()),
        }
    }
}

/// A change that has been proposed, and is waiting for one person to answer.
///
/// `Serialize` so that a pending question can be shown and a declined one
/// recorded. It does not deserialise, and [`Call`] is what stops it: a proposal
/// read back off a disk would be a question about a machine that has since
/// restarted, answerable by somebody who never saw it asked.
#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Proposal {
    /// What would run, validated, with its sentence already generated.
    call: Call,
    /// Whose authority it would run under.
    grantee: Grantee,
    /// When it was put to the person.
    proposed_at: SystemTime,
    /// When the question stops standing.
    lapses: SystemTime,
}

impl Proposal {
    /// Propose a change, checking everything that has to be true of one.
    ///
    /// `from` is the moment it is proposed; `standing` is how long the question
    /// waits to be answered — the pair rather than an end time, as with
    /// [`crate::Grant::checked`], because every caller has a duration in hand.
    ///
    /// The call is borrowed rather than taken, so a refusal leaves it with the
    /// caller to record.
    ///
    /// # Errors
    /// [`ProposalError`], saying what to do instead.
    pub fn checked(
        call: &Call,
        grantee: &Grantee,
        grants: &Grants,
        from: SystemTime,
        standing: Duration,
    ) -> Result<Self, ProposalError> {
        if !call.waits_for_approval() {
            return Err(ProposalError::ReadDoesNotWait {
                verb: call.verb().to_owned(),
            });
        }
        if let Some(why) = call.refusal(grants, grantee, from) {
            return Err(ProposalError::NotGranted(why));
        }
        if standing.is_zero() {
            return Err(ProposalError::NoTime);
        }
        let lapses = from.checked_add(standing).ok_or(ProposalError::NoEnd)?;
        Ok(Self {
            call: call.clone(),
            grantee: grantee.clone(),
            proposed_at: from,
            lapses,
        })
    }

    /// What the person is being asked to approve, in words.
    ///
    /// Generated from the validated arguments when the call was made, so this
    /// is the same sentence however many times it is read.
    #[must_use]
    pub fn sentence(&self) -> &str {
        self.call.sentence()
    }

    /// The verb that would run.
    #[must_use]
    pub fn verb(&self) -> &str {
        self.call.verb()
    }

    /// The whole call, for a record of what was proposed or declined.
    #[must_use]
    pub fn call(&self) -> &Call {
        &self.call
    }

    /// Whose authority it would run under.
    #[must_use]
    pub fn grantee(&self) -> &Grantee {
        &self.grantee
    }

    /// When it was put to the person.
    #[must_use]
    pub fn proposed_at(&self) -> SystemTime {
        self.proposed_at
    }

    /// Whether the question still stands at this moment.
    ///
    /// A question that lapses at five o'clock does not include five o'clock, on
    /// the same side of the boundary as [`crate::Grant::is_active_at`].
    #[must_use]
    pub fn is_waiting_at(&self, now: SystemTime) -> bool {
        self.lapses > now
    }

    /// How long is left to answer, or `None` once it has lapsed.
    #[must_use]
    pub fn lapses_in(&self, now: SystemTime) -> Option<Duration> {
        self.lapses
            .duration_since(now)
            .ok()
            .filter(|left| !left.is_zero())
    }

    /// What an approval needs out of a proposal, once it has been answered.
    ///
    /// Crate-private: taking a proposal apart is how an approval is made, and
    /// nothing outside this crate has a reason to do it.
    pub(crate) fn into_parts(self) -> (Call, Grantee) {
        (self.call, self.grantee)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::test_calls::{archiving_march, files, granting, hour, listing_invoices, noon};
    use crate::testing::in_english;

    /// A read never becomes a question. It answers inside the turn, and asking
    /// about it would train a person to approve without reading.
    #[test]
    fn a_read_is_never_proposed() {
        let err = Proposal::checked(
            &listing_invoices(),
            &files(),
            &granting(&["/home/anna/Invoices"]),
            noon(),
            hour(),
        )
        .unwrap_err();
        assert_eq!(
            err,
            ProposalError::ReadDoesNotWait {
                verb: "list_folder".to_owned()
            }
        );
        let said = err.said(&in_english());
        assert!(said.text().contains("run it in the turn"), "{said}");
    }

    /// A change the grants refuse is never put to a person: the refusal happens
    /// here, in the grants' own words, rather than after somebody approved it.
    #[test]
    fn a_change_the_grants_refuse_is_never_put_to_a_person() {
        let err = Proposal::checked(
            &archiving_march(),
            &files(),
            &granting(&["/home/anna/Invoices"]),
            noon(),
            hour(),
        )
        .unwrap_err();
        assert!(matches!(err, ProposalError::NotGranted(_)), "{err:?}");
        let why = err.said(&in_english());
        assert!(why.text().contains("has not been granted"), "{why}");
        assert!(why.text().contains("/home/anna/Archive"), "{why}");
    }

    /// The question has an end, like a grant does. There is no proposal that
    /// waits for ever, and one that lapses at once could never be answered.
    #[test]
    fn a_question_has_to_lapse() {
        let both = granting(&["/home/anna/Invoices", "/home/anna/Archive"]);
        assert_eq!(
            Proposal::checked(&archiving_march(), &files(), &both, noon(), Duration::ZERO)
                .unwrap_err(),
            ProposalError::NoTime
        );
        assert_eq!(
            Proposal::checked(&archiving_march(), &files(), &both, noon(), Duration::MAX)
                .unwrap_err(),
            ProposalError::NoEnd
        );
    }

    /// What waits is the sentence that was generated from the validated
    /// arguments — not a description the model wrote afterwards.
    #[test]
    fn a_proposal_carries_the_sentence_that_was_generated() {
        let call = archiving_march();
        let proposal = Proposal::checked(
            &call,
            &files(),
            &granting(&["/home/anna/Invoices", "/home/anna/Archive"]),
            noon(),
            hour(),
        )
        .unwrap();
        assert_eq!(
            proposal.sentence(),
            "move /home/anna/Invoices/march.pdf into /home/anna/Archive"
        );
        assert_eq!(proposal.verb(), "move_file");
        assert_eq!(proposal.call(), &call);
        assert_eq!(proposal.grantee(), &files());
        assert_eq!(proposal.proposed_at(), noon());
        assert!(proposal.is_waiting_at(noon()));
        assert_eq!(proposal.lapses_in(noon()), Some(hour()));
        assert!(!proposal.is_waiting_at(noon() + hour()));
        assert!(proposal.lapses_in(noon() + hour()).is_none());
    }

    /// A declined question is recorded, so a proposal has to survive being
    /// written down — with the sentence somebody was actually shown.
    #[test]
    fn a_proposal_can_be_written_down() {
        let proposal = Proposal::checked(
            &archiving_march(),
            &files(),
            &granting(&["/home/anna/Invoices", "/home/anna/Archive"]),
            noon(),
            hour(),
        )
        .unwrap();
        let written = serde_json::to_string(&proposal).unwrap();
        assert!(written.contains("move_file"), "{written}");
        assert!(written.contains("@files"), "{written}");
        assert!(
            written.contains("move /home/anna/Invoices/march.pdf into"),
            "{written}"
        );
    }

    /// The errors say what to do, not what went wrong.
    #[test]
    fn the_errors_say_what_to_do() {
        let strings = in_english();
        assert!(
            ProposalError::NoTime
                .said(&strings)
                .text()
                .contains("how long")
        );
        assert!(
            ProposalError::NoEnd
                .said(&strings)
                .text()
                .contains("how long this one stands")
        );
    }
}
