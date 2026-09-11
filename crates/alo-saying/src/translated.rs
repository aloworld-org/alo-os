//! A translator's line, held to the rule the English is held to.
//!
//! `docs/features.md` promises that a person never learns the name of anything
//! alo OS rents, and [`crate::rented`] is how the English keeps it: every
//! sentence, note and key this workspace declares is walked against
//! [`crate::EVERYTHING_WE_RENT`] in CI. What that walk cannot reach is written
//! in `docs/contracts/translations.md` itself: a translation is a file a person
//! outside this repository types, it arrives in the image, and no test here
//! ever sees it. *Das Flatpak konnte nicht installiert werden* would have
//! reached a screen in somebody's own language, past a rule written to stop
//! exactly that sentence in this one.
//!
//! So the same question is asked of a translation at the moment it loads, which
//! is the first moment this repository holds the file. Same list, same
//! matching, same argument: [`what_a_translation_would_teach`] walks every
//! translated line with the matcher [`crate::rented`] uses on the English, and
//! a line naming a rented component answers with *what a person reading it
//! would have to learn* — never with *forbidden word*, because whoever fixes it
//! is looking for another sentence to write.
//!
//! # The line is left out; the file is not thrown away
//!
//! A translation is somebody's donated work, and `docs/contracts/translations.md`
//! already settles what one wrong line costs: that line, and nothing else. A
//! dropped gap, an invented gap, a renamed key — each is left out and the rest
//! of the language is shown. A rented name is held to exactly that rule rather
//! than a harsher one, for the same reason: refusing a whole language over one
//! line would turn a person's language off to keep a promise about one
//! sentence, which is a different promise being broken. [`crate::loading`] is
//! where the line is taken out, and [`crate::Damage`] is where the refusal is
//! reported — naming the file, the key and the language, so whoever fixes it
//! can find the line.
//!
//! # Only the translated text is read here
//!
//! The English check reads three places — the sentence, the note and the key —
//! because all three are this repository's to write. In a translation the only
//! text the translator wrote is the line itself: the keys are the vocabulary's,
//! already held to the rule where they are declared, and a key nothing declares
//! never reaches a screen — the vocabulary check leaves that line out as one
//! nothing here says. So this walk reads the sentences and nothing else.
//!
//! # This is a load-time answer and it does not stop a machine
//!
//! Like everything else about a translation, a rented name costs a line in a
//! log, never a boot: the value travels in [`crate::Damage`] and its sentence
//! keeps its English, because it is read by whoever built the image or
//! contributed the file — [`crate::failing`] is that argument in full.

use std::fmt;

use alo_strings::{Key, Language, Translation};

use crate::rented::{EVERYTHING_WE_RENT, Rented, names};

/// One rented name in one translated line: what a person reading that language
/// would have been made to learn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Taught {
    /// The file the line arrived in, so whoever fixes it knows which one to
    /// open.
    file: String,
    /// The string the line translates.
    key: Key,
    /// The language the line is written in.
    language: Language,
    /// What was found, and what alo OS rents it for.
    rented: Rented,
    /// The line itself, quoted back so nobody has to go looking.
    text: String,
}

impl Taught {
    /// The file the line arrived in.
    #[must_use]
    pub fn file(&self) -> &str {
        &self.file
    }

    /// The string the line translates.
    #[must_use]
    pub fn key(&self) -> &Key {
        &self.key
    }

    /// The language the line is written in.
    #[must_use]
    pub fn language(&self) -> &Language {
        &self.language
    }

    /// What was found.
    #[must_use]
    pub const fn rented(&self) -> Rented {
        self.rented
    }

    /// The line it was found in.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// Said the way [`crate::rented`] says it of the English — *this is what a
/// person would have to learn* — and on one line, because it is read in a
/// service log where one entry is one line.
impl fmt::Display for Taught {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{file}: a person reading {key} in {language} would have to learn what {name} is — \
             alo OS rents it as {why}, and they did not choose it. The line was left out and the \
             rest of the file was kept; it says: \"{text}\"",
            file = self.file,
            key = self.key,
            language = self.language,
            name = self.rented.name(),
            why = self.rented.why(),
            text = self.text
        )
    }
}

/// Every rented name in one translation, before any of it is shown.
///
/// Answers with all of them rather than with the first, for the reason the
/// English check does: somebody fixing a file wants to see the work, not to be
/// told about the next line each time they try again. A line naming two rented
/// things is two answers, one per name, because each needs its own sentence
/// about what it stands in for.
///
/// An empty answer is the ordinary case: a translation of sentences that name
/// nothing rented — which is every sentence alo OS ships — has nothing rented
/// to carry over.
#[must_use]
pub fn what_a_translation_would_teach(file: &str, translation: &Translation) -> Vec<Taught> {
    let mut taught = Vec::new();
    for (key, text) in translation.texts() {
        for rented in EVERYTHING_WE_RENT {
            if names(text, rented.name()) {
                taught.push(Taught {
                    file: file.to_owned(),
                    key: key.clone(),
                    language: translation.language().clone(),
                    rented,
                    text: text.to_owned(),
                });
            }
        }
    }
    taught
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A translation holding one line, for asking about one line at a time.
    fn saying(language: &str, key: &str, text: &str) -> Translation {
        Translation::into_language(Language::written(language).unwrap())
            .says(Key::named(key).unwrap(), text)
    }

    /// **A translated line naming a rented component is refused with the same
    /// argument the English is held to**, and the refusal names the key and
    /// the language, so whoever fixes it can find the line.
    #[test]
    fn a_translated_line_naming_a_rented_component_is_found() {
        let taught = what_a_translation_would_teach(
            "de.toml",
            &saying(
                "de",
                "applications.not-installed",
                "Das Flatpak konnte nicht installiert werden",
            ),
        );
        assert_eq!(taught.len(), 1, "{taught:?}");
        let only = taught.first().unwrap();
        assert_eq!(only.rented().name(), "Flatpak");
        assert_eq!(only.key().as_str(), "applications.not-installed");
        assert_eq!(only.language().tag(), "de");
        assert_eq!(only.file(), "de.toml");
        let said = only.to_string();
        assert!(
            said.contains("would have to learn what Flatpak is"),
            "{said}"
        );
        assert!(said.contains("applications.not-installed"), "{said}");
        assert!(said.contains("de"), "{said}");
        assert!(said.contains("how an application arrives"), "{said}");
        assert!(
            said.contains("Das Flatpak konnte nicht installiert werden"),
            "{said}"
        );
        assert!(!said.contains('\n'), "one log entry is one line: {said}");
    }

    /// **The list is the same list.** Every name the English is held to is
    /// caught in a translation too — a shorter list here would be the rule the
    /// contract says translations are held to quietly meaning less.
    #[test]
    fn every_name_the_english_is_held_to_is_caught_in_a_translation() {
        for rented in EVERYTHING_WE_RENT {
            let taught = what_a_translation_would_teach(
                "fr.toml",
                &saying(
                    "fr",
                    "files.gone",
                    &format!("le composant {} ne répond pas", rented.name()),
                ),
            );
            assert_eq!(taught.len(), 1, "{} was not found", rented.name());
            assert_eq!(taught.first().unwrap().rented().name(), rented.name());
        }
    }

    /// **However a language capitalises it.** German capitalises nouns and
    /// French does not, and neither spelling spares anybody the trouble of
    /// finding out what the word means.
    #[test]
    fn it_does_not_matter_how_a_language_writes_the_name() {
        for written in ["Ollama", "ollama", "OLLAMA"] {
            let taught = what_a_translation_would_teach(
                "de.toml",
                &saying("de", "models.gone", &format!("{written} läuft nicht")),
            );
            assert_eq!(taught.len(), 1, "{written} was not found");
        }
    }

    /// **A name inside a longer word is not a leak in any language either.**
    /// The matcher is the English check's, so what it waves through there it
    /// waves through here, and nobody learns to work around it.
    #[test]
    fn a_name_inside_a_longer_word_is_not_a_leak() {
        for sentence in ["ein Bootcamp", "une approche waylandaise"] {
            assert!(
                what_a_translation_would_teach("de.toml", &saying("de", "files.gone", sentence))
                    .is_empty(),
                "{sentence}"
            );
        }
    }

    /// **One line naming two rented things is two answers**, because each name
    /// needs its own sentence about what it was standing in for.
    #[test]
    fn a_line_naming_two_rented_things_is_two_answers() {
        let taught = what_a_translation_would_teach(
            "de.toml",
            &saying("de", "files.gone", "Docker oder Podman fehlt"),
        );
        assert_eq!(taught.len(), 2, "{taught:?}");
    }

    /// A translation with nothing rented in it teaches nothing, which is every
    /// translation of the sentences alo OS actually ships.
    #[test]
    fn a_translation_with_nothing_rented_teaches_nothing() {
        assert!(
            what_a_translation_would_teach(
                "de.toml",
                &saying("de", "files.gone", "Es ist nicht mehr da")
            )
            .is_empty()
        );
    }

    /// A translation with nothing in it at all has nothing to teach, which is
    /// the file a language somebody just started arrives as.
    #[test]
    fn an_empty_translation_teaches_nothing() {
        assert!(
            what_a_translation_would_teach(
                "mt.toml",
                &Translation::into_language(Language::written("mt").unwrap())
            )
            .is_empty()
        );
    }
}
