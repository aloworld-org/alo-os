//! **Which application a name means** (`docs/contracts/terminal-commands.md`,
//! *Naming the application* and *Reserved words*).
//!
//! A name is read in this order: the reserved words `list`, `show`, `front` and
//! `alo`; then a full identifier (`org.gnome.TextEditor`); then a short name,
//! which is the last part of an identifier in lower case (`texteditor`).
//! **Never the name a menu shows**, because two applications may both call
//! themselves *Text Editor* and the one acted on must be the one meant.
//!
//! Two installed applications sharing a short name make it name neither: it is
//! refused, and both identifiers are listed. Nothing is guessed.

use alo_applications::{Application, Installed};
use alo_strings::{Filling, Said, Strings};

use crate::typed::{ALO, FRONT, RESERVED};
use crate::words;

/// What a name in the application's place means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Named<'a> {
    /// `front`: the window the person was working in before this terminal.
    Front,
    /// `alo`: alo OS's own surfaces.
    Alo,
    /// One installed application.
    Installed(&'a Application),
}

/// Why a name means no application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotNamed {
    /// Nothing installed answers to it.
    NotInstalled {
        /// What was typed.
        typed: String,
    },
    /// More than one installed application has it as its short name.
    Shared {
        /// What was typed.
        typed: String,
        /// Every identifier it could mean, sorted.
        identifiers: Vec<String>,
    },
}

impl NotNamed {
    /// What the person is told.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::NotInstalled { typed } => strings.say(
                &words::NOT_INSTALLED.key(),
                &Filling::of(words::TYPED, typed.clone()),
            ),
            Self::Shared { typed, identifiers } => strings.say(
                &words::SHARED_SHORT_NAME.key(),
                &Filling::of(words::TYPED, typed.clone()).and(words::LIST, identifiers.join(", ")),
            ),
        }
    }
}

/// How a person reaches an installed application by name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reached {
    /// By its short name.
    ByItsShortName(String),
    /// Only by its identifier, because its short name is one of the words
    /// alo-app keeps for itself.
    AsIdentifierBecauseReserved {
        /// The short name it cannot be reached by.
        short: String,
    },
    /// Only by its identifier, because another installed application has the
    /// same short name.
    AsIdentifierBecauseShared {
        /// The short name it shares.
        short: String,
    },
}

impl Reached {
    /// What a person types to reach it.
    #[must_use]
    pub fn typed<'a>(&'a self, identifier: &'a str) -> &'a str {
        match self {
            Self::ByItsShortName(short) => short,
            Self::AsIdentifierBecauseReserved { .. } | Self::AsIdentifierBecauseShared { .. } => {
                identifier
            }
        }
    }

    /// What a person is told about how to reach it.
    #[must_use]
    pub fn said(&self, identifier: &str, strings: &Strings) -> Said {
        match self {
            Self::ByItsShortName(short) => strings.say(
                &words::TYPE_SHORT.key(),
                &Filling::of(words::SHORT, short.clone()),
            ),
            Self::AsIdentifierBecauseReserved { short } => strings.say(
                &words::TYPE_IDENTIFIER_RESERVED.key(),
                &Filling::of(words::IDENTIFIER, identifier.to_owned())
                    .and(words::SHORT, short.clone()),
            ),
            Self::AsIdentifierBecauseShared { short } => strings.say(
                &words::TYPE_IDENTIFIER_SHARED.key(),
                &Filling::of(words::IDENTIFIER, identifier.to_owned())
                    .and(words::SHORT, short.clone()),
            ),
        }
    }
}

/// The short name of an identifier: its last part, in lower case.
#[must_use]
pub fn short_name(identifier: &str) -> String {
    identifier
        .rsplit('.')
        .next()
        .unwrap_or(identifier)
        .to_lowercase()
}

/// Every installed application whose short name is this, sorted by identifier.
fn sharing<'a>(short: &str, installed: &'a Installed) -> Vec<&'a Application> {
    let mut found: Vec<&Application> = installed
        .all()
        .filter(|application| short_name(application.identifier()) == short)
        .collect();
    found.sort_by(|one, other| one.identifier().cmp(other.identifier()));
    found
}

/// What `typed` means, among what is installed now.
///
/// # Errors
/// [`NotNamed`]: no application, or more than one.
pub fn named<'a>(typed: &str, installed: &'a Installed) -> Result<Named<'a>, NotNamed> {
    match typed {
        FRONT => return Ok(Named::Front),
        ALO => return Ok(Named::Alo),
        _ => {}
    }
    if let Some(application) = installed.knows(typed) {
        return Ok(Named::Installed(application));
    }
    let not_installed = || NotNamed::NotInstalled {
        typed: typed.to_owned(),
    };
    // A reserved word is never a short name, and a name with a dot in it is an
    // identifier or nothing.
    if RESERVED.contains(&typed) || typed.contains('.') {
        return Err(not_installed());
    }
    match sharing(typed, installed).as_slice() {
        [] => Err(not_installed()),
        [only] => Ok(Named::Installed(only)),
        several => Err(NotNamed::Shared {
            typed: typed.to_owned(),
            identifiers: several
                .iter()
                .map(|application| application.identifier().to_owned())
                .collect(),
        }),
    }
}

/// How this installed application is reached by a person, now.
#[must_use]
pub fn how_it_is_reached(application: &Application, installed: &Installed) -> Reached {
    let short = short_name(application.identifier());
    if RESERVED.contains(&short.as_str()) {
        return Reached::AsIdentifierBecauseReserved { short };
    }
    if sharing(&short, installed).len() > 1 {
        return Reached::AsIdentifierBecauseShared { short };
    }
    Reached::ByItsShortName(short)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A machine with these installed.
    fn holding(identifiers: &[&str]) -> Installed {
        Installed::holding(
            identifiers
                .iter()
                .map(|identifier| Application::identified(identifier).unwrap()),
        )
    }

    #[test]
    fn a_short_name_is_the_last_part_in_lower_case() {
        assert_eq!(short_name("org.gnome.TextEditor"), "texteditor");
        assert_eq!(short_name("org.mozilla.firefox"), "firefox");
        assert_eq!(short_name("firefox"), "firefox");
    }

    #[test]
    fn an_identifier_and_a_short_name_both_reach_one_application() {
        let installed = holding(&["org.gnome.TextEditor", "org.gnome.Loupe"]);
        for typed in ["org.gnome.TextEditor", "texteditor"] {
            assert!(matches!(
                named(typed, &installed).unwrap(),
                Named::Installed(application) if application.identifier() == "org.gnome.TextEditor"
            ));
        }
    }

    /// **Never the menu name, never a guess**: an unknown name is refused, and
    /// a short name two applications share is refused naming both.
    #[test]
    fn an_unknown_or_shared_name_is_refused_and_nothing_is_guessed() {
        let installed = holding(&["org.gnome.TextEditor", "org.kde.TextEditor"]);
        assert_eq!(
            named("texteditor", &installed).unwrap_err(),
            NotNamed::Shared {
                typed: "texteditor".to_owned(),
                identifiers: vec![
                    "org.gnome.TextEditor".to_owned(),
                    "org.kde.TextEditor".to_owned()
                ],
            }
        );
        for typed in ["Text Editor", "TextEditor", "org.gnome.Nope", "gedit"] {
            assert_eq!(
                named(typed, &installed).unwrap_err(),
                NotNamed::NotInstalled {
                    typed: typed.to_owned()
                },
                "{typed}"
            );
        }
        assert!(matches!(
            named("org.kde.TextEditor", &installed).unwrap(),
            Named::Installed(application) if application.identifier() == "org.kde.TextEditor"
        ));
        assert_eq!(
            how_it_is_reached(installed.knows("org.kde.TextEditor").unwrap(), &installed),
            Reached::AsIdentifierBecauseShared {
                short: "texteditor".to_owned()
            }
        );
    }

    /// **A reserved word is read first**, so an application whose short name
    /// is one is reached only by its identifier — and is told so.
    #[test]
    fn a_reserved_word_is_read_before_any_application() {
        let installed = holding(&["com.example.Front", "com.example.Alo", "com.example.List"]);
        assert_eq!(named("front", &installed).unwrap(), Named::Front);
        assert_eq!(named("alo", &installed).unwrap(), Named::Alo);
        assert!(named("list", &installed).is_err());
        let front = installed.knows("com.example.Front").unwrap();
        assert!(
            matches!(named("com.example.Front", &installed).unwrap(), Named::Installed(found) if found == front)
        );
        let reached = how_it_is_reached(front, &installed);
        assert_eq!(
            reached,
            Reached::AsIdentifierBecauseReserved {
                short: "front".to_owned()
            }
        );
        assert_eq!(reached.typed("com.example.Front"), "com.example.Front");
    }
}
