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

use alo_strings::{Filling, Strings};

use crate::refusing::NotAnApplication;
use crate::words;

/// One application this machine has.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Application {
    /// What this machine knows it by, and what a grant is made over. Matched
    /// exactly, with no case folding.
    identifier: String,
    /// What it calls itself, when that is something a person can be shown.
    called: Option<String>,
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
            called: showable(called),
        })
    }

    /// What this machine knows it by, and what a grant is made over.
    #[must_use]
    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    /// What it calls itself, when that is something a person can be shown.
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        self.called.as_deref()
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
                    &Filling::of("called", called.clone())
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
        assert_eq!(blender.name(), Some("Blender"));
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
}
