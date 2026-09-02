//! What one entry says happened.
//!
//! Four things can happen to an agent's attempt on this machine, and the record
//! keeps all four. Three of them are refusals or near-refusals, which is the
//! point: **a record that keeps only successes cannot answer what a security
//! review actually asks.** "The agent tried and was stopped" is the sentence
//! that matters, and it is worthless if the only entries are the ones where
//! nothing went wrong.
//!
//! - [`Happened::Ran`] — it ran, with the four answers ADR 0001 §7 asks for;
//! - [`Happened::Stopped`] — a call that was properly formed and was refused
//!   anyway. [`Stopped`] says where in the journey it was stopped, because
//!   *nobody was even asked*, *the person said no* and *the grants said no at
//!   the last moment* are three different facts about a machine;
//! - [`Happened::TurnedAway`] — something that never became a call at all: a
//!   verb that is not on the list, or an argument that did not survive
//!   validation. It has no arguments kept against it, and that is deliberate —
//!   see [`Happened::TurnedAway`];
//! - [`Happened::Answered`] — where a question was answered (ADR 0008), so that
//!   "where did that answer come from" is answerable afterwards and not only at
//!   the moment it appeared.
//!
//! **Numbers, not handles.** An approval and a grant are recorded by their
//! number rather than as an [`alo_capability::ProposalId`] or a
//! [`alo_capability::GrantId`]. A handle read back off a disk would be a handle
//! into a list that has moved on — a live thing pointing at something that may
//! no longer exist — and a record holds facts about the past, not references
//! into the present.

use alo_models::InferenceSource;
use serde::{Deserialize, Serialize};

use crate::line::Line;
use crate::what::What;

/// Where in its journey a properly formed call was stopped.
///
/// The three are not degrees of the same thing. A change nobody was asked
/// about never interrupted anybody; a change a person declined is the system
/// working exactly as intended; a change stopped at the last moment is a grant
/// that changed under it. A record that flattened them into "refused" would
/// answer none of the questions each one raises.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Stopped {
    /// It was never put to a person: a read offered for approval, or a change
    /// the grants already refused. Nobody was interrupted, and the words are
    /// the refusal's own.
    BeforeAnybodyWasAsked(Line),
    /// The person said no.
    ///
    /// Nothing is kept about why. "No" is the whole answer, and a system that
    /// recorded a reason would be a system that asked for one.
    ByThePerson,
    /// The grants were asked at the moment it would have run, and said no.
    ///
    /// This is where a grant revoked after an approval takes effect, so an
    /// entry here is often not a misbehaving agent but a person changing their
    /// mind and the machine honouring it.
    AtTheMoment(Line),
}

impl Stopped {
    /// Why it was stopped, in words — empty when the person simply said no.
    #[must_use]
    pub fn why(&self) -> Option<&Line> {
        match self {
            Self::BeforeAnybodyWasAsked(why) | Self::AtTheMoment(why) => Some(why),
            Self::ByThePerson => None,
        }
    }
}

/// One thing that happened.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Happened {
    /// A verb ran, with everything ADR 0001 §7 asks of the record: what ran,
    /// under whose authority, from which approval, and against which grant.
    Ran {
        /// Whose authority it ran under.
        agent: Line,
        /// What ran.
        what: What,
        /// The approval it was redeemed from — absent for a read, which needs
        /// none, and that absence is an answer rather than a gap.
        from_approval: Option<u64>,
        /// The grants that permitted it, one for each thing it touched. Empty
        /// for a verb whose author wrote down why it needs no grant.
        against: Vec<u64>,
    },
    /// A properly formed call that was refused.
    Stopped {
        /// Whose authority it would have run under.
        agent: Line,
        /// What would have run.
        what: What,
        /// Where in the journey it was stopped.
        how: Stopped,
    },
    /// Something that never became a call at all — a verb that is not on the
    /// list, or an argument that did not survive validation.
    ///
    /// **No arguments are kept.** The arguments of a call that never validated
    /// are whatever the model was persuaded to send, and a record is read by
    /// people: keeping unvalidated text under an entry that looks like every
    /// other entry is how a record gets used to say something nobody did. The
    /// verb name and the refusal go through [`Line`] for the same reason.
    TurnedAway {
        /// Whose authority it would have run under.
        agent: Line,
        /// The verb that was asked for, as it was asked for.
        verb: Line,
        /// Why it did not become a call.
        why: Line,
    },
    /// A question was answered, and this is where (ADR 0008).
    ///
    /// **What was asked is not kept, and never will be.** The record answers
    /// *where did that answer come from*; a record that also kept the question
    /// would be a transcript of everything a person ever said to their machine,
    /// which is the thing this product exists not to be.
    Answered {
        /// Which agent asked.
        agent: Line,
        /// Where it was answered.
        source: InferenceSource,
    },
}

impl Happened {
    /// Whose authority this was.
    #[must_use]
    pub fn agent(&self) -> &Line {
        match self {
            Self::Ran { agent, .. }
            | Self::Stopped { agent, .. }
            | Self::TurnedAway { agent, .. }
            | Self::Answered { agent, .. } => agent,
        }
    }

    /// What ran or would have run — absent when nothing ever became a call.
    #[must_use]
    pub fn what(&self) -> Option<&What> {
        match self {
            Self::Ran { what, .. } | Self::Stopped { what, .. } => Some(what),
            Self::TurnedAway { .. } | Self::Answered { .. } => None,
        }
    }

    /// Whether the agent was stopped.
    ///
    /// Both refusals count, whether the call was well formed or not: a security
    /// review asking what was refused wants the ones that never validated as
    /// much as the ones the grants turned down.
    #[must_use]
    pub fn was_stopped(&self) -> bool {
        matches!(self, Self::Stopped { .. } | Self::TurnedAway { .. })
    }

    /// Whether a verb ran.
    #[must_use]
    pub fn ran(&self) -> bool {
        matches!(self, Self::Ran { .. })
    }

    /// Where in its journey a properly formed call was stopped — absent when
    /// it ran, or when nothing ever became a call.
    #[must_use]
    pub fn stopped(&self) -> Option<&Stopped> {
        match self {
            Self::Stopped { how, .. } => Some(how),
            Self::Ran { .. } | Self::TurnedAway { .. } | Self::Answered { .. } => None,
        }
    }

    /// Which approval this ran from, when it ran from one.
    #[must_use]
    pub fn from_approval(&self) -> Option<u64> {
        match self {
            Self::Ran { from_approval, .. } => *from_approval,
            Self::Stopped { .. } | Self::TurnedAway { .. } | Self::Answered { .. } => None,
        }
    }

    /// Which grants this ran against.
    #[must_use]
    pub fn against(&self) -> &[u64] {
        match self {
            Self::Ran { against, .. } => against,
            Self::Stopped { .. } | Self::TurnedAway { .. } | Self::Answered { .. } => &[],
        }
    }

    /// Where a question was answered, when this entry is about one.
    #[must_use]
    pub fn source(&self) -> Option<&InferenceSource> {
        match self {
            Self::Answered { source, .. } => Some(source),
            Self::Ran { .. } | Self::Stopped { .. } | Self::TurnedAway { .. } => None,
        }
    }

    /// Whether this caused something to leave the machine.
    ///
    /// Law 1: an answer from a paired machine on the same network left the
    /// machine as surely as one from a hosted provider did, so both are egress
    /// here and the difference is said in words rather than by staying silent.
    #[must_use]
    pub fn caused_egress(&self) -> bool {
        self.source().is_some_and(InferenceSource::causes_egress)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_calls::{archiving_march, listing_invoices};

    fn ran() -> Happened {
        Happened::Ran {
            agent: Line::of("@files"),
            what: What::of(&archiving_march()),
            from_approval: Some(3),
            against: vec![0, 1],
        }
    }

    /// An execution answers all four of ADR 0001 §7's questions from the entry
    /// alone, without anything else having to be consulted.
    #[test]
    fn an_execution_answers_what_under_whom_from_which_approval_and_against_which_grant() {
        let happened = ran();
        assert!(happened.agent().is("@files"));
        assert!(
            happened
                .what()
                .is_some_and(|what| what.verb().is("move_file"))
        );
        assert_eq!(happened.from_approval(), Some(3));
        assert_eq!(happened.against(), [0, 1]);
        assert!(happened.ran());
        assert!(!happened.was_stopped());
    }

    /// A read ran under no approval, and that absence is an answer rather than
    /// a gap: no approval, because none was needed.
    #[test]
    fn a_read_ran_from_no_approval_and_says_so() {
        let happened = Happened::Ran {
            agent: Line::of("@files"),
            what: What::of(&listing_invoices()),
            from_approval: None,
            against: vec![0],
        };
        assert!(happened.ran());
        assert_eq!(happened.from_approval(), None);
        assert_eq!(happened.against(), [0]);
    }

    /// Three refusals, three different facts about the machine. Flattening them
    /// into "refused" would answer none of the questions each one raises.
    #[test]
    fn where_something_was_stopped_is_kept_apart_from_that_it_was() {
        let never_asked = Stopped::BeforeAnybodyWasAsked(Line::of("@files has not been granted"));
        assert!(
            never_asked
                .why()
                .is_some_and(|why| why.as_str().contains("has not been granted"))
        );

        let at_the_moment = Stopped::AtTheMoment(Line::of("the grant has expired"));
        assert!(at_the_moment.why().is_some());

        // A person who says no is not asked to justify it, so nothing is kept.
        assert_eq!(Stopped::ByThePerson.why(), None);
        assert_ne!(never_asked, at_the_moment);
    }

    /// A refusal is a thing that happened, and both kinds count — the call that
    /// was well formed and refused, and the one that never formed at all.
    #[test]
    fn both_kinds_of_refusal_are_refusals() {
        let stopped = Happened::Stopped {
            agent: Line::of("@files"),
            what: What::of(&archiving_march()),
            how: Stopped::ByThePerson,
        };
        assert!(stopped.was_stopped());
        assert!(!stopped.ran());
        assert!(stopped.what().is_some());

        let turned_away = Happened::TurnedAway {
            agent: Line::of("@files"),
            verb: Line::of("delete_everything"),
            why: Line::of("there is no verb called delete_everything"),
        };
        assert!(turned_away.was_stopped());
        assert!(
            turned_away.what().is_none(),
            "a call that never formed has no arguments to keep"
        );
    }

    /// Law 1: a paired machine on the same network is egress too. This is the
    /// exception somebody will one day argue for, so it is a test here as well
    /// as in `alo-models`.
    #[test]
    fn an_answer_from_the_next_room_is_still_egress() {
        let here = Happened::Answered {
            agent: Line::of("@mail"),
            source: InferenceSource::ThisMachine,
        };
        assert!(!here.caused_egress());

        let next_room = Happened::Answered {
            agent: Line::of("@mail"),
            source: InferenceSource::PairedMachine {
                machine: "the studio workstation".to_owned(),
            },
        };
        assert!(next_room.caused_egress());
        assert!(!ran().caused_egress());
    }

    /// Everything the record keeps has to survive being written down and read
    /// back, or the record only answers questions asked in the same session.
    #[test]
    fn what_happened_survives_being_written_down_and_read_back() {
        for happened in [
            ran(),
            Happened::Stopped {
                agent: Line::of("@files"),
                what: What::of(&archiving_march()),
                how: Stopped::AtTheMoment(Line::of("the grant has expired")),
            },
            Happened::TurnedAway {
                agent: Line::of("@files"),
                verb: Line::of("delete_everything"),
                why: Line::of("there is no verb called delete_everything"),
            },
            Happened::Answered {
                agent: Line::of("@mail"),
                source: InferenceSource::ThisMachine,
            },
        ] {
            let written = serde_json::to_string(&happened).unwrap_or_default();
            assert_eq!(
                serde_json::from_str::<Happened>(&written).ok(),
                Some(happened),
                "{written}"
            );
        }
    }
}
