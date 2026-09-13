//! The door that reaches another machine on the local network: a verb, or a
//! question about one, put to a paired machine from inside a turn.
//!
//! [`Turning::asking`] is the door that reaches off this machine with a
//! question, and this is the same door for what crosses to a paired machine
//! — cut on the same line law 1 draws, and shaped by ADR 0020 in the same
//! way. A turn does not decide what crosses or how: `alo-corridor` builds
//! the verb, makes the proof and dials, and a turn cannot name that crate
//! because that crate names this one. What a turn adds around it is what it
//! adds around a question — the order, the boundary, the record — and it
//! adds them by taking the crossing as a closure and running it from inside
//! a boundary permitting the one address discovery measured.
//!
//! # What is written down, and when
//!
//! Before the caller is told anything, as everywhere in this crate:
//!
//! | What the closure came back with | What is written |
//! |---|---|
//! | [`Departed::Left`] — it left, under a departure | `Entry::left`, whether or not an answer came back |
//! | [`Departed::HeldBack`] — the egress rule refused it | `Entry::held_back`, from the rule's refusal |
//! | [`Departed::NothingLeft`] — nothing was addressed | nothing, for `crate::unanswered`'s reason |
//! | the boundary could not be imposed | `Entry::not_bounded`, the machine's own refusal |
//!
//! *Whether or not an answer came back* is the line the plan asks for: a
//! verb that crossed and met a closed door, or no door, is a verb that left
//! this machine, and a record of only the answered ones would report a
//! quieter day than the machine had.
//!
//! # Resolved before the boundary, put from inside it
//!
//! ADR 0020. The address handed in is the one discovery measured
//! (`alo_nearby::Found::where_it_answers`), so there is nothing to resolve
//! and nothing a bounded thread would need a name server for; it is
//! registered as the one destination the boundary permits, and the closure
//! runs inside that boundary. A boundary that cannot be imposed is ADR
//! 0015's rule rather than a smaller question: nothing crossed, and the
//! machine's refusal is written down and handed back as
//! [`NoAnswer::NotBounded`].

use std::net::SocketAddr;
use std::time::SystemTime;

use alo_egress::{Departing, EgressPolicy, Indicator, NotPermitted};
use alo_record::Entry;

use crate::places::Places;
use crate::turning::Turning;
use crate::unanswered::NoAnswer;
use crate::unbounded::NoBoundary;

/// What a crossing came back with, as the closure hands it to the turn.
///
/// The departure is handed out of whatever carried it, so the turn can
/// write it down and take it off the indicator; what came back is whatever
/// the caller wants handed back to it, refusals included.
#[derive(Debug)]
pub enum Departed<T> {
    /// Something left under this departure, and this is what came of it —
    /// an answer, or the word a door said, or nothing at all.
    Left {
        /// The departure it left under, made by the indicator.
        departing: Departing,
        /// What came back, handed to the caller once the departure is
        /// written down.
        back: T,
    },
    /// The egress rule in force held it back before anything was dialled.
    HeldBack(NotPermitted),
    /// Nothing was addressed and nothing left: the machine is not paired
    /// with, or the destination could not be shown. Handed back unwritten.
    NothingLeft(T),
}

impl Turning<'_, '_> {
    /// Cross to a paired machine at `to`, from inside a boundary permitting
    /// that address and no other, and write the departure down.
    ///
    /// `put` is the crossing itself: it is handed the egress rule in force
    /// and the indicator, makes its departure on the indicator before it
    /// dials, and answers with what it came back with. It runs once, inside
    /// the boundary, or not at all.
    ///
    /// # Errors
    /// [`NoAnswer::TurnClosed`] on a turn that has stopped keeping evidence;
    /// [`NoAnswer::HeldBack`] when the rule refused it, written down as
    /// such; [`NoAnswer::NotBounded`] when no boundary could be imposed, so
    /// nothing crossed; [`NoAnswer::NotRecorded`] when the record could not
    /// take the entry, saying whether it left first. What `put` answered is
    /// handed back whole otherwise, refusals of its own included.
    pub fn crossing<T>(
        &mut self,
        to: SocketAddr,
        places: &Places<'_>,
        put: impl FnOnce(&EgressPolicy, &mut Indicator, SystemTime) -> Departed<T>,
        now: SystemTime,
    ) -> Result<T, NoAnswer> {
        if self.is_closed() {
            return Err(NoAnswer::TurnClosed);
        }
        let agent = self.grantee().clone();
        let policy = EgressPolicy::from(places.policy());
        // Registered, not resolved: this is what discovery measured.
        let registered = [to];
        let (bounding, indicator) = self.machine().bounding_and_indicator();
        let mut once = Some(put);
        let mut outcome = None;
        let bounded = bounding.carrying_out_a_departure(&registered, &mut || {
            if let Some(put) = once.take() {
                outcome = Some(put(&policy, indicator, now));
            }
        });
        match (bounded, outcome) {
            (Ok(()), Some(Departed::Left { departing, back })) => {
                let kept = self.keeping(Entry::left(&departing));
                self.machine().indicator().ended(departing);
                match kept {
                    Ok(()) => Ok(back),
                    Err(why) => Err(NoAnswer::NotRecorded {
                        why,
                        after_it_left: true,
                    }),
                }
            }
            (Ok(()), Some(Departed::HeldBack(refused))) => {
                let strings = self.machine().strings();
                let entry = Entry::held_back(&refused, strings, now);
                match self.keeping(entry) {
                    Ok(()) => Err(NoAnswer::HeldBack(refused)),
                    Err(why) => Err(NoAnswer::NotRecorded {
                        why,
                        after_it_left: false,
                    }),
                }
            }
            (Ok(()), Some(Departed::NothingLeft(back))) => Ok(back),
            (Err(why), _) => Err(self.nothing_was_bounded(why, &agent, now)),
            (Ok(()), None) => Err(self.nothing_was_bounded(
                NoBoundary::because(
                    "the boundary was imposed and nothing was put inside it".to_owned(),
                ),
                &agent,
                now,
            )),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::net::{Ipv4Addr, SocketAddr};

    use alo_capability::{Grantee, Grants};
    use alo_context::Context;
    use alo_egress::{Destination, EgressPolicy, Indicator, Leaving, Why};
    use alo_files::OnThisMachine;
    use alo_models::SourcePolicy;
    use alo_record::{Asking, Happened, Record};

    use super::Departed;
    use crate::machine::Machine;
    use crate::places::Places;
    use crate::testing::{NoBoundaryAtAll, NothingIsBounded, hour, in_english, noon};
    use crate::turning::Turning;
    use crate::unanswered::NoAnswer;

    /// Where the studio machine was found.
    fn the_studio() -> SocketAddr {
        SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 7_610)
    }

    /// One turn on a machine with this record, this indicator and this
    /// boundary, ending when the closure is done.
    fn on_a_machine<T>(
        record: &mut Record,
        indicator: &mut Indicator,
        bounding: &mut dyn crate::Bounding,
        doing: impl FnOnce(&mut Turning<'_, '_>) -> T,
    ) -> T {
        let strings = in_english();
        let mut machine =
            Machine::carrying_out_file_verbs(&strings, &OnThisMachine, bounding, indicator, record)
                .unwrap();
        let mut grants = Grants::default();
        let mut turning = Turning::beginning(
            Context::at_invocation(noon()),
            "@files",
            hour(),
            &mut grants,
            &mut machine,
        )
        .unwrap();
        let outcome = doing(&mut turning);
        assert!(!turning.ending(&mut grants));
        outcome
    }

    /// A crossing as `alo-corridor` makes one: the departure first, on the
    /// indicator, and then whatever came back.
    fn crossing_that_comes_back<T>(
        back: T,
    ) -> impl FnOnce(&EgressPolicy, &mut Indicator, std::time::SystemTime) -> Departed<T> {
        move |policy, indicator, now| {
            let leaving = Leaving::because(
                &Grantee::named("@files"),
                Why::Sending,
                Destination::paired("the studio machine").unwrap(),
            );
            match indicator.beginning(policy, leaving, now) {
                Ok(departing) => Departed::Left { departing, back },
                Err(refused) => Departed::HeldBack(refused),
            }
        }
    }

    /// How many entries say something left, or was held back.
    fn how_many(record: &Record, left: bool) -> usize {
        record
            .answering(&Asking::anything())
            .filter(|entry| match entry.happened() {
                Happened::Left { .. } => left,
                Happened::HeldBack { .. } => !left,
                _ => false,
            })
            .count()
    }

    /// **The departure is written down whether or not an answer came back,
    /// and the indicator is quiet afterwards.** Two crossings: one that
    /// came back with an answer, one that came back with a door's word.
    #[test]
    fn what_crossed_is_written_down_as_left_whether_or_not_an_answer_came_back() {
        let mut record = Record::default();
        let mut indicator = Indicator::default();
        let places = Places::under(&SourcePolicy::Anywhere);
        let (answered, refused) = on_a_machine(
            &mut record,
            &mut indicator,
            &mut NothingIsBounded,
            |turning| {
                let answered = turning
                    .crossing(
                        the_studio(),
                        &places,
                        crossing_that_comes_back(Ok::<&str, &str>("a listing")),
                        noon(),
                    )
                    .unwrap();
                let refused = turning
                    .crossing(
                        the_studio(),
                        &places,
                        crossing_that_comes_back(Err::<&str, &str>("not-granted-there")),
                        noon(),
                    )
                    .unwrap();
                (answered, refused)
            },
        );
        assert_eq!(answered, Ok("a listing"));
        assert_eq!(refused, Err("not-granted-there"));
        assert_eq!(how_many(&record, true), 2);
        assert_eq!(how_many(&record, false), 0);
        assert!(indicator.is_quiet());
    }

    /// **A rule that says nothing leaves holds the crossing back**, written
    /// down as held back, with nothing on the indicator.
    #[test]
    fn a_rule_that_says_nothing_leaves_holds_the_crossing_back_and_writes_it_down() {
        let mut record = Record::default();
        let mut indicator = Indicator::default();
        let places = Places::under(&SourcePolicy::ThisMachineOnly);
        let held = on_a_machine(
            &mut record,
            &mut indicator,
            &mut NothingIsBounded,
            |turning| {
                turning.crossing(
                    the_studio(),
                    &places,
                    crossing_that_comes_back(Ok::<&str, &str>("never")),
                    noon(),
                )
            },
        )
        .unwrap_err();
        assert!(matches!(held, NoAnswer::HeldBack(_)), "{held:?}");
        assert!(held.nothing_left());
        assert_eq!(how_many(&record, true), 0);
        assert_eq!(how_many(&record, false), 1);
        assert!(indicator.is_quiet());
    }

    /// **Nothing addressed, nothing written**: a crossing that never reached
    /// the indicator is handed back as it was, with the record unchanged.
    #[test]
    fn a_crossing_that_addressed_nothing_is_handed_back_unwritten() {
        let mut record = Record::default();
        let mut indicator = Indicator::default();
        let places = Places::under(&SourcePolicy::Anywhere);
        let back = on_a_machine(
            &mut record,
            &mut indicator,
            &mut NothingIsBounded,
            |turning| {
                turning.crossing(
                    the_studio(),
                    &places,
                    |_, _, _| Departed::NothingLeft("not-paired-with-it"),
                    noon(),
                )
            },
        )
        .unwrap();
        assert_eq!(back, "not-paired-with-it");
        assert_eq!(record.answering(&Asking::anything()).count(), 0);
        assert!(indicator.is_quiet());
    }

    /// **A boundary that cannot be imposed means nothing crossed**, and the
    /// machine's own refusal is written down and handed back.
    #[test]
    fn a_crossing_that_could_not_be_bounded_crosses_nothing_and_says_so() {
        let mut record = Record::default();
        let mut indicator = Indicator::default();
        let places = Places::under(&SourcePolicy::Anywhere);
        let refused = on_a_machine(
            &mut record,
            &mut indicator,
            &mut NoBoundaryAtAll::kernel_would_not_take_it(),
            |turning| {
                turning.crossing(
                    the_studio(),
                    &places,
                    crossing_that_comes_back(Ok::<&str, &str>("never")),
                    noon(),
                )
            },
        )
        .unwrap_err();
        assert!(matches!(refused, NoAnswer::NotBounded(_)), "{refused:?}");
        assert!(refused.nothing_left());
        assert_eq!(how_many(&record, true), 0);
        assert!(
            record
                .answering(&Asking::anything())
                .any(|entry| matches!(entry.happened(), Happened::NotBounded { .. }))
        );
        assert!(indicator.is_quiet());
    }
}
