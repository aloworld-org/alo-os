//! The appearance verbs, declared.
//!
//! `docs/features.md` promises at v0.5 that a person can **ask for it** — *use
//! dark after six* — *the same propose-then-approve as any other change, because
//! personalisation is exactly the low-stakes place people first learn to trust
//! the agent*. This is the road from those words to a change, and it is a
//! declaration rather than a new way to set anything: both verbs end at
//! [`crate::changes::Changes::follow`], which is the road a settings panel
//! writes through.
//!
//! # Two verbs, and why not more
//!
//! **What light and dark are is already decided and tested** — [`Scheme`],
//! [`Schedule`] and [`Following`], with the time passed in rather than read, so
//! a panel previewing a schedule and the compositor obeying it cannot disagree.
//! What was missing is only the asking. So the two verbs are the two shapes
//! [`Following`] has:
//!
//! | Verb | Argument | Ends at |
//! |---|---|---|
//! | `set_the_scheme` | one of light or dark | [`Following::Always`] |
//! | `follow_the_clock` | two hours | [`Following::TheClock`] |
//!
//! The background, the accent and the text size are deliberately not here. Each
//! is a value this crate defines and each would be one more verb of the same
//! shape; shipping two that work entirely beats six that need a second pass.
//!
//! # What a model may and may not say
//!
//! *Make the surface warmer* is a sentence a model reads, not an argument it
//! sends. Every argument here is a choice the verb wrote down or a whole number
//! in a stated range, so the words a person used never reach anything that acts
//! on them — `docs/contracts/agent-verbs.md`'s first rule. A model that has
//! understood *use dark after six* sends `18`, and the sentence a person
//! approves is built from that `18` rather than from what was typed.
//!
//! # What this does not do
//!
//! **Nothing calls these yet.** A declaration is a list; carrying a call out is
//! `alo-agentd`'s, and the task this pays says so in its own words — *making it
//! work on a machine* is the half every promise in this release owes. Said here
//! rather than implied by a green suite.

use alo_capability::{Arg, Effect, Offered, Requires, Takes, Verb, VerbError, Verbs, VerbsError};
use alo_strings::{Said, Strings, Word};

use crate::scheme::{Following, Schedule, ScheduleError, Scheme};
use crate::time::{TimeError, TimeOfDay};
use crate::words;

/// The name the standing-choice verb is asked for by.
pub const SET_THE_SCHEME: &str = "set_the_scheme";

/// The name the schedule verb is asked for by.
pub const FOLLOW_THE_CLOCK: &str = "follow_the_clock";

/// The argument naming light or dark.
pub const SCHEME: &str = "scheme";

/// The argument naming the hour the machine turns dark.
pub const DARK_AT: &str = "dark_at";

/// The argument naming the hour it turns light again.
pub const LIGHT_AT: &str = "light_at";

/// The option naming the light scheme, as a model sends it.
pub const LIGHT: &str = "light";

/// The option naming the dark scheme, as a model sends it.
pub const DARK: &str = "dark";

/// The first hour of a day: midnight.
const FIRST_HOUR: i64 = 0;

/// The last hour of a day: eleven at night.
///
/// A day has twenty-four hours and the last of them is 23, which is the kind of
/// off-by-one worth writing down beside the number rather than in a commit.
const LAST_HOUR: i64 = 23;

/// Why an appearance verb could not be declared.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Declaring {
    /// A declaration that does not satisfy the contract.
    #[error(transparent)]
    Verb(#[from] VerbError),
    /// A name already on the list.
    #[error(transparent)]
    List(#[from] VerbsError),
}

/// Why what a model asked for is not a schedule this machine can keep.
///
/// Both arms are reachable from arguments that **passed** validation: the range
/// check says an hour is between 0 and 23 and says nothing about whether two of
/// them differ, which is the whole distance between *a number in range* and *a
/// schedule*.
///
/// **No `Display`**, which is this crate's rule rather than this file's: the
/// only road to words is [`NotASchedule::said`], as it is for
/// [`TimeError::said`] and [`ScheduleError::said`]. [`Declaring`] above is
/// `thiserror`'s instead, because it carries `alo-capability`'s own errors and
/// that is how every other crate's `verbs.rs` declares it. **Two conventions
/// meet in this one file and neither is wrong** — one belongs to the crate the
/// error is about, the other to the contract the declaration is held to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotASchedule {
    /// An hour outside a day, which validation should already have refused.
    Time(TimeError),
    /// Dark and light at the same moment.
    Schedule(ScheduleError),
}

impl NotASchedule {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Time(why) => why.word(),
            Self::Schedule(why) => why.word(),
        }
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        match self {
            Self::Time(why) => why.said(strings),
            Self::Schedule(why) => why.said(strings),
        }
    }
}

impl From<TimeError> for NotASchedule {
    fn from(why: TimeError) -> Self {
        Self::Time(why)
    }
}

impl From<ScheduleError> for NotASchedule {
    fn from(why: ScheduleError) -> Self {
        Self::Schedule(why)
    }
}

/// The appearance verbs, as a list of their own.
///
/// # Errors
/// [`Declaring`], which the verbs as written cannot cause.
pub fn appearance_verbs() -> Result<Verbs, Declaring> {
    let mut verbs = Verbs::default();
    declare_into(&mut verbs)?;
    Ok(verbs)
}

/// Put the appearance verbs on an existing list.
///
/// # Errors
/// [`Declaring::List`] if the list already has a verb of either name.
pub fn declare_into(verbs: &mut Verbs) -> Result<(), Declaring> {
    verbs.declare(set_the_scheme()?)?;
    verbs.declare(follow_the_clock()?)?;
    Ok(())
}

/// Light or dark, whatever the time is.
fn set_the_scheme() -> Result<Verb, VerbError> {
    Verb::checked(
        SET_THE_SCHEME,
        words::VERB_SET_THE_SCHEME,
        Effect::Change,
        vec![Arg::taking(
            SCHEME,
            words::VERB_SCHEME,
            Takes::choice([
                Offered::called(LIGHT, words::SCHEME_LIGHT),
                Offered::called(DARK, words::SCHEME_DARK),
            ]),
        )],
        nothing_a_grant_could_cover(),
        words::VERB_SET_THE_SCHEME_SENTENCE,
    )
}

/// Dark from one hour, light again from another.
fn follow_the_clock() -> Result<Verb, VerbError> {
    Verb::checked(
        FOLLOW_THE_CLOCK,
        words::VERB_FOLLOW_THE_CLOCK,
        Effect::Change,
        vec![
            Arg::taking(
                DARK_AT,
                words::VERB_DARK_AT,
                Takes::count(FIRST_HOUR, LAST_HOUR),
            ),
            Arg::taking(
                LIGHT_AT,
                words::VERB_LIGHT_AT,
                Takes::count(FIRST_HOUR, LAST_HOUR),
            ),
        ],
        nothing_a_grant_could_cover(),
        words::VERB_FOLLOW_THE_CLOCK_SENTENCE,
    )
}

/// Why neither verb requires a grant, written down because the contract asks.
fn nothing_a_grant_could_cover() -> Requires {
    Requires::nothing_because(
        "how a machine looks is not a file, a device or a place anything can leave to, so there \
         is nothing for a grant to be over. The control is the approval: both verbs change the \
         machine, so neither runs until a person has read its sentence and agreed to it.",
    )
}

/// Which scheme an option's name means.
///
/// The names are the ones this file declares, so an unknown one is a caller that
/// did not validate against this verb's own options.
#[must_use]
pub fn the_scheme_named(chosen: &str) -> Option<Scheme> {
    match chosen {
        LIGHT => Some(Scheme::Light),
        DARK => Some(Scheme::Dark),
        _ => None,
    }
}

/// The standing choice an approved `set_the_scheme` means.
#[must_use]
pub fn a_standing_choice(scheme: Scheme) -> Following {
    Following::Always(scheme)
}

/// The schedule an approved `follow_the_clock` means.
///
/// Both hours are on the hour: a verb that offered minutes would be offering a
/// precision the promise does not ask for, and *use dark after six* is a time of
/// day rather than an instant.
///
/// # Errors
/// [`NotASchedule`] if an hour is outside a day, or if both name the same
/// moment — which a range check cannot see.
pub fn a_schedule_from(dark_at: u8, light_at: u8) -> Result<Following, NotASchedule> {
    let dark_from = TimeOfDay::checked(dark_at, 0)?;
    let light_from = TimeOfDay::checked(light_at, 0)?;
    Ok(Following::TheClock(Schedule::checked(
        dark_from, light_from,
    )?))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;
    use alo_capability::Given;

    /// **Both verbs are on the list, by name.** Asserted as existence rather
    /// than as a count: a test that only checked `declare_into` returned `Ok`
    /// would pass on a list with nothing on it, which is the shape
    /// `docs/misreadings/a-walk-answers-a-narrower-question-than-the-one-asked-of-it.md`
    /// is about.
    #[test]
    fn both_verbs_are_declared_and_named() {
        let verbs = appearance_verbs().unwrap();
        let named: BTreeSet<&str> = verbs.all().map(Verb::name).collect();
        assert!(named.contains(SET_THE_SCHEME), "{named:?}");
        assert!(named.contains(FOLLOW_THE_CLOCK), "{named:?}");
        assert_eq!(named.len(), 2, "{named:?}");
    }

    /// A list that already holds one of these names refuses the second putting.
    #[test]
    fn a_name_already_on_the_list_is_refused() {
        let mut verbs = appearance_verbs().unwrap();
        assert!(matches!(declare_into(&mut verbs), Err(Declaring::List(_))));
    }

    /// **The options the verb offers and the names [`the_scheme_named`] answers
    /// are two lists, and this compares them both ways.**
    ///
    /// Containment one way would pass while the other list was short — the fault
    /// `docs/misreadings/a-comparison-only-looks-at-what-both-sides-agree-exists.md`
    /// records, where a third drawn control was never announced because nothing
    /// asked whether the pairs were all there.
    #[test]
    fn every_option_offered_is_an_option_this_file_can_answer() {
        let verbs = appearance_verbs().unwrap();
        let verb = verbs.of(SET_THE_SCHEME).unwrap();
        let argument = verb.args().first().unwrap();
        // An argument that is no longer a choice yields an empty set, which the
        // comparison below reports as plainly as a drifted name would. Written
        // without `panic!` because this crate's lint allowance is
        // `clippy::unwrap_used` and nothing else, and one test is a poor reason
        // to widen it.
        let offered: BTreeSet<&str> = match argument.takes() {
            Takes::Choice(options) => options.iter().map(Offered::name).collect(),
            _ => BTreeSet::new(),
        };
        let answered: BTreeSet<&str> = [LIGHT, DARK].into_iter().collect();
        assert_eq!(
            offered,
            answered,
            "the options offered and the names answered have drifted apart, or the \
             argument is no longer a choice: {:?}",
            argument.takes()
        );

        for option in &offered {
            assert!(
                the_scheme_named(option).is_some(),
                "an option nothing can answer: {option}"
            );
        }
        assert!(the_scheme_named("sepia").is_none());
        assert!(the_scheme_named("").is_none());
    }

    /// Each option names a different scheme — a pair that both answered `Light`
    /// would satisfy the test above and still be wrong.
    #[test]
    fn the_two_options_are_the_two_schemes() {
        assert_eq!(the_scheme_named(LIGHT), Some(Scheme::Light));
        assert_eq!(the_scheme_named(DARK), Some(Scheme::Dark));
    }

    /// A standing choice ignores the clock, which is what `Always` means.
    #[test]
    fn a_standing_choice_holds_at_every_hour() {
        let chosen = a_standing_choice(Scheme::Dark);
        for hour in 0..=23 {
            let now = TimeOfDay::checked(hour, 0).unwrap();
            assert_eq!(chosen.at(now), Scheme::Dark, "at {hour}");
        }
    }

    /// **Use dark after six**, read back at the hours either side of it.
    #[test]
    fn dark_after_six_is_dark_after_six() {
        let following = a_schedule_from(18, 7).unwrap();
        assert_eq!(
            following.at(TimeOfDay::checked(17, 59).unwrap()),
            Scheme::Light
        );
        assert_eq!(
            following.at(TimeOfDay::checked(18, 0).unwrap()),
            Scheme::Dark
        );
        assert_eq!(
            following.at(TimeOfDay::checked(23, 0).unwrap()),
            Scheme::Dark
        );
        assert_eq!(
            following.at(TimeOfDay::checked(6, 59).unwrap()),
            Scheme::Dark
        );
        assert_eq!(
            following.at(TimeOfDay::checked(7, 0).unwrap()),
            Scheme::Light
        );
    }

    /// **The case a range check cannot see.** Both hours are inside 0 to 23, so
    /// the argument validation passes and the schedule is still not one.
    #[test]
    fn the_same_hour_twice_is_refused_though_both_are_valid_hours() {
        let both = 18;
        assert!(
            (FIRST_HOUR..=LAST_HOUR).contains(&i64::from(both)),
            "the test's own premise: the hour is in range"
        );
        assert!(matches!(
            a_schedule_from(both, both),
            Err(NotASchedule::Schedule(_))
        ));
    }

    /// An hour outside a day is refused as a time, not as a schedule.
    #[test]
    fn an_hour_outside_a_day_is_not_a_time() {
        assert!(matches!(a_schedule_from(24, 7), Err(NotASchedule::Time(_))));
    }

    /// Both verbs change the machine, so neither may run inside a turn.
    #[test]
    fn neither_verb_runs_without_an_approval() {
        let verbs = appearance_verbs().unwrap();
        for verb in verbs.all() {
            assert_eq!(verb.effect(), Effect::Change, "{}", verb.name());
        }
    }

    /// Every refusal has a word, and the word is one this crate declares —
    /// a refusal that reached `words` without being in `EVERY_WORD` would be a
    /// key no vocabulary holds.
    #[test]
    fn every_refusal_has_a_declared_word() {
        let refusals = [
            NotASchedule::Time(TimeError::NoSuchHour(24)),
            NotASchedule::Schedule(ScheduleError::TheSameMoment(
                TimeOfDay::checked(18, 0).unwrap(),
            )),
        ];
        let declared: BTreeSet<&str> = words::EVERY_WORD.iter().map(Word::named).collect();
        for refusal in refusals {
            assert!(
                declared.contains(refusal.word().named()),
                "{}",
                refusal.word().named()
            );
        }
    }

    /// **The validation is `alo-capability`'s, not this file's reading of it.**
    /// A call is what an agent actually makes, so the hours are checked by the
    /// same code that would check them on a machine.
    #[test]
    fn a_call_with_an_hour_outside_a_day_is_refused_at_the_boundary() {
        let verbs = appearance_verbs().unwrap();
        assert!(
            verbs
                .call(
                    FOLLOW_THE_CLOCK,
                    &[(DARK_AT, Given::number(24)), (LIGHT_AT, Given::number(7)),],
                )
                .is_err()
        );
        assert!(
            verbs
                .call(
                    FOLLOW_THE_CLOCK,
                    &[(DARK_AT, Given::number(18)), (LIGHT_AT, Given::number(7)),],
                )
                .is_ok()
        );
    }

    /// A scheme nobody offered is refused at the boundary too, so free text
    /// never reaches [`the_scheme_named`].
    #[test]
    fn a_call_naming_an_option_that_was_not_offered_is_refused() {
        let verbs = appearance_verbs().unwrap();
        assert!(
            verbs
                .call(SET_THE_SCHEME, &[(SCHEME, Given::text("warmer"))])
                .is_err()
        );
        assert!(
            verbs
                .call(SET_THE_SCHEME, &[(SCHEME, Given::text(DARK))])
                .is_ok()
        );
    }

    /// **The names a model sends, written out literally.**
    ///
    /// `docs/contracts/agent-verbs.md` calls a verb's name a *stable
    /// identifier, never reused for a different meaning*, and the same is true
    /// of an argument and of an option: they are what an agent on a machine
    /// already sends.
    ///
    /// Every other test here reads these through their constants, so renaming a
    /// constant moves the test with it and nothing goes red — **measured, by
    /// renaming `DARK` to `night` and watching all 107 tests pass.** This is the
    /// one place the spelling is written twice, which is what makes a rename
    /// visible rather than silent.
    #[test]
    fn the_names_that_go_on_the_wire_are_these_exact_spellings() {
        assert_eq!(SET_THE_SCHEME, "set_the_scheme");
        assert_eq!(FOLLOW_THE_CLOCK, "follow_the_clock");
        assert_eq!(SCHEME, "scheme");
        assert_eq!(DARK_AT, "dark_at");
        assert_eq!(LIGHT_AT, "light_at");
        assert_eq!(LIGHT, "light");
        assert_eq!(DARK, "dark");
    }
}
