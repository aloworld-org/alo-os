//! What the six verbs say, in the language the person reads.
//!
//! `alo_capability::Verb` carries the English a verb was declared with — its
//! purpose, each argument's purpose, and the sentence a person approves — and
//! it is the *source*, in the sense `alo-strings` means: the sentence somebody
//! translates, not the sentence everybody is shown. This file is the other
//! side of that. Given a call and the strings this machine reads, it answers
//! with what a person actually sees.
//!
//! # The sentence is the awkward one, and this is why it works
//!
//! A person approves a sentence, so `alo_capability::Verb::checked` refuses one
//! that does not name every argument the verb takes: an argument the sentence
//! leaves out is one they did not agree to. Translate the sentence and that
//! guarantee has to hold in the new language too — a German sentence that
//! dropped `{into}` would have somebody approving *move march.pdf* with no
//! word about where it was going.
//!
//! Nothing here re-implements that check. [`crate::words`] declares the
//! sentence **as the verb was declared with it**, and
//! `alo_strings::Vocabulary::check` refuses any translation that drops a gap
//! the source has or invents one it does not. So the rule that was enforced
//! once in English is enforced once per language, by the crate whose job that
//! is, on the same string.
//!
//! # What this does not do
//!
//! **It does not put a translated sentence into the record, or into an
//! approval.** `alo_capability::Call` renders and keeps its own sentence when
//! the call is made, and moving that onto `alo-strings` moves the whole of
//! `alo-capability` — which is item 9e in `docs/autonomy/QUEUE.md`, and is not
//! this crate's to do. Until then a shell shows what this file answers with,
//! and what is written down is the English the call carries.

use alo_capability::Call;
use alo_strings::{Filling, Said, Strings};

use crate::words::{self, Spoken};

/// What one of the six is for, in the language the person reads.
///
/// `None` for a verb that is not one of the six — the file half says what it
/// does and does not answer for anybody else's verbs.
#[must_use]
pub fn purpose(verb: &str, strings: &Strings) -> Option<Said> {
    Some(strings.say(&spoken(verb)?.purpose.key(), &Filling::nothing()))
}

/// What one of the six wants an argument for, in the language the person
/// reads — what a shell shows beside the box when an agent asks for something.
///
/// `None` for a verb that is not one of the six, and for an argument that verb
/// does not take.
#[must_use]
pub fn purpose_of(verb: &str, argument: &str, strings: &Strings) -> Option<Said> {
    Some(strings.say(
        &spoken(verb)?.argument(argument)?.key(),
        &Filling::nothing(),
    ))
}

/// **What a person is asked to approve, in the language they read.**
///
/// The words are the verb's and the values are the call's — validated values,
/// as `alo_capability::sentence` requires, because the whole point of the
/// sentence is that nothing the model wrote appears in it. Translating moves
/// the words and never the values.
///
/// `None` for a call to a verb that is not one of the six.
#[must_use]
pub fn sentence(call: &Call, strings: &Strings) -> Option<Said> {
    let spoken = spoken(call.verb())?;
    let mut filling = Filling::nothing();
    for (argument, value) in call.values() {
        filling = filling.and(argument.clone(), value.describe());
    }
    Some(strings.say(&spoken.sentence.key(), &filling))
}

/// The words of one of the six.
fn spoken(verb: &str) -> Option<&'static Spoken> {
    words::THE_SIX.iter().find(|spoken| spoken.verb == verb)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;
    use crate::verbs::file_verbs;
    use alo_strings::{CameFrom, Language, Translation, Vocabulary};

    /// A call to each of the three that change something.
    fn moving() -> Call {
        file_verbs()
            .unwrap()
            .call(
                "move_file",
                &[
                    (
                        "file",
                        alo_capability::Given::text("/home/anna/Invoices/march.pdf"),
                    ),
                    ("into", alo_capability::Given::text("/home/anna/Archive")),
                ],
            )
            .unwrap()
    }

    /// Nothing translated: what a person reads is what the verb was declared
    /// with, which is the sentence `alo_capability` already generated.
    #[test]
    fn with_no_translation_a_person_reads_what_the_verb_was_declared_with() {
        let call = moving();
        let said = sentence(&call, &in_english()).unwrap();
        assert_eq!(said.text(), call.sentence());
        assert_eq!(said.came_from(), &CameFrom::TheSource);
        assert!(said.unfilled().is_empty());
    }

    /// **The sentence a person approves, in German.** The values are the
    /// call's, the words are the translation's, and the two paths are in it
    /// because a translation that dropped one could not have been loaded.
    #[test]
    fn the_sentence_a_person_approves_is_the_one_they_read() {
        let mut strings = in_english();
        let german = strings
            .vocabulary()
            .check(
                Translation::into_language(Language::written("de").unwrap())
                    .says(
                        words::MOVE_FILE_SENTENCE.key(),
                        "{file} nach {into} verschieben",
                    )
                    .says(
                        words::MOVE_FILE.key(),
                        "eine Datei in einen Ordner verschieben",
                    ),
            )
            .unwrap();
        strings.speaks(german).unwrap();
        strings.prefers(&[Language::written("de").unwrap()]);

        let said = sentence(&moving(), &strings).unwrap();
        assert_eq!(
            said.text(),
            "/home/anna/Invoices/march.pdf nach /home/anna/Archive verschieben"
        );
        assert!(said.is_translated());
        assert_eq!(
            purpose("move_file", &strings).unwrap().text(),
            "eine Datei in einen Ordner verschieben"
        );
        // The argument nobody translated is still English, and says so.
        let into = purpose_of("move_file", "into", &strings).unwrap();
        assert_eq!(into.text(), "the folder it goes into");
        assert!(!into.is_translated());
    }

    /// **A translation that dropped an argument out of an approval sentence is
    /// refused before anybody can be shown it.** This is the guarantee item 9b
    /// exists to keep: `alo_capability` refuses such a sentence in English, and
    /// the check on a translation is that same rule in the other language.
    #[test]
    fn a_translated_sentence_that_leaves_an_argument_out_is_refused() {
        let vocabulary = Vocabulary::empty();
        let mut vocabulary = vocabulary;
        crate::words::declare_into(&mut vocabulary).unwrap();
        let wrongs = vocabulary
            .check(
                Translation::into_language(Language::written("de").unwrap())
                    .says(words::MOVE_FILE_SENTENCE.key(), "{file} verschieben"),
            )
            .unwrap_err();
        assert_eq!(wrongs.how_many(), 1);
        assert!(
            wrongs
                .to_string()
                .contains("put {into} back into the sentence"),
            "{wrongs}"
        );
    }

    /// Every one of the six answers, and nothing else does. A verb from
    /// somewhere else is not this crate's to describe, and answering for one
    /// would be answering with words about a capability we did not declare.
    #[test]
    fn the_six_answer_and_nothing_else_does() {
        let strings = in_english();
        let verbs = file_verbs().unwrap();
        for verb in verbs.all() {
            assert_eq!(
                purpose(verb.name(), &strings).unwrap().text(),
                verb.purpose(),
                "{}",
                verb.name()
            );
            for arg in verb.args() {
                let said = purpose_of(verb.name(), &arg.name, &strings).unwrap();
                assert_eq!(said.text(), arg.purpose, "{} {}", verb.name(), arg.name);
                assert!(!said.is_a_bug());
            }
            assert!(purpose_of(verb.name(), "colour", &strings).is_none());
        }
        assert!(purpose("open_application", &strings).is_none());
        assert!(purpose_of("open_application", "id", &strings).is_none());
    }
}
