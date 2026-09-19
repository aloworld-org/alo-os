//! A machine with no way out at all says so once.
//!
//! ADR 0009's rule, in the shape `alo-telling` already gave it for a model that
//! could not be reached: **say it once, where it happened, and continue.** The
//! same failure is available here and is worse, because a check has two
//! occasions — a person asking, and every start — so a machine on a network
//! with no route out would repeat the same line at every asking, forever, for a
//! situation the person cannot do anything about and already knows about.
//!
//! # Only *no way out* is remembered
//!
//! The other four refusals are about the far end rather than about this
//! machine, and each of them is news: a place that answered yesterday and
//! refuses today is something a person may want to know about. *There is no
//! road out of this machine* is the one that is true for hours at a time and
//! unchanged by asking again.
//!
//! # The refusal is consumed rather than returned
//!
//! [`SaidOnce::what_to_say`] takes the answer **by value** and a suppressed
//! telling hands back [`Say::SaidAlready`], which carries nothing. That is the
//! whole design, and it is `alo_telling::Telling`'s argument: handing the
//! refusal back and asking the caller not to word it makes suppression a
//! recommendation, and a surface that worded it anyway would break the promise
//! while passing every test. **On a suppressed telling there is no value left
//! anywhere in the program that a sentence could be drawn from.**
//!
//! # Anything else forgets it
//!
//! A check that got an answer — an offer, an up-to-date, or any of the four
//! refusals that are about the far end — means there *is* a road out, so the
//! next time there is not, the person is told again. A memory that never
//! forgot would say it once in the life of the machine, which is silence
//! wearing the promise's clothes.

use crate::found::Found;
use crate::refusing::NoAnswer;

/// What to put in front of a person about one check.
#[derive(Debug, PartialEq, Eq)]
pub enum Say {
    /// This, in the person's own language.
    This(NoAnswer),
    /// Nothing: they have already been told, and nothing has changed since.
    ///
    /// Carries nothing, on purpose — there is no refusal in here for a surface
    /// to word anyway.
    SaidAlready,
}

/// Whether this machine has already said it has no way out.
///
/// For as long as whatever holds it lasts, which is a session or a service:
/// nothing here is written to a disk, because a machine that remembered across
/// restarts would stay silent about a network that was down last week.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SaidOnce {
    /// Whether it has been said and nothing has answered since.
    said: bool,
}

impl SaidOnce {
    /// Nothing said yet.
    #[must_use]
    pub const fn new() -> Self {
        Self { said: false }
    }

    /// What to say about this answer.
    ///
    /// # Errors
    /// [`Say`] — either the refusal to word, or that it has been said already.
    pub fn what_to_say(&mut self, answer: Result<Found, NoAnswer>) -> Result<Found, Say> {
        match answer {
            Ok(found) => {
                self.said = false;
                Ok(found)
            }
            Err(NoAnswer::NoWayOut) if self.said => Err(Say::SaidAlready),
            Err(NoAnswer::NoWayOut) => {
                self.said = true;
                Err(Say::This(NoAnswer::NoWayOut))
            }
            Err(other) => {
                self.said = false;
                Err(Say::This(other))
            }
        }
    }

    /// Whether this machine is currently holding its tongue about having no
    /// way out.
    #[must_use]
    pub const fn has_said_it(&self) -> bool {
        self.said
    }
}

#[cfg(test)]
#[expect(
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::because::Because;
    use crate::testing::{a_moment, build};
    use alo_keeping_up::Standing;

    fn an_answer() -> Found {
        Found::of(
            build("aa"),
            Standing::UpToDate,
            Because::ThePersonAsked,
            a_moment(),
        )
    }

    /// **Once, and then not again.** Three checks with no road out produce one
    /// sentence.
    #[test]
    fn a_machine_with_no_way_out_says_so_once() {
        let mut said = SaidOnce::new();
        assert_eq!(
            said.what_to_say(Err(NoAnswer::NoWayOut)),
            Err(Say::This(NoAnswer::NoWayOut))
        );
        for _ in 0..2 {
            assert_eq!(
                said.what_to_say(Err(NoAnswer::NoWayOut)),
                Err(Say::SaidAlready)
            );
        }
        assert!(said.has_said_it());
    }

    /// **An answer forgets it**, so the next time the road is gone the person
    /// is told again rather than being told once in the life of the machine.
    #[test]
    fn a_check_that_was_answered_forgets_it() {
        let mut said = SaidOnce::new();
        assert!(said.what_to_say(Err(NoAnswer::NoWayOut)).is_err());
        assert!(said.what_to_say(Ok(an_answer())).is_ok());
        assert!(!said.has_said_it());
        assert_eq!(
            said.what_to_say(Err(NoAnswer::NoWayOut)),
            Err(Say::This(NoAnswer::NoWayOut))
        );
    }

    /// **Only *no way out* is held back.** The other four are about the far
    /// end, they are news, and a machine that swallowed the second of them
    /// would hide the one that mattered.
    #[test]
    fn every_other_refusal_is_said_every_time() {
        for refusal in NoAnswer::EVERY {
            if refusal == NoAnswer::NoWayOut {
                continue;
            }
            let mut said = SaidOnce::new();
            for _ in 0..3 {
                assert_eq!(
                    said.what_to_say(Err(refusal)),
                    Err(Say::This(refusal)),
                    "{refusal:?} was held back"
                );
            }
            assert!(!said.has_said_it());
        }
    }

    /// **A refusal about the far end also forgets an earlier silence**, because
    /// answering at all means there is a road out.
    #[test]
    fn a_far_ends_refusal_means_there_is_a_road_out_after_all() {
        let mut said = SaidOnce::new();
        assert!(said.what_to_say(Err(NoAnswer::NoWayOut)).is_err());
        assert_eq!(
            said.what_to_say(Err(NoAnswer::ItRefused)),
            Err(Say::This(NoAnswer::ItRefused))
        );
        assert!(!said.has_said_it());
    }

    /// **A suppressed telling leaves nothing behind to word.** There is no
    /// refusal inside `SaidAlready` for a surface to reach for.
    #[test]
    fn a_suppressed_telling_carries_nothing() {
        let mut said = SaidOnce::new();
        assert!(said.what_to_say(Err(NoAnswer::NoWayOut)).is_err());
        let Err(Say::SaidAlready) = said.what_to_say(Err(NoAnswer::NoWayOut)) else {
            panic!("a repeat was not suppressed");
        };
    }
}
