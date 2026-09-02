//! Why the application half says no, as values rather than as sentences.
//!
//! Two refusals live here and they are refusals of different things.
//! [`NotAnApplication`] refuses an **entry** — something this machine offered
//! for the list of what is installed that no verb could ever name.
//! [`NotInstalled`] refuses a **call** — a verb naming an application that is
//! not here.
//!
//! Both are values with a `said`, which is item 9e's rule met again: deciding
//! never depends on a vocabulary having been loaded, and the words are made
//! once, where the question was answered. [`NotInstalled`] in particular is
//! carried into `alo_capability::Refused` by [`crate::Reaching`], so what a
//! person was told is what the record keeps rather than a second rendering.

use alo_strings::{Filling, Said, Strings};

use crate::words;

/// Why something this machine offered cannot be an application a verb names.
///
/// Read by whoever is looking at why an installed application does not appear
/// — which is a person in front of the machine, not only whoever packaged it —
/// so it is said in their language like everything else here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotAnApplication {
    /// Nothing, or only spaces.
    NoIdentifier,
    /// Something that could never arrive as an argument: a space in it, a
    /// folder separator, a character that cannot be read in a sentence.
    NotAnIdentifier {
        /// The identifier as it was offered.
        offered: String,
    },
}

impl NotAnApplication {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub fn word(&self) -> words::Word {
        match self {
            Self::NoIdentifier => words::NO_IDENTIFIER,
            Self::NotAnIdentifier { .. } => words::NOT_AN_IDENTIFIER,
        }
    }

    /// What this says, in the language the person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = match self {
            Self::NoIdentifier => Filling::nothing(),
            Self::NotAnIdentifier { offered } => Filling::of("application", offered.clone()),
        };
        strings.say(&self.word().key(), &filling)
    }
}

/// A verb named an application that is not on this machine.
///
/// **Not a refusal by the grants**, and it is deliberately shaped like
/// `alo-files`' *there is nothing at that path*: there is nothing to act on, so
/// there is nothing to permit. It reaches the record through
/// `alo_capability::Refused::worded_elsewhere`, because only this crate holds
/// the list that could answer the question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotInstalled {
    /// The identifier the call named.
    wanted: String,
}

impl NotInstalled {
    /// Nothing on this machine goes by this identifier.
    #[must_use]
    pub fn wanting(identifier: &str) -> Self {
        Self {
            wanted: identifier.trim().to_owned(),
        }
    }

    /// What was asked for.
    #[must_use]
    pub fn wanted(&self) -> &str {
        &self.wanted
    }

    /// What this says, in the language the person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(
            &words::NOT_INSTALLED.key(),
            &Filling::of("application", self.wanted.clone()),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// Each refusal says what to do about the thing it refused, and names it.
    #[test]
    fn a_refusal_names_what_it_refused() {
        let strings = in_english();
        let missing = NotInstalled::wanting("  org.blender.Blender  ");
        assert_eq!(missing.wanted(), "org.blender.Blender");
        let said = missing.said(&strings);
        assert!(said.text().contains("org.blender.Blender"), "{said}");
        assert!(said.text().contains("name one that is here"), "{said}");

        let bad = NotAnApplication::NotAnIdentifier {
            offered: "/usr/bin/blender".to_owned(),
        };
        let said = bad.said(&strings);
        assert!(said.text().contains("/usr/bin/blender"), "{said}");
        assert!(said.text().contains("no folders in it"), "{said}");

        let none = NotAnApplication::NoIdentifier.said(&strings);
        assert!(none.text().contains("grant is made over"), "{none}");
    }

    /// **The identifier is the machine's and the words around it are the
    /// reader's.** The same rule a path is held to in `alo-capability`.
    #[test]
    fn the_words_around_an_identifier_are_translated_and_it_is_not() {
        let strings = translated(&[(
            words::NOT_INSTALLED,
            "{application} ist auf diesem Rechner nicht installiert",
        )]);
        let said = NotInstalled::wanting("org.blender.Blender").said(&strings);
        assert!(said.is_translated());
        assert_eq!(
            said.text(),
            "org.blender.Blender ist auf diesem Rechner nicht installiert"
        );
    }

    /// **A refusal never depends on a string table.** With no words at all it
    /// refuses exactly as firmly and answers with the key, marked.
    #[test]
    fn a_refusal_without_the_words_still_names_the_rule() {
        let strings = Strings::of(alo_strings::Vocabulary::empty());
        let said = NotInstalled::wanting("org.blender.Blender").said(&strings);
        assert!(said.is_a_bug());
        assert!(said.text().contains("applications.not-installed"), "{said}");
    }
}
