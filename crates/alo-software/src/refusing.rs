//! Why an application was not installed, updated or removed, as a value.
//!
//! One type for all three, because a person meets them in one place and most of
//! them are the same fact whichever was asked for: a place that is not set up,
//! kept out by a rule or not checking signatures refuses an installation and an
//! update in the same words. Each is decided without words and worded by
//! [`NotDone::said`], so a refusal never depends on a vocabulary having been
//! loaded — the rule every refusal in this workspace keeps.

use alo_applications::Application;
use alo_strings::{Filling, Said, Strings};

use crate::bound::SetBy;
use crate::words::{self, Word};

/// Why nothing was installed, updated or removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotDone {
    /// The place named is not one this machine installs from.
    NotEnabled {
        /// The place, as it was named.
        source: String,
    },
    /// A rule keeps the place out.
    OutsideTheBound {
        /// The place.
        source: String,
        /// Who set the rule.
        set_by: SetBy,
    },
    /// The place is set up with signature checking turned off.
    NotVerified {
        /// The place.
        source: String,
    },
    /// The place has no network address this machine can read.
    NowhereToReach {
        /// The place.
        source: String,
    },
    /// What arrived could not be shown to come from the place.
    SignatureNotShown {
        /// The place.
        source: String,
        /// The application it was for.
        application: String,
    },
    /// The place has no application by that identifier.
    NotOffered {
        /// The place.
        source: String,
        /// The application asked for.
        application: String,
    },
    /// The application is already here.
    AlreadyInstalled {
        /// The application.
        application: String,
    },
    /// The application is not here.
    NotInstalled {
        /// The application.
        application: String,
    },
    /// The application is open, so its update waits.
    StillOpen {
        /// The application.
        application: String,
    },
    /// The rented tool did not respond, or answered in a way nothing here reads.
    ///
    /// What it said is kept for whoever is fixing the machine, and never put
    /// in front of the person: it is the tool's English, naming the tool.
    DidNotAnswer {
        /// What the tool said, verbatim.
        said: String,
    },
}

impl NotDone {
    /// A place nothing here installs from, named as it was given.
    ///
    /// A name that cannot be shown on one line is shown with its unshowable
    /// characters replaced, because the refusal is about that text and the
    /// person has to be able to read which text it was.
    #[must_use]
    pub fn not_enabled(source: &str) -> Self {
        Self::NotEnabled {
            source: showable(source),
        }
    }

    /// The application, in the words an application's refusal carries.
    #[must_use]
    pub fn not_installed(application: &Application) -> Self {
        Self::NotInstalled {
            application: application.identifier().to_owned(),
        }
    }

    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(&self) -> Word {
        match self {
            Self::NotEnabled { .. } => words::NOT_ENABLED,
            Self::OutsideTheBound {
                set_by: SetBy::AnAdministrator,
                ..
            } => words::OUTSIDE_THE_BOUND_BY_AN_ADMINISTRATOR,
            Self::OutsideTheBound {
                set_by: SetBy::ThisMachine,
                ..
            } => words::OUTSIDE_THE_BOUND_BY_THIS_PERSON,
            Self::NotVerified { .. } => words::NOT_VERIFIED,
            Self::NowhereToReach { .. } => words::NOWHERE_TO_REACH,
            Self::SignatureNotShown { .. } => words::SIGNATURE_NOT_SHOWN,
            Self::NotOffered { .. } => words::NOT_OFFERED,
            Self::AlreadyInstalled { .. } => words::ALREADY_INSTALLED,
            Self::NotInstalled { .. } => words::NOT_INSTALLED,
            Self::StillOpen { .. } => words::STILL_OPEN,
            Self::DidNotAnswer { .. } => words::DID_NOT_ANSWER,
        }
    }

    /// What this says, in the language the person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = match self {
            Self::NotEnabled { source }
            | Self::OutsideTheBound { source, .. }
            | Self::NotVerified { source }
            | Self::NowhereToReach { source } => Filling::of(words::SOURCE, source.clone()),
            Self::SignatureNotShown {
                source,
                application,
            }
            | Self::NotOffered {
                source,
                application,
            } => Filling::of(words::SOURCE, source.clone())
                .and(words::APPLICATION, application.clone()),
            Self::AlreadyInstalled { application }
            | Self::NotInstalled { application }
            | Self::StillOpen { application } => {
                Filling::of(words::APPLICATION, application.clone())
            }
            Self::DidNotAnswer { .. } => Filling::nothing(),
        };
        strings.say(&self.word().key(), &filling)
    }
}

/// Text with every character that could not be shown on one line replaced.
fn showable(text: &str) -> String {
    text.trim()
        .chars()
        .map(|c| if c.is_control() { '\u{fffd}' } else { c })
        .collect()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    fn every_refusal() -> Vec<NotDone> {
        let source = || "acme-apps".to_owned();
        let application = || "org.gnome.TextEditor".to_owned();
        vec![
            NotDone::NotEnabled { source: source() },
            NotDone::OutsideTheBound {
                source: source(),
                set_by: SetBy::AnAdministrator,
            },
            NotDone::OutsideTheBound {
                source: source(),
                set_by: SetBy::ThisMachine,
            },
            NotDone::NotVerified { source: source() },
            NotDone::NowhereToReach { source: source() },
            NotDone::SignatureNotShown {
                source: source(),
                application: application(),
            },
            NotDone::NotOffered {
                source: source(),
                application: application(),
            },
            NotDone::AlreadyInstalled {
                application: application(),
            },
            NotDone::NotInstalled {
                application: application(),
            },
            NotDone::StillOpen {
                application: application(),
            },
            NotDone::DidNotAnswer {
                said: "error: flatpak exploded".to_owned(),
            },
        ]
    }

    /// **Every refusal reads, fills every gap, and names what it refused** —
    /// and none of them says the same as another.
    #[test]
    fn every_refusal_reads_and_names_what_it_refused() {
        let strings = in_english();
        let mut texts = Vec::new();
        for refusal in every_refusal() {
            let said = refusal.said(&strings);
            assert!(!said.is_a_bug(), "{refusal:?}: {said}");
            assert!(said.unfilled().is_empty(), "{refusal:?}: {said}");
            texts.push(said.into_text());
        }
        let mut distinct = texts.clone();
        distinct.sort();
        distinct.dedup();
        assert_eq!(distinct.len(), texts.len());
    }

    /// **The organisation's rule and the owner's own are told apart**, because
    /// ADR 0016 says a person is told who set a rule, and a restrictive rule is
    /// never on its own evidence an organisation did.
    #[test]
    fn who_set_the_rule_is_what_the_refusal_says() {
        let strings = in_english();
        let by_them = NotDone::OutsideTheBound {
            source: "flathub".to_owned(),
            set_by: SetBy::AnAdministrator,
        }
        .said(&strings);
        assert!(by_them.text().contains("organisation"), "{by_them}");
        let by_you = NotDone::OutsideTheBound {
            source: "flathub".to_owned(),
            set_by: SetBy::ThisMachine,
        }
        .said(&strings);
        assert!(by_you.text().contains("No organisation"), "{by_you}");
    }

    /// **What the rented tool said never reaches the person.** It names the
    /// tool, and it is English nobody translated.
    #[test]
    fn what_the_tool_said_is_not_what_the_person_reads() {
        let said = NotDone::DidNotAnswer {
            said: "error: flatpak exploded".to_owned(),
        }
        .said(&in_english());
        assert!(!said.text().to_lowercase().contains("flatpak"), "{said}");
        assert!(!said.text().contains("exploded"), "{said}");
    }

    /// **A name that could rewrite the line is shown, not obeyed.**
    #[test]
    fn a_place_named_with_a_newline_is_shown_on_one_line() {
        let said = NotDone::not_enabled("acme\nInstalled everything").said(&in_english());
        assert!(!said.text().contains('\n'), "{said}");
    }

    /// **The words around a name are the reader's and the name is not.**
    #[test]
    fn the_words_are_translated_and_the_names_are_not() {
        let strings = translated(&[(
            words::STILL_OPEN,
            "{application} ist geöffnet und wurde nicht aktualisiert. Schließen Sie es und \
             wählen Sie die Aktualisierung erneut",
        )]);
        let said = NotDone::StillOpen {
            application: "org.gnome.TextEditor".to_owned(),
        }
        .said(&strings);
        assert!(said.is_translated());
        assert!(
            said.text().starts_with("org.gnome.TextEditor ist"),
            "{said}"
        );
    }

    /// **A refusal never depends on a string table.**
    #[test]
    fn a_refusal_without_the_words_still_names_its_key() {
        let said =
            NotDone::not_installed(&Application::identified("org.gnome.TextEditor").unwrap())
                .said(&Strings::of(alo_strings::Vocabulary::empty()));
        assert!(said.is_a_bug());
        assert!(
            said.text().contains("software.refused.not-installed"),
            "{said}"
        );
    }
}
