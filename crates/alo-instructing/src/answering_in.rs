//! **The clause that says which language to answer in.**
//!
//! ★ `docs/features.md`: *the agent answers in the language it was asked in* —
//! task 5 of `docs/autonomy/v0-5-access-and-language-plan.md`. A person with an
//! English shell may ask in German, so the language is the **question's**, read
//! by [`crate::the_language_of`] on this machine with nothing sent anywhere.
//!
//! # Why it travels with the request and not with the instructions
//!
//! The instructions are what a grade is a grade of: a catalogue grade names
//! them by the SHA-256 of their text ([ADR 0034](../../../docs/decisions/0034-the-instructions-show-every-door-they-ask-a-model-to-choose.md)),
//! and every grade a machine reads to decide whether a model may be given the
//! agent was earned under the text [`Instructions::SHOWN_TO_A_TURN`] holds
//! today. A clause added inside that text would be a different text, and every
//! one of those grades would describe words no model had been shown.
//!
//! So the clause goes where the request goes — after it, in the same breath,
//! one line — and the graded text is untouched. What that costs is stated
//! rather than hidden: the line is still something the model reads, so a grade
//! earned without it is not proof about a turn that has it. The access plan's
//! task 5 measures the same fixed set with the clause present on the model this
//! machine holds, and the report says what it found.
//!
//! # What it does not do
//!
//! It never translates. The verbs, their arguments and the shape of the answer
//! are not words a person reads and stay exactly as they are; alo OS's own
//! sentences around an answer stay in the shell's language, where a person
//! chose them. And where the language cannot be read from the request, no
//! clause is added at all — a guess in front of a model is worse than the
//! model's own judgement of what it was asked in.

use alo_capability::Verbs;
use alo_strings::Language;

use crate::instructions::Instructions;
use crate::shown::shown_to_a_model;
use crate::the_language_asked_in::the_language_of;

/// **The clause, for a language that was read from the request.**
///
/// Names the language in its own word for itself where `alo-strings` has one —
/// *Deutsch*, not *German* — because that is the word the answer should be in.
#[must_use]
pub fn answering_in(language: &Language) -> String {
    match language.in_its_own_language() {
        Some(calls_itself) => format!(
            "\n\nAnswer in the language the request is written in: {calls_itself} ({}). The verb \
             and argument names above are not words a person reads; leave them exactly as they \
             are.",
            language.tag()
        ),
        None => String::new(),
    }
}

/// **The whole text an agent turn shows a model, answering in the language the
/// request was written in.**
///
/// [`crate::shown_to_a_turn`] with the clause after the request, and exactly
/// that text where the language could not be read — so a caller has one
/// function and no decision to get wrong.
#[must_use]
pub fn shown_to_a_turn_answering_in_the_language_asked_in(verbs: &Verbs, request: &str) -> String {
    let mut text = shown_to_a_model(Instructions::SHOWN_TO_A_TURN, verbs, request);
    if let Some(language) = the_language_of(request) {
        text.push_str(&answering_in(&language));
    }
    text
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::shown::shown_to_a_turn;
    use crate::testing::the_verbs;

    /// **The graded text is untouched**: what a turn is shown is what it was
    /// shown before, with one line after the request and nothing else moved.
    #[test]
    fn the_clause_is_added_after_the_request_and_the_graded_text_is_unchanged() {
        let verbs = the_verbs();
        let asked = "wo ist die Rechnung von Northstar?";
        let with = shown_to_a_turn_answering_in_the_language_asked_in(&verbs, asked);
        let without = shown_to_a_turn(&verbs, asked);
        assert!(with.starts_with(&without), "the graded text moved");
        assert!(with.contains(Instructions::SHOWN_TO_A_TURN.text()));
        assert!(with.trim_end().ends_with("leave them exactly as they are."));
        assert!(with[without.len()..].contains("Deutsch (de)"));
    }

    /// **A language that cannot be read adds no clause**, rather than a guess.
    #[test]
    fn a_request_in_no_language_this_machine_reads_is_shown_as_it_always_was() {
        let verbs = the_verbs();
        for unreadable in ["Northstar", "2026-09-16", ""] {
            assert_eq!(
                shown_to_a_turn_answering_in_the_language_asked_in(&verbs, unreadable),
                shown_to_a_turn(&verbs, unreadable),
                "{unreadable:?}"
            );
        }
    }

    /// **The clause names the language in its own language**, for each of the
    /// 24, and never translates a verb.
    #[test]
    fn the_clause_names_the_language_in_its_own_word_for_itself() {
        let german = Language::written("de").unwrap();
        let clause = answering_in(&german);
        assert!(clause.contains("Deutsch (de)"), "{clause}");
        assert!(clause.contains("not words a person reads"), "{clause}");

        let greek = Language::written("el").unwrap();
        assert!(answering_in(&greek).contains("Ελληνικά (el)"));

        // A language `alo-strings` does not carry has no word to name, so
        // nothing is said rather than a tag being shown to a model.
        let unofficial = Language::written("is").unwrap();
        assert_eq!(answering_in(&unofficial), "");
    }
}
