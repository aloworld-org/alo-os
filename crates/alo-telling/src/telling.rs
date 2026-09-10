//! What this machine has already said about not being able to reach a model.
//!
//! [`crate::ToldOnce`] is one telling. This is the thing that lives as long as
//! the session: it remembers which unavailabilities a person has been told
//! about, and it is the only door a failure can become a telling through.
//!
//! # It is one per machine, and it is deliberately not one per turn
//!
//! `alo_turn::Turning` is a turn and ends with it. A memory kept inside a turn
//! would be a memory that forgets between turns, which is the greyed-out panel
//! ADR 0009 refused, said once per turn instead of once per session — the same
//! sentence in front of somebody every time they ask anything, all afternoon.
//! Nor is it one per surface: two surfaces each saying a thing once is a thing
//! said twice, and the person who has two of them open is the person most
//! likely to be told the same thing twice in the same second.
//!
//! So it is held where the session is held, handed the failure at the moment a
//! turn ends, and it is the whole of what makes *says so once* a mechanism
//! rather than a habit.
//!
//! # The three promises, and what each is in the code
//!
//! - **The same unavailability told once is told once.** [`Telling::about`]
//!   consumes the failure. On a repeat it is dropped: nothing is worded, and
//!   there is no value left anywhere that a surface could word instead.
//! - **Telling it again takes something changing.** The source or the reason
//!   changing is a different [`crate::Unavailable`] and is not in the memory;
//!   the person asking again is [`crate::WhoAsked::ThePerson`], which is never
//!   suppressed, because an answer to a question somebody just asked is not a
//!   reminder.
//! - **Nothing is told until a turn actually failed.** There is no other door.
//!   [`Telling::about`] needs an `alo_answering::Failed`, which
//!   `alo_answering::Answering::did_not_answer` is the only thing that makes,
//!   and a fresh [`Telling`] holds nothing and says nothing.
//!
//! # What it forgets, and why forgetting is bounded rather than clever
//!
//! The memory holds [`AS MANY AS A SESSION REMEMBERS`](HOW_MANY_IT_REMEMBERS)
//! unavailabilities and drops the oldest when a new one arrives on a full one.
//! It has to be bounded — a service flapping between statuses could otherwise
//! grow it without limit, and an operating system with an unbounded list in it
//! has a way of ending badly that has nothing to do with nagging.
//!
//! The eviction is *oldest first* and that direction is the whole of the
//! argument. Forgetting an old telling costs at most one repeat of something
//! that happened a long time ago; refusing to remember a new one would suppress
//! a failure nobody has been told about, which is the acceptance's *a machine
//! that swallowed the second failure would hide the one that mattered*. A bound
//! may cost a repetition. It may never cost a telling.

use std::collections::VecDeque;

use alo_answering::Failed;

use crate::told_once::ToldOnce;
use crate::unavailable::Unavailable;
use crate::who_asked::WhoAsked;

/// How many unavailabilities one session remembers having told somebody about.
///
/// Larger than the number a real machine produces — a person's machine has one
/// local runtime and a small number of providers, and eight reasons between
/// them — and small enough that the list is nothing. It exists for the service
/// that flaps, not for the machine that works.
pub const HOW_MANY_IT_REMEMBERS: usize = 64;

/// What one call to [`Telling::about`] did.
///
/// Two answers, and the second is deliberately empty: there is no failure in
/// it, no sentence in it and nothing to draw. A variant carrying the failure
/// back would be a suppression a caller could step around by wording it
/// themselves, which is a suggestion rather than a guarantee.
#[derive(Debug, PartialEq, Eq)]
pub enum Tell {
    /// Nobody has been told this: put it in front of the person.
    Say(ToldOnce),
    /// It has been said, and nobody asked again. **Nothing is shown**, and the
    /// failure is gone — so there is nothing left to draw and nothing left to
    /// answer.
    SaidAlready,
}

impl Tell {
    /// The telling, if there is one to show.
    #[must_use]
    pub fn to_say(&self) -> Option<&ToldOnce> {
        match self {
            Self::Say(told) => Some(told),
            Self::SaidAlready => None,
        }
    }

    /// Whether this call put anything in front of anybody.
    #[must_use]
    pub fn was_said(&self) -> bool {
        matches!(self, Self::Say(_))
    }
}

/// What this machine has already told somebody about being unable to reach a
/// model, for as long as a session lasts.
///
/// One per machine; `crate` documentation is why it is not one per turn and not
/// one per surface. It holds no sentence and no failure — only which
/// unavailabilities have been told about — so there is nothing in it that could
/// be put back on a screen at a later moment, which is ADR 0009's *where it
/// happened* as a shape rather than as an instruction.
///
/// ```
/// use alo_answering::{Answering, WentWrong};
/// use alo_models::{InferenceSource, SourcePolicy};
/// use alo_telling::{Tell, Telling, WhoAsked};
///
/// /// A question put on this machine, which the runtime did not answer.
/// fn it_did_not_answer() -> alo_answering::Failed {
///     Answering::chosen(InferenceSource::ThisMachine, &SourcePolicy::Anywhere)
///         .expect("nothing forbids answering here")
///         .did_not_answer(WentWrong::NothingAnswered, &[], &SourcePolicy::Anywhere)
///         .expect("a runtime that is not running can happen here")
/// }
///
/// let mut telling = Telling::nothing_said_yet();
///
/// // A person asks, and is told.
/// assert!(telling.about(it_did_not_answer(), WhoAsked::ThePerson).was_said());
///
/// // The machine retries by itself, twice. It has said this already.
/// assert_eq!(telling.about(it_did_not_answer(), WhoAsked::TheMachine), Tell::SaidAlready);
/// assert_eq!(telling.about(it_did_not_answer(), WhoAsked::TheMachine), Tell::SaidAlready);
///
/// // The person asks again themselves, and is answered rather than ignored.
/// assert!(telling.about(it_did_not_answer(), WhoAsked::ThePerson).was_said());
/// ```
#[derive(Debug, Default)]
pub struct Telling {
    /// Every unavailability somebody has been told about, oldest first.
    ///
    /// Identities and nothing else: no sentence, no failure, no moment. See
    /// `unavailable.rs` for why there is no way to word one of these.
    already: VecDeque<Unavailable>,
}

impl Telling {
    /// Where every session starts: nothing has been said to anybody.
    #[must_use]
    pub fn nothing_said_yet() -> Self {
        Self {
            already: VecDeque::new(),
        }
    }

    /// How many unavailabilities this session remembers having told somebody
    /// about.
    ///
    /// A count and never the things themselves, for the reason at the top of
    /// this file: what is remembered exists to be compared, not to be shown.
    #[must_use]
    pub fn how_many_it_remembers(&self) -> usize {
        self.already.len()
    }

    /// Whether somebody has already been told about this one.
    ///
    /// Answering the question without asking it is what [`Telling::about`]
    /// does; this is for a caller that has to decide something *before* a
    /// question is put — whether to retry quietly, say — and it changes
    /// nothing.
    #[must_use]
    pub fn has_said(&self, unavailable: &Unavailable) -> bool {
        self.already.contains(unavailable)
    }

    /// A turn failed: what, if anything, to put in front of the person.
    ///
    /// Takes the failure, because a failure that has already been told about is
    /// **dropped here** rather than handed back. That is the difference between
    /// a guarantee and a recommendation: on [`Tell::SaidAlready`] there is no
    /// value left anywhere in the program that could be worded onto a screen.
    ///
    /// The person asking again is never suppressed. Everything else is
    /// suppressed exactly when this machine has said it before.
    pub fn about(&mut self, failed: Failed, who: WhoAsked) -> Tell {
        let unavailable = Unavailable::of(&failed);
        let said_before = self.has_said(&unavailable);
        if said_before && !who.is_a_person_waiting() {
            // Nothing is worded, and `failed` goes out of scope here: the
            // machine has said this, nobody asked, and it continues.
            return Tell::SaidAlready;
        }
        if !said_before {
            self.remember(unavailable);
        }
        Tell::Say(ToldOnce::about(failed))
    }

    /// Keep one more, forgetting the oldest if the memory is full.
    fn remember(&mut self, unavailable: Unavailable) {
        self.already.push_back(unavailable);
        while self.already.len() > HOW_MANY_IT_REMEMBERS {
            self.already.pop_front();
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{failing, here, hosted, in_english, paired};
    use alo_answering::WentWrong;
    use alo_models::{InferenceSource, Region};

    /// **A fresh session has said nothing**, and there is no door that produces
    /// a telling out of it: the only method that answers a [`Tell`] needs a
    /// failure.
    #[test]
    fn a_fresh_session_has_told_nobody_anything() {
        for telling in [Telling::default(), Telling::nothing_said_yet()] {
            assert_eq!(telling.how_many_it_remembers(), 0);
            assert!(!telling.has_said(&Unavailable::of(&failing(
                here(),
                WentWrong::NothingAnswered,
                &[]
            ))));
        }
    }

    /// **The same thing, said once.** Four background attempts against one
    /// empty account produce one sentence, which is the whole promise.
    #[test]
    fn the_same_unavailability_told_once_is_told_once() {
        let mut telling = Telling::nothing_said_yet();
        assert!(
            telling
                .about(
                    failing(hosted(), WentWrong::RanOut, &[]),
                    WhoAsked::ThePerson
                )
                .was_said()
        );
        for _ in 0..4 {
            assert_eq!(
                telling.about(
                    failing(hosted(), WentWrong::RanOut, &[]),
                    WhoAsked::TheMachine
                ),
                Tell::SaidAlready
            );
        }
        assert_eq!(telling.how_many_it_remembers(), 1);
    }

    /// **A different reason is a different telling**, and it is never
    /// suppressed — a machine that swallowed the second failure would hide the
    /// one that mattered.
    #[test]
    fn a_different_reason_in_the_same_place_is_said() {
        let mut telling = Telling::nothing_said_yet();
        assert!(
            telling
                .about(
                    failing(hosted(), WentWrong::RanOut, &[]),
                    WhoAsked::ThePerson
                )
                .was_said()
        );
        assert!(
            telling
                .about(
                    failing(hosted(), WentWrong::KeyNotAccepted, &[]),
                    WhoAsked::TheMachine
                )
                .was_said()
        );
        assert!(
            telling
                .about(
                    failing(hosted(), WentWrong::HavingTrouble(503), &[]),
                    WhoAsked::TheMachine
                )
                .was_said()
        );
        assert_eq!(telling.how_many_it_remembers(), 3);
    }

    /// **A different place is a different telling.** *Nothing answered on this
    /// machine* and *nothing was answered by alo, in the EU* are two facts
    /// about two halves of somebody's arrangements, and being told the first
    /// does not mean being told the second.
    #[test]
    fn the_same_reason_somewhere_else_is_said() {
        let mut telling = Telling::nothing_said_yet();
        assert!(
            telling
                .about(
                    failing(here(), WentWrong::NothingAnswered, &[]),
                    WhoAsked::ThePerson
                )
                .was_said()
        );
        for elsewhere in [hosted(), paired()] {
            assert!(
                telling
                    .about(
                        failing(elsewhere, WentWrong::NothingAnswered, &[]),
                        WhoAsked::TheMachine
                    )
                    .was_said()
            );
        }
        assert_eq!(telling.how_many_it_remembers(), 3);
    }

    /// **A person who asks again is answered**, however often they ask. Withholding
    /// it would be the key that silently does nothing, which is the worst
    /// outcome available — and it does not grow the memory, because it is the
    /// same unavailability.
    #[test]
    fn a_person_asking_again_is_told_again() {
        let mut telling = Telling::nothing_said_yet();
        for _ in 0..5 {
            assert!(
                telling
                    .about(
                        failing(hosted(), WentWrong::RanOut, &[]),
                        WhoAsked::ThePerson
                    )
                    .was_said()
            );
        }
        assert_eq!(telling.how_many_it_remembers(), 1);
    }

    /// **A suppressed failure leaves nothing behind.** There is no failure in
    /// [`Tell::SaidAlready`] and no telling on it, so a surface holding one has
    /// nothing to draw and nothing to word — which is what makes the
    /// suppression a mechanism rather than an instruction.
    #[test]
    fn a_suppressed_failure_leaves_nothing_to_show() {
        let mut telling = Telling::nothing_said_yet();
        drop(telling.about(
            failing(here(), WentWrong::TookTooLong, &[]),
            WhoAsked::ThePerson,
        ));

        let again = telling.about(
            failing(here(), WentWrong::TookTooLong, &[]),
            WhoAsked::TheMachine,
        );
        assert_eq!(again.to_say(), None);
        assert!(!again.was_said());
    }

    /// **The memory is bounded, and it forgets the oldest.** A service flapping
    /// between statuses cannot grow it without limit; what that costs is a
    /// repetition of something old, and never a new failure going unsaid.
    #[test]
    fn the_memory_is_bounded_and_forgets_the_oldest_first() {
        let mut telling = Telling::nothing_said_yet();
        let first = failing(here(), WentWrong::NothingAnswered, &[]);
        let remembered = Unavailable::of(&first);
        drop(telling.about(first, WhoAsked::ThePerson));

        // Enough different places to fill the memory past its bound. Each is a
        // provider of its own, which is the shape a flapping gateway takes.
        for which in 0..HOW_MANY_IT_REMEMBERS {
            let source = InferenceSource::Hosted {
                provider: format!("provider {which}"),
                region: Region::Unknown,
            };
            assert!(
                telling
                    .about(
                        failing(source, WentWrong::NothingAnswered, &[]),
                        WhoAsked::TheMachine
                    )
                    .was_said()
            );
        }

        assert_eq!(telling.how_many_it_remembers(), HOW_MANY_IT_REMEMBERS);
        // The oldest is what went, so the worst a full memory costs is that an
        // old telling is said a second time.
        assert!(!telling.has_said(&remembered));
        assert!(
            telling
                .about(
                    failing(here(), WentWrong::NothingAnswered, &[]),
                    WhoAsked::TheMachine
                )
                .was_said()
        );
    }

    /// The question can be asked without a failure to hand, which is what a
    /// caller deciding whether to retry quietly needs — and asking it changes
    /// nothing.
    #[test]
    fn asking_what_has_been_said_changes_nothing() {
        let mut telling = Telling::nothing_said_yet();
        let unavailable = Unavailable::of(&failing(hosted(), WentWrong::RanOut, &[]));
        assert!(!telling.has_said(&unavailable));

        drop(telling.about(
            failing(hosted(), WentWrong::RanOut, &[]),
            WhoAsked::ThePerson,
        ));
        assert!(telling.has_said(&unavailable));
        assert!(telling.has_said(&unavailable));
        assert_eq!(telling.how_many_it_remembers(), 1);
    }

    /// And a telling that is shown is the whole telling: the memory holds the
    /// identity, and what reaches a person still comes off the failure.
    #[test]
    fn what_is_remembered_is_an_identity_and_what_is_shown_is_a_failure() {
        let mut telling = Telling::nothing_said_yet();
        let tell = telling.about(
            failing(hosted(), WentWrong::RanOut, &[]),
            WhoAsked::ThePerson,
        );
        let told = tell.to_say().unwrap();
        assert_eq!(told.unavailable().why(), WentWrong::RanOut);
        assert!(!told.what_happened(&in_english()).is_a_bug());
    }
}
