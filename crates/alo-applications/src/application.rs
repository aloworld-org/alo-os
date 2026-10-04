//! One installed application: the identifier it is granted by, and the name a
//! person sees.
//!
//! # The identifier is approved; the name is only shown
//!
//! This is the decision the whole file exists for. An application has two
//! names: the identifier this machine knows it by (`org.blender.Blender`) and
//! whatever it calls itself in its desktop entry (*Blender*). Only the first
//! ever goes into a grant or into the sentence a person approves.
//!
//! The reason is that the second is written by whoever packaged the
//! application, and **two applications can call themselves the same thing**.
//! *Approve: open Mail* is a sentence that reads identically whether the
//! application behind it is the one a person installed on purpose or one that
//! arrived beside it, and a capability model whose approval sentence can be
//! chosen by the thing being approved has given away the only part of itself
//! that matters. No two applications share an identifier, so the identifier is
//! what is approved, and the name is shown beside it
//! ([`Application::shown`]) rather than inside it.
//!
//! # A name is data, not a string
//!
//! What an application calls itself arrives in whatever language it was
//! packaged in and is not ours to translate — the rule `alo-files` holds a
//! filename to and `alo-egress` holds a host to. It is also not ours to trust:
//! a name carrying a newline or an escape sequence could rewrite the line a
//! person is reading, so a name that cannot be shown is **dropped** and the
//! application is shown by its identifier alone. Nothing is lost that could
//! have been acted on, because nothing is ever acted on by name.
//!
//! # One exception, and it is in the type rather than in a habit
//!
//! [ADR 0085](../../../docs/decisions/0085-how-a-person-reaches-settings.md)
//! decided that an application **this project packages** takes its name from
//! the vocabulary, while one **somebody else packages** keeps the name they
//! were given. The discriminator is the rule's own stated reason rather than
//! its wording: *packaged in and not ours to translate* is a fact about third-
//! party software and says nothing about ours. A Dock entry reading `Settings`
//! in every language is the bug `CLAUDE.md` calls hardcoded English, shipped by
//! us, in the first row a person sees.
//!
//! So [`Called`] has two kinds and **nothing may guess which**. That is why
//! [`Application::name`] hands back the kind rather than a `&str`: a caller
//! that wanted the packager's string and got `None` for Settings would have
//! guessed by omission, which is the habit this type exists to prevent.

use alo_strings::{Filling, Strings, Word};

use crate::refusing::NotAnApplication;
use crate::words;

/// **What an application is called, and whose words those are.**
///
/// Two kinds, because there are two kinds of packager, and a reader must not be
/// able to take one for the other (ADR 0085). Ordering is by the text a person
/// would see, with a packager's name before one of ours when the two read the
/// same, so a sorted list is stable without either kind being privileged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Called {
    /// What it calls itself, in whatever language it was packaged in. Not ours
    /// to translate and not ours to trust, which is why it arrived through
    /// this file's own `showable`.
    ByWhoeverPackagedIt(String),
    /// One of this project's own words, for an application this project
    /// packages. Read in the person's language like every other sentence.
    InOurOwnWords(Word),
}

impl Called {
    /// What a person reads, in their own language where the words are ours.
    ///
    /// A `String` rather than an `alo_strings::Said`, for the reason
    /// [`Application::shown`] gives: what comes out is a fragment placed inside
    /// something somebody else is writing. Half of these never passed through a
    /// vocabulary at all — a packager's name has no key and no translation to
    /// be missing — so a `Said` would be carrying an answer about one kind that
    /// is meaningless for the other.
    #[must_use]
    pub fn shown(&self, strings: &Strings) -> String {
        match self {
            Self::ByWhoeverPackagedIt(called) => called.clone(),
            Self::InOurOwnWords(word) => strings.say(&word.key(), &Filling::nothing()).into_text(),
        }
    }

    /// What this sorts by: the words themselves, in the language the code is
    /// written in, which is the only text available without a vocabulary.
    fn as_written(&self) -> &str {
        match self {
            Self::ByWhoeverPackagedIt(called) => called,
            Self::InOurOwnWords(word) => word.says(),
        }
    }

    /// Which kind, for an ordering that does not depend on what two names say.
    const fn kind(&self) -> u8 {
        match self {
            Self::ByWhoeverPackagedIt(_) => 0,
            Self::InOurOwnWords(_) => 1,
        }
    }
}

impl PartialOrd for Called {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Called {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.as_written()
            .cmp(other.as_written())
            .then_with(|| self.kind().cmp(&other.kind()))
    }
}

/// One application this machine has.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Application {
    /// What this machine knows it by, and what a grant is made over. Matched
    /// exactly, with no case folding.
    identifier: String,
    /// What it calls itself, when that is something a person can be shown.
    called: Option<Called>,
}

impl Application {
    /// An application this machine knows only by its identifier.
    ///
    /// # Errors
    /// [`NotAnApplication`], if the identifier is one no verb could ever name.
    pub fn identified(identifier: &str) -> Result<Self, NotAnApplication> {
        Ok(Self {
            identifier: checked(identifier)?,
            called: None,
        })
    }

    /// An application, and what its desktop entry says it is called.
    ///
    /// A name that cannot be shown in one line is dropped rather than refused:
    /// the application is still installed, still grantable and still reachable,
    /// and only the way it is displayed changes.
    ///
    /// # Errors
    /// [`NotAnApplication`], if the identifier is one no verb could ever name.
    /// Never for the name — see above.
    pub fn called(identifier: &str, called: &str) -> Result<Self, NotAnApplication> {
        Ok(Self {
            identifier: checked(identifier)?,
            called: showable(called).map(Called::ByWhoeverPackagedIt),
        })
    }

    /// **An application this project packages, named in our own words.**
    ///
    /// The exception ADR 0085 decided, and the only road to it: a caller cannot
    /// reach [`Called::InOurOwnWords`] with a word of somebody else's, because
    /// the only words this crate can be handed are the ones it declares.
    ///
    /// # Errors
    /// [`NotAnApplication`], if the identifier is one no verb could ever name.
    pub fn ours(identifier: &str, called: Word) -> Result<Self, NotAnApplication> {
        Ok(Self {
            identifier: checked(identifier)?,
            called: Some(Called::InOurOwnWords(called)),
        })
    }

    /// **Settings**, which this project packages and which the Dock shows.
    ///
    /// ADR 0085 puts it on the Dock among the applications rather than beside
    /// them, so it is an application like any other — and the one application
    /// whose name is read out of the vocabulary.
    ///
    /// # Errors
    /// [`NotAnApplication`], if `identifier` is one no verb could ever name.
    pub fn settings(identifier: &str) -> Result<Self, NotAnApplication> {
        Self::ours(identifier, words::SETTINGS)
    }

    /// What this machine knows it by, and what a grant is made over.
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// **What it is called, and whose words those are**, when there is a name
    /// a person can be shown.
    ///
    /// The kind rather than the text, so that nothing may guess which (ADR
    /// 0085). [`Called::shown`] is how a caller gets words out of it, and it
    /// needs the vocabulary because half of these are ours.
    #[must_use]
    pub const fn name(&self) -> Option<&Called> {
        self.called.as_ref()
    }

    /// How a shell shows it in a list — the name and the identifier together,
    /// or the identifier alone when there is no name to show.
    ///
    /// A `String` rather than a `Said`, because what comes out is a **fragment
    /// placed inside something** — a row, a column, a sentence somebody else is
    /// writing — which is the shape `alo_capability::Reach::shown` uses for the
    /// same reason.
    #[must_use]
    pub fn shown(&self, strings: &Strings) -> String {
        match &self.called {
            Some(called) => strings
                .say(
                    &words::CALLED.key(),
                    &Filling::of("called", called.shown(strings))
                        .and("application", self.identifier.clone()),
                )
                .into_text(),
            None => self.identifier.clone(),
        }
    }
}

/// An identifier a verb could name, or the refusal saying why not.
///
/// The rules are `alo_capability::Arg`'s for a `Takes::Application`, on purpose:
/// an entry on this machine's list that could never arrive as an argument is an
/// entry nothing can ever reach, and it is better to be told about it than to
/// have it sit there looking installed.
fn checked(identifier: &str) -> Result<String, NotAnApplication> {
    let identifier = identifier.trim();
    if identifier.is_empty() {
        return Err(NotAnApplication::NoIdentifier);
    }
    if identifier.chars().any(char::is_control)
        || identifier.chars().any(char::is_whitespace)
        || identifier.contains('/')
        || identifier.contains('\\')
    {
        return Err(NotAnApplication::NotAnIdentifier {
            offered: identifier.to_owned(),
        });
    }
    Ok(identifier.to_owned())
}

/// A name if it is one a person can be shown, and nothing if it is not.
fn showable(called: &str) -> Option<String> {
    let called = called.trim();
    if called.is_empty() || called.chars().any(char::is_control) {
        return None;
    }
    Some(called.to_owned())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    #[test]
    fn an_application_is_its_identifier_and_what_it_calls_itself() {
        let blender = Application::called("  org.blender.Blender ", " Blender ").unwrap();
        assert_eq!(blender.identifier(), "org.blender.Blender");
        assert_eq!(
            blender.name(),
            Some(&Called::ByWhoeverPackagedIt("Blender".to_owned()))
        );
        assert_eq!(
            blender.shown(&in_english()),
            "Blender (org.blender.Blender)"
        );

        let bare = Application::identified("org.gimp.GIMP").unwrap();
        assert_eq!(bare.name(), None);
        assert_eq!(bare.shown(&in_english()), "org.gimp.GIMP");
    }

    /// **An entry no verb could name is refused where it is offered**, rather
    /// than sitting on the list looking installed. The rules are the ones an
    /// `application` argument is validated against, so the two cannot drift.
    #[test]
    fn an_identifier_no_verb_could_name_is_refused() {
        assert_eq!(
            Application::identified("   "),
            Err(NotAnApplication::NoIdentifier)
        );
        for offered in [
            "org blender",
            "/usr/bin/blender",
            "apps\\blender",
            "org.blender\u{7}",
        ] {
            assert!(
                matches!(
                    Application::identified(offered),
                    Err(NotAnApplication::NotAnIdentifier { .. })
                ),
                "{offered:?}"
            );
        }
    }

    /// **A name that could rewrite the line it is shown in is dropped, and the
    /// application stays.** Losing the name costs nothing — nothing is ever
    /// acted on by name — and refusing the application would let whoever
    /// packaged it decide what this machine can reach.
    #[test]
    fn a_name_that_cannot_be_shown_is_dropped_and_the_application_is_not() {
        let sneaky =
            Application::called("com.example.Mail", "Mail\nran: deleted everything").unwrap();
        assert_eq!(sneaky.identifier(), "com.example.Mail");
        assert_eq!(sneaky.name(), None);
        assert_eq!(sneaky.shown(&in_english()), "com.example.Mail");

        let unnamed = Application::called("com.example.Mail", "   ").unwrap();
        assert_eq!(unnamed.name(), None);
    }

    /// **Two applications can call themselves the same thing; no two share an
    /// identifier.** This is the whole argument for approving the identifier,
    /// written as the test that would catch somebody deciding otherwise.
    #[test]
    fn what_distinguishes_two_applications_is_never_the_name() {
        let honest = Application::called("com.example.Mail", "Mail").unwrap();
        let other = Application::called("com.acme.Mail", "Mail").unwrap();
        assert_eq!(honest.name(), other.name());
        assert_ne!(honest.identifier(), other.identifier());
        assert_ne!(honest.shown(&in_english()), other.shown(&in_english()));
    }

    /// The brackets are the language's; the two names inside them are the
    /// machine's and the packager's, and neither is translated.
    #[test]
    fn how_an_application_is_shown_is_the_readers_and_what_is_shown_is_not() {
        let strings = translated(&[(words::CALLED, "{called} – {application}")]);
        assert_eq!(
            Application::called("org.blender.Blender", "Blender")
                .unwrap()
                .shown(&strings),
            "Blender – org.blender.Blender"
        );
    }

    /// **An application this project packages is read in the person's own
    /// language**, and one somebody else packaged is not — the whole of
    /// ADR 0085's second decision, in one test.
    ///
    /// The German vocabulary here translates our word and has nothing to say
    /// about a packager's name, because there is nothing it could say: a name
    /// off a desktop entry has no key.
    #[test]
    fn our_own_application_is_translated_and_somebody_elses_is_not() {
        let strings = translated(&[(words::SETTINGS, "Einstellungen")]);

        let ours = Application::settings("org.alo.Settings").unwrap();
        assert_eq!(
            ours.name(),
            Some(&Called::InOurOwnWords(words::SETTINGS)),
            "ours carries the word rather than a string"
        );
        assert_eq!(ours.name().unwrap().shown(&strings), "Einstellungen");
        assert_eq!(
            ours.shown(&strings),
            "Einstellungen (org.alo.Settings)",
            "and the sentence around it is translated too"
        );

        let theirs = Application::called("org.blender.Blender", "Blender").unwrap();
        assert_eq!(theirs.name().unwrap().shown(&strings), "Blender");
        assert!(
            theirs.shown(&strings).contains("Blender"),
            "a packager's name is shown as it was given, in any language"
        );
    }

    /// **Nothing may guess which kind a name is**, which is what the type is
    /// for: two applications whose names read the same in English are still
    /// not the same name, because one of them changes in German and the other
    /// does not.
    #[test]
    fn two_names_that_read_alike_in_english_are_not_the_same_name() {
        let ours = Called::InOurOwnWords(words::SETTINGS);
        let theirs = Called::ByWhoeverPackagedIt("Settings".to_owned());
        assert_eq!(ours.shown(&in_english()), theirs.shown(&in_english()));
        assert_ne!(ours, theirs, "they are not interchangeable");

        let strings = translated(&[(words::SETTINGS, "Einstellungen")]);
        assert_ne!(
            ours.shown(&strings),
            theirs.shown(&strings),
            "and in German only one of them moves"
        );
    }

    /// **Our own name still answers with no vocabulary at all**, marked as the
    /// bug it is, the way every other sentence this crate says does. A machine
    /// that lost its words shows a key rather than nothing.
    #[test]
    fn our_own_name_without_the_words_answers_with_the_key() {
        let nothing = Strings::of(alo_strings::Vocabulary::empty());
        let shown = Application::settings("org.alo.Settings")
            .unwrap()
            .name()
            .unwrap()
            .shown(&nothing);
        assert!(shown.contains("applications.ours.settings"), "{shown}");
    }

    /// **A name of ours is only ever one of our words.** The road to
    /// [`Called::InOurOwnWords`] takes a [`Word`], and the only words a caller
    /// can hand this crate are the ones it declares — so an identifier cannot
    /// arrive dressed as a translated name.
    #[test]
    fn the_only_words_our_own_names_can_carry_are_this_crates() {
        let ours = Application::ours("org.alo.Settings", words::SETTINGS).unwrap();
        assert_eq!(ours.name(), Some(&Called::InOurOwnWords(words::SETTINGS)));
        assert!(
            words::EVERY_WORD.contains(&words::SETTINGS),
            "the word a name of ours carries is one this crate declares, so \
             `alo-saying` collects it and a translator is given it"
        );
    }

    /// An identifier is still checked when the name is ours: the exception is
    /// about the name and reaches nothing else.
    #[test]
    fn our_own_application_is_refused_an_identifier_no_verb_could_name() {
        assert!(Application::settings("/usr/bin/settings").is_err());
        assert!(Application::ours("", words::SETTINGS).is_err());
    }
}
