//! The answer to *what opens this*: an application, and the reason it was the
//! one.
//!
//! An [`Opener`] is a name and a reason, and nothing more. It launches nothing
//! and holds no handle on anything that could: running the application is a
//! verb with a grant, or a person's click, and neither is this crate's.
//!
//! # The reason is part of the answer
//!
//! `docs/features.md` promises *what opens what, changeable by a person*, and a
//! person cannot change what they cannot see the cause of. So the answer always
//! says which it was — the person's own choice, or the application's
//! declaration — and, when the person's choice could not be used because what
//! they chose is no longer installed, it says that too rather than quietly
//! answering with something else.

use alo_opening::Kind;
use alo_strings::{Filling, Said, Strings};

use crate::application::Application;
use crate::words;

/// Why this application is the one that opens the kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Because {
    /// The person chose it for this kind.
    ThePersonChoseIt,
    /// It declares the kind, and the person chose nothing for it.
    ItDeclaresIt,
    /// It declares the kind, and what the person chose for it is not
    /// installed.
    ItDeclaresItAndTheChoiceIsNotInstalled {
        /// The identifier the person chose.
        chosen: String,
    },
}

/// The application that opens a kind of file, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Opener {
    /// The application, as this machine has it.
    application: Application,
    /// The kind of file it opens.
    kind: Kind,
    /// Why it is the one.
    because: Because,
}

impl Opener {
    /// This application opens this kind, for this reason.
    pub(crate) const fn new(application: Application, kind: Kind, because: Because) -> Self {
        Self {
            application,
            kind,
            because,
        }
    }

    /// The application that opens it.
    #[must_use]
    pub const fn application(&self) -> &Application {
        &self.application
    }

    /// The kind of file, as read from its bytes.
    #[must_use]
    pub const fn kind(&self) -> Kind {
        self.kind
    }

    /// Why this application is the one.
    #[must_use]
    pub const fn because(&self) -> &Because {
        &self.because
    }

    /// Whether this was the person's own choice, rather than a declaration.
    #[must_use]
    pub const fn was_chosen(&self) -> bool {
        matches!(self.because, Because::ThePersonChoseIt)
    }

    /// What opens it and why, in the language the person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = Filling::of("application", self.application.identifier().to_owned())
            .and_said("what", &self.kind.said(strings));
        match &self.because {
            Because::ThePersonChoseIt => strings.say(&words::OPENS_CHOSEN.key(), &filling),
            Because::ItDeclaresIt => strings.say(&words::OPENS_DECLARED.key(), &filling),
            Because::ItDeclaresItAndTheChoiceIsNotInstalled { chosen } => strings.say(
                &words::OPENS_DECLARED_INSTEAD.key(),
                &filling.and("chosen", chosen.clone()),
            ),
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
    use crate::testing::in_english;

    /// **Each reason is its own sentence, and each names the application by
    /// its identifier and the kind in words.**
    #[test]
    fn every_reason_says_which_it_was() {
        let strings = in_english();
        let papers = Application::called("org.gnome.Papers", "Papers").unwrap();
        let mut seen = Vec::new();
        for because in [
            Because::ThePersonChoseIt,
            Because::ItDeclaresIt,
            Because::ItDeclaresItAndTheChoiceIsNotInstalled {
                chosen: "org.kde.okular".to_owned(),
            },
        ] {
            let opener = Opener::new(papers.clone(), Kind::Pdf, because.clone());
            let said = opener.said(&strings);
            assert!(!said.is_a_bug(), "{because:?}: {said}");
            assert!(said.unfilled().is_empty(), "{because:?}: {said}");
            assert!(said.text().contains("org.gnome.Papers"), "{said}");
            assert!(said.text().contains("PDF"), "{said}");
            assert_eq!(opener.was_chosen(), because == Because::ThePersonChoseIt);
            seen.push(said.into_text());
        }
        assert!(seen.iter().any(|said| said.contains("you chose")));
        assert!(seen.iter().any(|said| said.contains("org.kde.okular")));
        seen.dedup();
        assert_eq!(seen.len(), 3);
    }
}
