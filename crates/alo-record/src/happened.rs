//! What one entry says happened.
//!
//! Six things can happen to an agent's attempt on this machine, and the record
//! keeps all six. Three of them are refusals, which is the point: **a record
//! that keeps only successes cannot answer what a security review actually
//! asks.** "The agent tried and was stopped" is the sentence that matters, and
//! it is worthless if the only entries are the ones where nothing went wrong.
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
//! - [`Happened::AnsweredHere`] — a question answered on this machine, which is
//!   the ordinary day and the thing law 1 promises there will be a great many
//!   of;
//! - [`Happened::Left`] — something left this machine (law 1);
//! - [`Happened::HeldBack`] — something the egress policy refused to let leave.
//!
//! # Where a question was answered is where it went
//!
//! The last three are one decision, and it is the decision this file exists to
//! record. An answer from a provider is two facts at once — *where that answer
//! came from* (ADR 0008) and *something left this machine* (law 1) — and an
//! earlier shape of this enum kept the first as an `Answered` entry carrying an
//! `InferenceSource`. Adding a departure entry beside it would have made one
//! question two entries, and law 1's *what left this machine today* would have
//! counted that departure twice.
//!
//! So there is one entry, and it is the departure. A question answered
//! somewhere else **is** an egress and is kept as [`Happened::Left`] with
//! [`Why::Asking`]; a question answered here is [`Happened::AnsweredHere`],
//! which has no destination because there is nowhere for it to name. Nothing is
//! lost by it: [`Destination`] says everything an `InferenceSource` said — the
//! paired machine by name, the provider and the region it declared — and
//! `alo-egress` already maps one to the other in one place.
//!
//! Two things follow, and both are worth more than the entry they cost.
//! **Whether something caused egress is a variant rather than a calculation**,
//! so the record cannot hold an entry whose egress-ness has to be worked out
//! and could be worked out differently by the next reader. And **an answer from
//! somewhere else cannot be recorded without a departure**, because
//! [`Happened::Left`] is reachable only from an [`alo_egress::Departing`] —
//! see [`crate::departed`]. A record that could say *the answer came from a
//! provider* while saying *nothing left this machine* would be a record that
//! contradicts itself, and that is now not a record that can be written.
//!
//! **Numbers, not handles.** An approval and a grant are recorded by their
//! number rather than as an [`alo_capability::ProposalId`] or a
//! [`alo_capability::GrantId`]. A handle read back off a disk would be a handle
//! into a list that has moved on — a live thing pointing at something that may
//! no longer exist — and a record holds facts about the past, not references
//! into the present.

use alo_egress::{Destination, Why};
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
    /// A question was answered on this machine (ADR 0008).
    ///
    /// **What was asked is not kept, and never will be.** There is nowhere in
    /// this variant for it to go: it holds who asked and nothing else. A record
    /// that kept the questions would be a transcript of everything a person
    /// ever said to their machine, which is the thing this product exists not
    /// to be.
    ///
    /// A question answered anywhere else is [`Happened::Left`], because it is
    /// one — see this module's documentation.
    AnsweredHere {
        /// Which agent asked.
        agent: Line,
    },
    /// Something left this machine (law 1).
    ///
    /// Made only from an [`alo_egress::Departing`], which the indicator is the
    /// only maker of, so an egress that was never shown to anybody is not an
    /// entry that can be written. See [`crate::departed`].
    Left {
        /// Whose authority it left under.
        agent: Line,
        /// Where it went.
        destination: Destination,
        /// Why it left.
        why: Why,
    },
    /// The egress policy refused to let something leave, so nothing did.
    ///
    /// A refusal is a thing that happened, and this one is the organisation's
    /// rule doing exactly what it was set to do. Made only from an
    /// [`alo_egress::NotPermitted`], so a refusal cannot be recorded that the
    /// policy did not make.
    HeldBack {
        /// Whose authority it would have left under.
        agent: Line,
        /// Where it would have gone.
        destination: Destination,
        /// Why it would have left.
        why: Why,
        /// Why it was not permitted, in the policy's own words.
        refused: Line,
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
            | Self::AnsweredHere { agent }
            | Self::Left { agent, .. }
            | Self::HeldBack { agent, .. } => agent,
        }
    }

    /// What ran or would have run — absent when nothing ever became a call.
    #[must_use]
    pub fn what(&self) -> Option<&What> {
        match self {
            Self::Ran { what, .. } | Self::Stopped { what, .. } => Some(what),
            Self::TurnedAway { .. }
            | Self::AnsweredHere { .. }
            | Self::Left { .. }
            | Self::HeldBack { .. } => None,
        }
    }

    /// Whether the agent was stopped.
    ///
    /// All three refusals count, whether the call was well formed or not and
    /// whether it was a call at all: a security review asking what was refused
    /// wants the ones that never validated, and the egress the policy held
    /// back, as much as the ones the grants turned down.
    #[must_use]
    pub fn was_stopped(&self) -> bool {
        matches!(
            self,
            Self::Stopped { .. } | Self::TurnedAway { .. } | Self::HeldBack { .. }
        )
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
            Self::Ran { .. }
            | Self::TurnedAway { .. }
            | Self::AnsweredHere { .. }
            | Self::Left { .. }
            | Self::HeldBack { .. } => None,
        }
    }

    /// Why it was stopped, in words — from wherever in the journey the refusal
    /// came from.
    ///
    /// `None` when nothing was stopped, and when a person simply said no: "no"
    /// is the whole answer, and a system that recorded a reason would be a
    /// system that asked for one.
    #[must_use]
    pub fn why_stopped(&self) -> Option<&Line> {
        match self {
            Self::Stopped { how, .. } => how.why(),
            Self::TurnedAway { why, .. } | Self::HeldBack { refused: why, .. } => Some(why),
            Self::Ran { .. } | Self::AnsweredHere { .. } | Self::Left { .. } => None,
        }
    }

    /// Which approval this ran from, when it ran from one.
    #[must_use]
    pub fn from_approval(&self) -> Option<u64> {
        match self {
            Self::Ran { from_approval, .. } => *from_approval,
            Self::Stopped { .. }
            | Self::TurnedAway { .. }
            | Self::AnsweredHere { .. }
            | Self::Left { .. }
            | Self::HeldBack { .. } => None,
        }
    }

    /// Which grants this ran against.
    #[must_use]
    pub fn against(&self) -> &[u64] {
        match self {
            Self::Ran { against, .. } => against,
            Self::Stopped { .. }
            | Self::TurnedAway { .. }
            | Self::AnsweredHere { .. }
            | Self::Left { .. }
            | Self::HeldBack { .. } => &[],
        }
    }

    /// Where something went, or would have gone had the policy permitted it.
    #[must_use]
    pub fn destination(&self) -> Option<&Destination> {
        match self {
            Self::Left { destination, .. } | Self::HeldBack { destination, .. } => {
                Some(destination)
            }
            Self::Ran { .. }
            | Self::Stopped { .. }
            | Self::TurnedAway { .. }
            | Self::AnsweredHere { .. } => None,
        }
    }

    /// Why something was leaving, when this entry is about an egress.
    #[must_use]
    pub fn why_it_was_leaving(&self) -> Option<Why> {
        match self {
            Self::Left { why, .. } | Self::HeldBack { why, .. } => Some(*why),
            Self::Ran { .. }
            | Self::Stopped { .. }
            | Self::TurnedAway { .. }
            | Self::AnsweredHere { .. } => None,
        }
    }

    /// Whether this caused something to leave the machine.
    ///
    /// A variant rather than a calculation, which is the whole point of the
    /// shape this enum has: law 1 asks *what left this machine today* and gets
    /// one entry per departure, worked out once at the moment the policy
    /// permitted it rather than re-derived by whoever answers the question.
    #[must_use]
    pub fn caused_egress(&self) -> bool {
        matches!(self, Self::Left { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_calls::{archiving_march, listing_invoices, to_alo, to_the_studio};

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
    /// as in `alo-models` and `alo-egress`.
    #[test]
    fn an_answer_from_the_next_room_is_still_egress() {
        let here = Happened::AnsweredHere {
            agent: Line::of("@mail"),
        };
        assert!(!here.caused_egress());

        let next_room = Happened::Left {
            agent: Line::of("@mail"),
            destination: to_the_studio(),
            why: Why::Asking,
        };
        assert!(next_room.caused_egress());
        assert!(
            next_room
                .destination()
                .is_some_and(Destination::stays_in_the_building)
        );
        assert!(!ran().caused_egress());
    }

    /// **The decision this file turns on.** A question answered somewhere else
    /// is one entry, not two: the departure *is* where the answer came from, so
    /// law 1's question and ADR 0008's question are answered by the same entry
    /// and neither counts the other's.
    #[test]
    fn an_answer_from_somewhere_else_is_the_departure_it_caused_and_nothing_beside_it() {
        let asked = Happened::Left {
            agent: Line::of("@mail"),
            destination: to_alo(),
            why: Why::Asking,
        };
        assert!(asked.caused_egress());
        assert_eq!(asked.why_it_was_leaving(), Some(Why::Asking));
        assert_eq!(asked.destination(), Some(&to_alo()));

        // A question answered here has nowhere to name, and there is no field
        // in which it could name one.
        let here = Happened::AnsweredHere {
            agent: Line::of("@mail"),
        };
        assert_eq!(here.destination(), None);
        assert_eq!(here.why_it_was_leaving(), None);
        assert!(!here.caused_egress() && !here.was_stopped() && !here.ran());
    }

    /// **An egress the policy refused is a refusal, and nothing left.** It is
    /// findable as a refusal and it is not findable as egress — a record that
    /// counted it as a departure would be a record that says something left
    /// when nothing did.
    #[test]
    fn an_egress_the_policy_refused_is_a_refusal_and_not_a_departure() {
        let held = Happened::HeldBack {
            agent: Line::of("@files"),
            destination: to_alo(),
            why: Why::Sending,
            refused: Line::of("this machine is set to let nothing leave"),
        };
        assert!(held.was_stopped());
        assert!(!held.caused_egress());
        assert!(!held.ran());
        assert!(
            held.why_stopped()
                .is_some_and(|why| why.as_str().contains("nothing leave"))
        );
        assert_eq!(held.destination(), Some(&to_alo()));
        assert_eq!(held.why_it_was_leaving(), Some(Why::Sending));
    }

    /// Why something was stopped is one question however far it got, so the
    /// three refusals answer it in one place rather than in three.
    #[test]
    fn why_something_was_stopped_is_one_question_across_all_three_refusals() {
        let at_the_moment = Happened::Stopped {
            agent: Line::of("@files"),
            what: What::of(&archiving_march()),
            how: Stopped::AtTheMoment(Line::of("the grant has expired")),
        };
        assert!(
            at_the_moment
                .why_stopped()
                .is_some_and(|why| why.is("the grant has expired"))
        );

        let turned_away = Happened::TurnedAway {
            agent: Line::of("@files"),
            verb: Line::of("delete_everything"),
            why: Line::of("there is no verb called delete_everything"),
        };
        assert!(turned_away.why_stopped().is_some());

        // A person who says no is not asked to justify it, and something that
        // was not stopped has nothing to explain.
        let declined = Happened::Stopped {
            agent: Line::of("@files"),
            what: What::of(&archiving_march()),
            how: Stopped::ByThePerson,
        };
        assert_eq!(declined.why_stopped(), None);
        assert_eq!(ran().why_stopped(), None);
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
            Happened::AnsweredHere {
                agent: Line::of("@mail"),
            },
            Happened::Left {
                agent: Line::of("@mail"),
                destination: to_alo(),
                why: Why::Asking,
            },
            Happened::HeldBack {
                agent: Line::of("@files"),
                destination: to_alo(),
                why: Why::Sending,
                refused: Line::of("this machine is set to let nothing leave"),
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
