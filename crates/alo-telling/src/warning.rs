//! What this machine has already said about weights larger than its memory.
//!
//! [`crate::WarnedOnce`] is one warning. This is the thing that lives as long
//! as the session: it remembers which weights a person has been warned about
//! on this machine, and it is the only door a cost can become a warning
//! through.
//!
//! `docs/features.md`: *a model too large for the memory in this laptop is
//! said so plainly, **once** — and then run anyway if that is what somebody
//! asked for.* `alo_models::Cost` is the *plainly*: a value with no `Err` in
//! it. `alo_models::Weights::lines` is the *where* — at the moment the weights
//! are added. What had never been written is the *once* across a session: a
//! person who chose forty gigabytes of their own weights on a sixteen gigabyte
//! laptop would have read the same sentence at every question, all afternoon,
//! which is the machine arguing with them about their own hardware.
//!
//! # It is beside [`crate::Telling`], not inside it
//!
//! Two memories with one shape, one file each. A telling is about a question
//! that was really put somewhere and really was not answered; a warning is
//! about weights that were really costed on this machine and fit or did not.
//! They are made from different things, remembered as different identities,
//! and worded from different crates — and a single memory holding both would
//! be a file with two reasons to change. What they share is the rule, and the
//! rule is [`crate::WhoAsked`]'s: **said when this machine has something to
//! say that it has not said, or when somebody just asked.**
//!
//! # The three promises, and what each is in the code
//!
//! - **The same weights on the same machine warned once are warned once.**
//!   [`Warning::about`] answers [`Warn::SaidAlready`] the second time, and
//!   there is no value in it to word.
//! - **Warning again takes something changing.** Other weights, or the same
//!   weights on a machine with a different amount of memory, are a different
//!   [`crate::TooLarge`] and are not in the memory; the person asking again
//!   is [`crate::WhoAsked::ThePerson`], which is never suppressed.
//! - **Nothing is warned about until weights were costed and did not fit.**
//!   Weights that fit are [`Warn::Fits`], with nothing in it and nothing
//!   remembered.
//!
//! # And one thing it can never do
//!
//! Refuse. There is no `Err` here and no `bool` meaning may-not-run: the door
//! answers what to show, and a caller holding a [`Warn`] has nothing to put an
//! `if` in front of except whether to draw two lines. The weights run whichever
//! answer this gives, because the answer was never about that.

use std::collections::VecDeque;

use alo_models::Weights;

use crate::telling::HOW_MANY_IT_REMEMBERS;
use crate::too_large::TooLarge;
use crate::warned_once::WarnedOnce;
use crate::who_asked::WhoAsked;

/// What one call to [`Warning::about`] did.
///
/// Three answers, and two of them are deliberately empty: there is no cost in
/// them, no sentence and nothing to draw. A variant carrying the cost back
/// would be a suppression a caller could step around by wording it themselves.
#[derive(Debug, PartialEq)]
pub enum Warn {
    /// Nobody has been warned about this: put it in front of the person.
    Say(WarnedOnce),
    /// It has been said, and nobody asked again. **Nothing is shown.**
    SaidAlready,
    /// The weights fit in this machine's memory. There is nothing to warn
    /// about, and nothing was remembered.
    Fits,
}

impl Warn {
    /// The warning, if there is one to show.
    #[must_use]
    pub fn to_say(&self) -> Option<&WarnedOnce> {
        match self {
            Self::Say(warned) => Some(warned),
            Self::SaidAlready | Self::Fits => None,
        }
    }

    /// Whether this call put anything in front of anybody.
    #[must_use]
    pub fn was_said(&self) -> bool {
        matches!(self, Self::Say(_))
    }
}

/// What this machine has already warned somebody about, for as long as a
/// session lasts.
///
/// One per machine, beside [`crate::Telling`]. It holds no sentence and no
/// cost — only which warnings have been given — so there is nothing in it that
/// could be put back on a screen at a later moment.
///
/// ```
/// use alo_models::{Weights, costing::GIGABYTE};
/// use alo_telling::{Warn, Warning, WhoAsked};
///
/// let theirs = Weights::checked("their-own-70b", 40 * GIGABYTE).expect("a name");
/// let mut warning = Warning::nothing_said_yet();
///
/// // The person chooses them on a sixteen gigabyte machine, and is told.
/// assert!(warning.about(&theirs, 16.0, WhoAsked::ThePerson).was_said());
///
/// // Every question afterwards costs them again, and says nothing.
/// for _ in 0..20 {
///     assert_eq!(warning.about(&theirs, 16.0, WhoAsked::TheMachine), Warn::SaidAlready);
/// }
///
/// // Weights that fit were never anything to say.
/// let small = Weights::checked("small", 4 * GIGABYTE).expect("a name");
/// assert_eq!(warning.about(&small, 16.0, WhoAsked::TheMachine), Warn::Fits);
/// ```
#[derive(Debug, Default)]
pub struct Warning {
    /// Every warning somebody has been given, oldest first. Identities and
    /// nothing else.
    already: VecDeque<TooLarge>,
}

impl Warning {
    /// Where every session starts: nobody has been warned about anything.
    #[must_use]
    pub fn nothing_said_yet() -> Self {
        Self {
            already: VecDeque::new(),
        }
    }

    /// How many warnings this session remembers having given.
    #[must_use]
    pub fn how_many_it_remembers(&self) -> usize {
        self.already.len()
    }

    /// Whether somebody has already been warned about this one.
    ///
    /// For a caller deciding something before a cost is put in front of
    /// anybody; asking changes nothing.
    #[must_use]
    pub fn has_said(&self, too_large: &TooLarge) -> bool {
        self.already.contains(too_large)
    }

    /// Weights were costed on this machine: what, if anything, to put in
    /// front of the person.
    ///
    /// [`Warn::Fits`] when they fit, and nothing is remembered. Otherwise the
    /// person asking is never suppressed, and the machine asking is suppressed
    /// exactly when this machine has said it before.
    ///
    /// Takes the weights by reference because a warning does not spend them:
    /// they are about to run, whichever answer this gives.
    pub fn about(&mut self, weights: &Weights, machine_gb: f32, who: WhoAsked) -> Warn {
        let Some(too_large) = TooLarge::of(weights, machine_gb) else {
            return Warn::Fits;
        };
        let said_before = self.has_said(&too_large);
        if said_before && !who.is_a_person_waiting() {
            return Warn::SaidAlready;
        }
        let cost = too_large.cost();
        if !said_before {
            self.remember(too_large);
        }
        Warn::Say(WarnedOnce::about(weights.clone(), cost))
    }

    /// Keep one more, forgetting the oldest if the memory is full — bounded
    /// for `telling.rs`'s reason, and in that direction: a bound may cost a
    /// repetition and may never cost a warning.
    fn remember(&mut self, too_large: TooLarge) {
        self.already.push_back(too_large);
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
    use crate::testing::{in_english, weights_of};
    use alo_models::costing::GIGABYTE;

    /// **A fresh session has warned nobody about anything.**
    #[test]
    fn a_fresh_session_has_warned_nobody_about_anything() {
        for warning in [Warning::default(), Warning::nothing_said_yet()] {
            assert_eq!(warning.how_many_it_remembers(), 0);
            let theirs = TooLarge::of(&weights_of("theirs", 40 * GIGABYTE), 16.0).unwrap();
            assert!(!warning.has_said(&theirs));
        }
    }

    /// **The same weights on the same machine, warned once, are warned once.**
    /// Forty questions against forty gigabytes on a sixteen gigabyte laptop
    /// produce one sentence.
    #[test]
    fn the_same_weights_on_the_same_machine_are_warned_about_once() {
        let mut warning = Warning::nothing_said_yet();
        let theirs = weights_of("theirs", 40 * GIGABYTE);
        assert!(warning.about(&theirs, 16.0, WhoAsked::ThePerson).was_said());
        for _ in 0..40 {
            assert_eq!(
                warning.about(&theirs, 16.0, WhoAsked::TheMachine),
                Warn::SaidAlready
            );
        }
        assert_eq!(warning.how_many_it_remembers(), 1);
    }

    /// **Weights that fit are never anything to say**, and are not
    /// remembered as having been said.
    #[test]
    fn weights_that_fit_are_nothing_to_say_and_nothing_to_remember() {
        let mut warning = Warning::nothing_said_yet();
        let small = weights_of("small", 4 * GIGABYTE);
        for who in [WhoAsked::ThePerson, WhoAsked::TheMachine] {
            let warn = warning.about(&small, 16.0, who);
            assert_eq!(warn, Warn::Fits);
            assert!(!warn.was_said());
            assert_eq!(warn.to_say(), None);
        }
        assert_eq!(warning.how_many_it_remembers(), 0);
    }

    /// **Other weights are a different warning, and so is the same file on a
    /// machine with a different amount of memory** — neither is suppressed by
    /// the first.
    #[test]
    fn other_weights_or_another_machine_are_warned_about_too() {
        let mut warning = Warning::nothing_said_yet();
        let theirs = weights_of("theirs", 40 * GIGABYTE);
        assert!(warning.about(&theirs, 16.0, WhoAsked::ThePerson).was_said());
        assert!(
            warning
                .about(
                    &weights_of("others", 40 * GIGABYTE),
                    16.0,
                    WhoAsked::TheMachine
                )
                .was_said()
        );
        assert!(
            warning
                .about(&theirs, 32.0, WhoAsked::TheMachine)
                .was_said()
        );
        assert_eq!(warning.how_many_it_remembers(), 3);
    }

    /// **A person who asks again is told again**, however often, and it does
    /// not grow the memory.
    #[test]
    fn a_person_asking_again_is_warned_again() {
        let mut warning = Warning::nothing_said_yet();
        let theirs = weights_of("theirs", 40 * GIGABYTE);
        for _ in 0..5 {
            assert!(warning.about(&theirs, 16.0, WhoAsked::ThePerson).was_said());
        }
        assert_eq!(warning.how_many_it_remembers(), 1);
    }

    /// **A suppressed warning leaves nothing behind**: no cost, no sentence,
    /// nothing to draw.
    #[test]
    fn a_suppressed_warning_leaves_nothing_to_show() {
        let mut warning = Warning::nothing_said_yet();
        let theirs = weights_of("theirs", 40 * GIGABYTE);
        drop(warning.about(&theirs, 16.0, WhoAsked::ThePerson));
        let again = warning.about(&theirs, 16.0, WhoAsked::TheMachine);
        assert_eq!(again.to_say(), None);
        assert!(!again.was_said());
    }

    /// **Nothing here refuses**, whatever the numbers: every answer is about
    /// what to show, and a warning that is shown is the whole warning.
    #[test]
    fn nothing_here_refuses_and_what_is_shown_is_the_whole_warning() {
        let mut warning = Warning::nothing_said_yet();
        let enormous = weights_of("enormous", 400 * GIGABYTE);
        let warn = warning.about(&enormous, 0.0, WhoAsked::ThePerson);
        let warned = warn.to_say().unwrap();
        assert_eq!(warned.weights().id, "enormous");
        for line in warned.lines(&in_english()) {
            assert!(!line.is_a_bug(), "{line}");
        }
        // A machine that reported no memory at all warns, which is the whole
        // of what a warning can do — and it is still not a refusal.
        assert!(warn.was_said());
    }

    /// **The memory is bounded, and it forgets the oldest**: what a full
    /// memory costs is a repetition of something old, never a warning going
    /// unsaid.
    #[test]
    fn the_memory_is_bounded_and_forgets_the_oldest_first() {
        let mut warning = Warning::nothing_said_yet();
        let first = weights_of("first", 40 * GIGABYTE);
        let remembered = TooLarge::of(&first, 16.0).unwrap();
        drop(warning.about(&first, 16.0, WhoAsked::ThePerson));

        for which in 0..HOW_MANY_IT_REMEMBERS {
            let weights = weights_of(&format!("weights {which}"), 40 * GIGABYTE);
            assert!(
                warning
                    .about(&weights, 16.0, WhoAsked::TheMachine)
                    .was_said()
            );
        }

        assert_eq!(warning.how_many_it_remembers(), HOW_MANY_IT_REMEMBERS);
        assert!(!warning.has_said(&remembered));
        assert!(warning.about(&first, 16.0, WhoAsked::TheMachine).was_said());
    }
}
