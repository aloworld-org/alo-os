//! What a person reads when a change could not be put to them, or when an
//! answer reached a machine with nothing waiting for one.
//!
//! Two types, because they are read at two moments and a caller does two
//! different things about them: [`NotAsked`] means nobody was asked anything,
//! and [`NotAnswered`] means somebody answered and nothing happened. Neither is
//! silent — a change that went nowhere and said nothing is an agent that looks
//! as though it ignored an instruction, and the person's next act is to ask for
//! the same thing again.
//!
//! Like everything else this repository says to a person, the sentences are
//! declared in [`crate::words`] and answered through a `said`; there is no
//! `Display` on either that would put English on a screen by accident.
//!
//! # Most of these are somebody else's words
//!
//! Three of the five carry the value whoever refused handed over —
//! `alo_capability::AnswerError` for a number that is not waiting or a question
//! that stood too long, and `alo_turn::NotDone` for everything a turn can
//! refuse — and ask *that* for the sentence. A surface is the last thing in the
//! chain and the one with the least right to describe what the capability model
//! decided: a second rendering of any of it would be a machine able to describe
//! one moment two ways, and the one the person read would be the one that
//! decided nothing.
//!
//! What is left is this crate's own two facts, and nobody else knows them:
//! there was nowhere to put the question, and there was nothing in front of the
//! person to answer.
//!
//! # Where the first two are read, since the screen is the thing that is
//! missing
//!
//! Not on this surface — there is not one. They are read in a service log by
//! whoever is standing the machine up, or on a text console while a session
//! starts. They are still declared and still translated, because a person
//! reading a log on their own machine reads their own language, and because the
//! alternative is English written into a source file, which `CLAUDE.md` calls a
//! bug wherever it appears.

use alo_capability::AnswerError;
use alo_strings::{Filling, Said, Strings};
use alo_turn::NotDone;

use crate::surface::SurfaceRefused;
use crate::words;

/// Why a change was not put to the person.
///
/// Three shapes of *nobody was asked*: no compositor at all, a compositor that
/// refused, and a number that names no question this turn is waiting on. The
/// first two are kept apart because the person is told different things — one
/// is *the desktop is not running*, the other is a fact the compositor knows,
/// like a machine with no screen — and because they are fixed by different
/// actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotAsked {
    /// There is no compositor to ask: nothing on this machine is drawing a
    /// screen at all.
    NoCompositor,
    /// The compositor was asked and refused, for the reason carried.
    Surface(SurfaceRefused),
    /// That number names nothing this turn is waiting on, or names a question
    /// that stood too long — `alo-capability`'s own refusal, carried whole.
    NotWaiting(AnswerError),
}

impl NotAsked {
    /// What to tell the person, in the language they read.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not:
    /// there is always something to put in front of the person, and where it
    /// came from is on the [`Said`]. A `Strings` that was never given
    /// [`crate::approving_words`] answers with the key, marked
    /// `Said::is_a_bug` — the honest answer to *the shell forgot to declare
    /// what this crate can say*.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::NoCompositor => strings.say(&words::NO_COMPOSITOR.key(), &Filling::nothing()),
            Self::Surface(SurfaceRefused::NothingToShowOn) => {
                strings.say(&words::NOTHING_TO_SHOW_ON.key(), &Filling::nothing())
            }
            Self::NotWaiting(why) => why.said(strings),
        }
    }

    /// Whether this is the machine having nowhere to put the question, rather
    /// than there being no question to put.
    ///
    /// A shell does two different things about them: the first is worth saying
    /// out loud to whoever is standing the machine up, and the second is an
    /// ordinary answer to a stale surface.
    #[must_use]
    pub fn is_nowhere_to_show_it(&self) -> bool {
        matches!(self, Self::NoCompositor | Self::Surface(_))
    }
}

/// Why an answer did not carry a change out.
///
/// Two shapes, and only one of them is about the machine. **The turn refused**
/// covers everything ADR 0001 §5 stops at the moment of execution — a grant
/// revoked since the question was asked, a question that stood too long, a disk
/// that could not — and is worded by `alo-turn`, which is worded by whoever
/// actually refused. **Nothing to answer** is this crate's own, and it is what
/// a second click on one approval meets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotAnswered {
    /// Nothing was in front of the person to answer.
    ///
    /// Most often because the same question was answered a moment ago, which is
    /// how *one approval, one execution* is kept at the surface as well as in
    /// the list: the second answer never reaches the turn at all.
    NothingToAnswer,
    /// The turn refused it, for the reason carried and in that reason's own
    /// words.
    Turn(NotDone),
}

impl NotAnswered {
    /// What to tell the person, in the language they read.
    ///
    /// One of these is this crate's sentence and the other is whoever refused
    /// the change — see this module's documentation for why it is not both.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::NothingToAnswer => {
                strings.say(&words::NOTHING_TO_ANSWER.key(), &Filling::nothing())
            }
            Self::Turn(why) => why.said(strings),
        }
    }

    /// Whether this machine has stopped keeping evidence.
    ///
    /// `alo_turn::NotDone::is_the_end_of_the_turn` read through the surface: a
    /// daemon meeting it has a machine to stop rather than an answer to retry,
    /// and a shell meeting it must not offer the question again.
    #[must_use]
    pub fn is_the_end_of_the_turn(&self) -> bool {
        match self {
            Self::NothingToAnswer => false,
            Self::Turn(why) => why.is_the_end_of_the_turn(),
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
    use crate::testing::{archiving_march, in_english, translated};
    use alo_strings::Strings;

    /// Every way a question can fail to be put to somebody.
    fn every_refusal_to_ask() -> Vec<NotAsked> {
        vec![
            NotAsked::NoCompositor,
            NotAsked::Surface(SurfaceRefused::NothingToShowOn),
            NotAsked::NotWaiting(AnswerError::NothingWaiting { number: 0 }),
            NotAsked::NotWaiting(AnswerError::Lapsed {
                call: Box::new(archiving_march()),
            }),
        ]
    }

    /// Every way an answer can fail to carry a change out.
    fn every_refusal_to_answer() -> Vec<NotAnswered> {
        vec![
            NotAnswered::NothingToAnswer,
            NotAnswered::Turn(NotDone::TurnClosed),
            NotAnswered::Turn(NotDone::NotAnswered(AnswerError::Lapsed {
                call: Box::new(archiving_march()),
            })),
        ]
    }

    /// **Every refusal says something a person could read**, and no two ways of
    /// failing at one moment read the same — a person told the same sentence
    /// for a missing desktop and a missing screen would go and fix the wrong
    /// one.
    ///
    /// The two lists are checked apart rather than together, because a fact
    /// that can be met at both moments is one fact and must read the same at
    /// both. The test below is the one that says so.
    #[test]
    fn every_refusal_reads_and_no_two_at_one_moment_read_the_same() {
        let strings = in_english();
        for moment in [
            every_refusal_to_ask()
                .iter()
                .map(|refusal| refusal.said(&strings))
                .collect::<Vec<_>>(),
            every_refusal_to_answer()
                .iter()
                .map(|refusal| refusal.said(&strings))
                .collect::<Vec<_>>(),
        ] {
            let mut seen: Vec<String> = Vec::new();
            for said in moment {
                assert!(!said.text().is_empty(), "something says nothing");
                assert!(!said.is_a_bug(), "{said} is not declared anywhere");
                assert!(!seen.contains(&said.text().to_owned()), "two say {said}");
                seen.push(said.text().to_owned());
            }
        }
    }

    /// **One fact reads the same wherever it is met.** A question that stood
    /// too long can be found when the surface tries to put it up and again when
    /// somebody answers it, and neither of those is this crate's sentence: both
    /// are `alo-capability`'s one refusal, carried whole. A machine with two
    /// wordings for it would be a machine whose person cannot tell whether the
    /// same thing happened twice.
    #[test]
    fn a_question_that_stood_too_long_reads_the_same_at_both_moments() {
        let strings = in_english();
        let lapsed = AnswerError::Lapsed {
            call: Box::new(archiving_march()),
        };
        assert_eq!(
            NotAsked::NotWaiting(lapsed.clone()).said(&strings).text(),
            NotAnswered::Turn(NotDone::NotAnswered(lapsed))
                .said(&strings)
                .text()
        );
    }

    /// **The only sentences this crate says of its own are its own three.**
    /// With nothing but this crate's list loaded, the three that are ours read
    /// and everything else is a key nothing declares — which is what says the
    /// rest are somebody else's words rather than copies of them.
    #[test]
    fn the_only_sentences_this_crate_says_of_its_own_are_its_own_three() {
        let ours = Strings::of(crate::words::approving_words().unwrap());
        let mut said_by_us = 0;
        for said in every_refusal_to_ask()
            .iter()
            .map(|refusal| refusal.said(&ours))
            .chain(
                every_refusal_to_answer()
                    .iter()
                    .map(|refusal| refusal.said(&ours)),
            )
        {
            if !said.is_a_bug() {
                said_by_us += 1;
            }
        }
        assert_eq!(
            said_by_us, 3,
            "this surface has started saying something somebody else already says"
        );
    }

    /// **A refusal arrives in the language the person reads** when somebody has
    /// translated it, and says so.
    #[test]
    fn a_refusal_is_read_in_the_language_the_person_reads() {
        let german = translated();
        // Nobody translated this crate's refusals in the fixture, so they are
        // English and say as much rather than pretending.
        let untranslated = NotAsked::NoCompositor.said(&german);
        assert!(!untranslated.is_translated());
        assert!(!untranslated.is_a_bug(), "{untranslated}");

        // And the sentence inside somebody else's refusal is rendered in the
        // language the refusal is read in, rather than in whichever language
        // the verb was declared in.
        let lapsed = NotAsked::NotWaiting(AnswerError::Lapsed {
            call: Box::new(archiving_march()),
        })
        .said(&german);
        assert!(
            lapsed.text().contains("verschieben"),
            "the quoted question was not read in the person's language: {lapsed}"
        );
    }

    /// The two facts a caller acts on are told apart without matching every
    /// variant there is: nowhere to put the question, and a machine that has
    /// stopped keeping evidence.
    #[test]
    fn the_facts_a_caller_acts_on_are_answerable_without_matching_everything() {
        assert!(NotAsked::NoCompositor.is_nowhere_to_show_it());
        assert!(NotAsked::Surface(SurfaceRefused::NothingToShowOn).is_nowhere_to_show_it());
        assert!(
            !NotAsked::NotWaiting(AnswerError::NothingWaiting { number: 3 })
                .is_nowhere_to_show_it()
        );

        assert!(NotAnswered::Turn(NotDone::TurnClosed).is_the_end_of_the_turn());
        assert!(!NotAnswered::NothingToAnswer.is_the_end_of_the_turn());
        assert!(
            !NotAnswered::Turn(NotDone::NotAnswered(AnswerError::NothingWaiting {
                number: 3
            }))
            .is_the_end_of_the_turn()
        );
    }
}
