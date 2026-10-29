//! The whole text a model is shown: how to answer, the verbs, and the request.
//!
//! The verbs come from the registry the text is being built against, so a model
//! is told about the verbs the machine really has. Every model faces the same
//! text for the same registry, which is what makes two grades comparable at all
//! — and what makes a grade a statement about the turn that asks, once the turn
//! is built from here
//! ([ADR 0037](../../../docs/decisions/0037-the-words-a-turn-shows-a-model-are-the-products-own.md)).
//!
//! The request is last, so nothing after it can be mistaken for part of it.

use alo_capability::Verbs;

use crate::instructions::Instructions;
use crate::verb_as_told::as_told;
use crate::what_it_may_reach::WhatItMayReach;

/// **The whole text a model is shown**, under the given [`Instructions`]: how
/// to answer, every verb the registry declares, **what it may reach**, and the
/// request last.
///
/// `may_reach` is taken rather than assumed, for the same reason the verbs
/// are: a model is told what this machine really holds. It was told nothing at
/// all until 2026-10-24, and a turn with a real grant to
/// `/home/alo/Documents` spent all eight of its rounds guessing
/// `/home/user/…` and being refused — `crate::what_it_may_reach` carries the
/// measurement.
#[must_use]
pub fn shown_to_a_model(
    instructions: Instructions,
    verbs: &Verbs,
    may_reach: &WhatItMayReach,
    request: &str,
) -> String {
    let mut text = the_products_words(instructions, verbs);
    text.push_str(&may_reach.told());
    text.push_str("\n\nThe request: ");
    text.push_str(request);
    text
}

/// **The part of what a model is shown that is the product's alone**: how to
/// answer, and every verb the registry declares — everything before what the
/// turn may reach and the request.
///
/// Its own function so that `alo-turn` can recognise a question that already
/// begins with exactly these words, from the same registry, and not put them
/// in front of it a second time: the agent builds the whole text a turn shows,
/// and wrapping it again showed a model the instructions and every verb twice.
#[must_use]
pub fn the_products_words(instructions: Instructions, verbs: &Verbs) -> String {
    let mut text = String::from(instructions.text());
    for verb in verbs.all() {
        text.push_str(&as_told(verb));
    }
    text
}

/// **The whole text an agent turn shows a model** — [`shown_to_a_model`] under
/// [`Instructions::SHOWN_TO_A_TURN`].
///
/// A turn takes this rather than the set, so that what a turn shows and what a
/// grade was earned under cannot come apart by somebody passing the other one.
#[must_use]
pub fn shown_to_a_turn(verbs: &Verbs, may_reach: &WhatItMayReach, request: &str) -> String {
    shown_to_a_model(Instructions::SHOWN_TO_A_TURN, verbs, may_reach, request)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_verb_of_somebody_elses, the_verbs};

    /// **A model is told what a verb is for in the verb's own words.** Two
    /// descriptions of one verb are two things that can disagree, and the one
    /// the model saw would be the one nobody maintains.
    #[test]
    fn the_text_describes_each_verb_in_the_words_the_verb_declared() {
        let verbs = the_verbs();
        let text = shown_to_a_model(
            Instructions::AsFirstWritten,
            &verbs,
            &WhatItMayReach::nothing(),
            "do something",
        );
        for verb in verbs.all() {
            assert!(text.contains(verb.name()), "{}", verb.name());
            assert!(text.contains(verb.purpose_as_written()), "{}", verb.name());
            for arg in verb.args() {
                assert!(text.contains(arg.purpose_as_written()), "{}", arg.name());
            }
        }
    }

    /// Every kind of argument is described, including the two that carry a
    /// bound: a model told "a whole number" and refused for sending 5000 has
    /// been measured on something nobody asked it.
    #[test]
    fn every_bound_a_verb_declared_is_in_the_text() {
        let text = shown_to_a_model(
            Instructions::AsFirstWritten,
            &the_verbs(),
            &WhatItMayReach::nothing(),
            "do something",
        );
        assert!(text.contains("a full path"), "{text}");
        assert!(text.contains("at most 255 characters"), "{text}");
        assert!(text.contains("a whole number from 1 to 1000"), "{text}");
        assert!(
            text.contains("one of: left_half, right_half, whole_screen"),
            "{text}"
        );
        assert!(
            text.contains("an installed application's identifier"),
            "{text}"
        );
    }

    /// The two doors are named, because sending a change through the read door
    /// is one of the ways a model fails and it must not fail for want of being
    /// told which is which.
    #[test]
    fn the_text_says_which_door_each_verb_takes() {
        let text = shown_to_a_model(
            Instructions::AsFirstWritten,
            &the_verbs(),
            &WhatItMayReach::nothing(),
            "do something",
        );
        assert!(text.contains("list_folder (read)"), "{text}");
        assert!(text.contains("move_file (propose)"), "{text}");
    }

    /// The request is the last thing in the text, so nothing after it can be
    /// mistaken for part of it.
    #[test]
    fn the_request_comes_last() {
        let text = shown_to_a_model(
            Instructions::AsFirstWritten,
            &the_verbs(),
            &WhatItMayReach::nothing(),
            "list what is in /home/anna",
        );
        assert!(
            text.ends_with("The request: list what is in /home/anna"),
            "{text}"
        );
    }

    /// **Only how to answer differs between two sets.** Everything a model is
    /// told about the machine — the verbs, their doors, their bounds — and the
    /// request itself are the same text under either set, which is what lets
    /// two grades be read as one variable apart (ADR 0034, decision 2).
    #[test]
    fn two_sets_of_instructions_differ_only_in_how_to_answer() {
        let verbs = the_verbs();
        let request = "rename /home/anna/scan001.pdf to march.pdf";
        for instructions in Instructions::ALL {
            let text = shown_to_a_model(instructions, &verbs, &WhatItMayReach::nothing(), request);
            let (shown, rest) = text.split_at(instructions.text().len());
            assert_eq!(shown, instructions.text());
            assert_eq!(
                rest,
                shown_to_a_model(
                    Instructions::AsFirstWritten,
                    &verbs,
                    &WhatItMayReach::nothing(),
                    request
                )
                .split_at(Instructions::AsFirstWritten.text().len())
                .1
            );
        }
    }

    /// **What a turn shows is what the named set says**, and nothing about the
    /// machine or the request is added or dropped for a turn. A turn asking in
    /// words a measurement never used is the mismatch ADR 0037 exists to close.
    #[test]
    fn a_turn_is_shown_the_set_this_crate_names() {
        let verbs = the_verbs();
        let request = "what is in /home/anna";
        assert_eq!(
            shown_to_a_turn(&verbs, &WhatItMayReach::nothing(), request),
            shown_to_a_model(
                Instructions::SHOWN_TO_A_TURN,
                &verbs,
                &WhatItMayReach::nothing(),
                request
            )
        );
        assert!(
            shown_to_a_turn(&verbs, &WhatItMayReach::nothing(), request)
                .starts_with(Instructions::SHOWN_TO_A_TURN.text())
        );
    }

    /// A registry with a verb this system does not ship tells a model about it
    /// like any other, because the words are built from whatever registry is
    /// handed over — an adapter's verbs are the machine's verbs.
    #[test]
    fn a_verb_from_somewhere_else_is_told_of_like_any_other() {
        let mut verbs = the_verbs();
        verbs.declare(a_verb_of_somebody_elses()).unwrap();
        let text = shown_to_a_turn(&verbs, &WhatItMayReach::nothing(), "water the kitchen");
        assert!(text.contains("water_the_plants (propose)"), "{text}");
        assert!(text.contains("one name, at most 64 characters"), "{text}");
    }
}
