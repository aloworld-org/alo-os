//! **Which application a name means** (`docs/contracts/terminal-commands.md`,
//! *Naming the application* and *Reserved words*).
//!
//! A name is read in this order: the reserved words `list`, `show`, `front` and
//! `alo`; then a full identifier (`org.gnome.TextEditor`); then a short name,
//! which is the last part of an identifier in lower case (`texteditor`), less
//! a version number it begins with (`libreoffice26.2-writer` is `writer`).
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

/// The short name of an identifier: its last part, in lower case — **less a
/// version number and the hyphen after it** where that part begins with one.
///
/// The second half is for identifiers that are not reverse-DNS.
/// `libreoffice26.2-writer` has its last dot inside a version number, so its
/// last part is `2-writer`; its short name is `writer`. It never changes a
/// reverse-DNS identifier, because an element of one may not begin with a
/// digit (the D-Bus naming rule application identifiers follow).
#[must_use]
pub fn short_name(identifier: &str) -> String {
    let last = the_last_part(identifier);
    match without_a_version(&last) {
        Some(rest) => rest.to_owned(),
        None => last,
    }
}

/// The last part of an identifier, in lower case: the short name as it was
/// read until 2026-11-08, which still reaches the application it reached.
fn the_last_part(identifier: &str) -> String {
    identifier
        .rsplit('.')
        .next()
        .unwrap_or(identifier)
        .to_lowercase()
}

/// `part` less the digits it begins with and the hyphen after them, when it
/// begins that way and something is left.
fn without_a_version(part: &str) -> Option<&str> {
    let digits = part.bytes().take_while(u8::is_ascii_digit).count();
    let rest = part.get(digits..)?.strip_prefix('-')?;
    (digits > 0 && !rest.is_empty()).then_some(rest)
}

/// Whether this application answers to `short`: by its short name, or by the
/// last part it answered to before (`2-writer`), which is kept so a name that
/// reached an application still does.
fn answers_to(application: &Application, short: &str) -> bool {
    let identifier = application.identifier();
    short_name(identifier) == short || the_last_part(identifier) == short
}

/// Every installed application that answers to this short name, sorted by
/// identifier.
fn sharing<'a>(short: &str, installed: &'a Installed) -> Vec<&'a Application> {
    let mut found: Vec<&Application> = installed
        .all()
        .filter(|application| answers_to(application, short))
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

    /// **A version number the last part begins with is left off**, as the
    /// walk of 2026-11-08 found LibreOffice needs — and nothing else changes:
    /// reverse-DNS identifiers, hyphens elsewhere, digits with no hyphen, and
    /// a hyphen with nothing after it all read as they did.
    #[test]
    fn a_version_the_last_part_begins_with_is_left_off() {
        assert_eq!(short_name("libreoffice26.2-writer"), "writer");
        assert_eq!(short_name("libreoffice26.2-calc"), "calc");
        assert_eq!(short_name("libreoffice26.2-startcenter"), "startcenter");
        assert_eq!(short_name("libreoffice26.2.1-base"), "base");
        assert_eq!(short_name("org.gnome.font-viewer"), "font-viewer");
        assert_eq!(short_name("libreoffice-writer"), "libreoffice-writer");
        assert_eq!(short_name("org.example.3d"), "3d");
        assert_eq!(short_name("tool1.2-"), "2-");
        assert_eq!(short_name("org.example.App-2"), "app-2");
    }

    /// **The new short name and the old one both reach it**, so a script
    /// written against `2-writer` keeps working; `list` offers `writer`.
    #[test]
    fn the_new_and_the_old_short_name_both_reach_it() {
        let installed = holding(&["libreoffice26.2-writer", "libreoffice26.2-calc"]);
        for typed in ["writer", "2-writer", "libreoffice26.2-writer"] {
            assert!(
                matches!(
                    named(typed, &installed).unwrap(),
                    Named::Installed(application)
                        if application.identifier() == "libreoffice26.2-writer"
                ),
                "{typed}"
            );
        }
        assert_eq!(
            how_it_is_reached(
                installed.knows("libreoffice26.2-writer").unwrap(),
                &installed
            ),
            Reached::ByItsShortName("writer".to_owned())
        );
    }

    /// **The clash rule still refuses**: two LibreOffice versions both answer
    /// to `writer`, so it names neither and lists both; each one's old name,
    /// which differs, still reaches only it.
    #[test]
    fn two_versions_sharing_writer_are_refused_naming_both() {
        let installed = holding(&["libreoffice25.8-writer", "libreoffice26.2-writer"]);
        assert_eq!(
            named("writer", &installed).unwrap_err(),
            NotNamed::Shared {
                typed: "writer".to_owned(),
                identifiers: vec![
                    "libreoffice25.8-writer".to_owned(),
                    "libreoffice26.2-writer".to_owned()
                ],
            }
        );
        assert!(matches!(
            named("8-writer", &installed).unwrap(),
            Named::Installed(application) if application.identifier() == "libreoffice25.8-writer"
        ));
        assert_eq!(
            how_it_is_reached(
                installed.knows("libreoffice26.2-writer").unwrap(),
                &installed
            ),
            Reached::AsIdentifierBecauseShared {
                short: "writer".to_owned()
            }
        );
    }

    /// **A clash with a reverse-DNS application is refused too**, and is the
    /// one case where a name that reached something before no longer does:
    /// `writer` reached `org.example.Writer` alone until LibreOffice answered
    /// to it as well. It is refused naming both rather than guessed.
    #[test]
    fn a_clash_with_a_reverse_dns_name_is_refused() {
        let installed = holding(&["org.example.Writer", "libreoffice26.2-writer"]);
        assert!(matches!(
            named("writer", &installed).unwrap_err(),
            NotNamed::Shared { identifiers, .. } if identifiers.len() == 2
        ));
        for identifier in ["org.example.Writer", "libreoffice26.2-writer"] {
            assert!(matches!(
                named(identifier, &installed).unwrap(),
                Named::Installed(application) if application.identifier() == identifier
            ));
        }
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
