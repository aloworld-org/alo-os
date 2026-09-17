//! A keyboard layout as the rented data names it: a layout, and sometimes a
//! variant of it.
//!
//! **No layout is described here.** What letter each key prints, which key is
//! dead and which level AltGr reaches are `xkeyboard-config`'s, rented and
//! unmodified (ADR 0011, and `CLAUDE.md`'s *engines are configured, never
//! patched*). A [`Layout`] is the name of one of its layouts and nothing else,
//! and [`crate::rented`] is the only thing that can say whether such a layout
//! exists on this machine.
//!
//! # Why this is a checked type and not a `String`
//!
//! Because the name reaches a rented component, and because it comes back off
//! a disk: a person's own `keyboards.toml` is a file they may edit by hand.
//! A name with a space, a slash or a newline in it is refused here, once, so
//! that nothing further down has to wonder.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A keyboard layout, and the variant of it when there is one.
///
/// Written the way the rented rules write it: `de`, or `us(intl)`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Layout {
    /// The layout's name, such as `de`.
    name: String,
    /// The variant of it, such as `intl`, when the layout is not the plain one.
    variant: Option<String>,
}

/// The longest a name or a variant may be.
///
/// The longest either is in the rented rules today is fifteen characters; this
/// is generous and is a bound rather than a prediction.
const AT_MOST: usize = 32;

impl Layout {
    /// A plain layout, such as `de`.
    ///
    /// # Errors
    /// [`LayoutError`] when the name is not one a rules file could hold.
    pub fn named(name: &str) -> Result<Self, LayoutError> {
        Ok(Self {
            name: checked(name)?,
            variant: None,
        })
    }

    /// A variant of a layout, such as `us(intl)`.
    ///
    /// # Errors
    /// [`LayoutError`] when either part is not one a rules file could hold.
    pub fn variant_of(name: &str, variant: &str) -> Result<Self, LayoutError> {
        Ok(Self {
            name: checked(name)?,
            variant: Some(checked(variant)?),
        })
    }

    /// One this crate offers with a language, built without being checked.
    ///
    /// The offers in [`crate::offering`] are written into this crate, so they
    /// are the compiler's rather than a person's — and
    /// `every_offered_layout_is_a_layout` puts every one of them through
    /// [`Layout::named`] anyway, which is the shape `alo_shortcuts`' shipped
    /// chords are held to.
    pub(crate) fn offered(name: &'static str, variant: Option<&'static str>) -> Self {
        Self {
            name: name.to_owned(),
            variant: variant.map(ToOwned::to_owned),
        }
    }

    /// The layout's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The variant, when this is a variant of a layout.
    #[must_use]
    pub fn variant(&self) -> Option<&str> {
        self.variant.as_deref()
    }

    /// The mark shown in the status area while more than one keyboard is in
    /// use: `DE`, `FR`, `GR`.
    ///
    /// **Not a sentence and never translated**, for the reason
    /// `alo_shortcuts::Key::mark` is not one: it is what is printed in a small
    /// square, the same in every language, and a translator asked to render it
    /// would be being asked to translate an abbreviation of a thing that has no
    /// name in their language either. Two or three characters, upper case,
    /// which is the convention every desktop that has such a square uses.
    #[must_use]
    pub fn badge(&self) -> String {
        self.name
            .chars()
            .take(3)
            .flat_map(char::to_uppercase)
            .collect()
    }
}

/// How the rented rules write a layout: `de`, or `us(intl)`.
impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.variant {
            Some(variant) => write!(f, "{}({variant})", self.name),
            None => f.write_str(&self.name),
        }
    }
}

impl From<Layout> for String {
    fn from(layout: Layout) -> Self {
        layout.to_string()
    }
}

impl TryFrom<String> for Layout {
    type Error = LayoutError;

    fn try_from(written: String) -> Result<Self, Self::Error> {
        match written.split_once('(') {
            None => Self::named(&written),
            Some((name, rest)) => match rest.strip_suffix(')') {
                Some(variant) => Self::variant_of(name, variant),
                None => Err(LayoutError::NotClosed { written }),
            },
        }
    }
}

/// A name that could not be a layout's.
///
/// Said in English to whoever is reading a log, not to a person in their own
/// language: what a person is told about their own hand-edited file is
/// [`crate::FileNotRead`], which says the file and the line rather than this.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum LayoutError {
    /// An empty name.
    #[error("a keyboard layout has a name and this one is empty")]
    Empty,
    /// A name too long to be one.
    #[error("{written} is {how_many} characters, and no keyboard layout is named that")]
    TooLong {
        /// What was written.
        written: String,
        /// How long it was.
        how_many: usize,
    },
    /// A character a rules file could not hold.
    #[error(
        "{written} is not how a keyboard layout is named: {what} is not a letter, a digit, - or _"
    )]
    NotAName {
        /// What was written.
        written: String,
        /// The first character that is not one.
        what: char,
    },
    /// A variant opened with `(` and never closed.
    #[error("{written} opens a variant with ( and never closes it")]
    NotClosed {
        /// What was written.
        written: String,
    },
}

/// One part of a layout's name, checked.
fn checked(part: &str) -> Result<String, LayoutError> {
    if part.is_empty() {
        return Err(LayoutError::Empty);
    }
    if part.len() > AT_MOST {
        return Err(LayoutError::TooLong {
            written: part.to_owned(),
            how_many: part.len(),
        });
    }
    if let Some(what) = part
        .chars()
        .find(|c| !(c.is_ascii_alphanumeric() || *c == '-' || *c == '_'))
    {
        return Err(LayoutError::NotAName {
            written: part.to_owned(),
            what,
        });
    }
    Ok(part.to_owned())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A layout is written the way the rented rules write one, and reads back
    /// as the same layout.
    #[test]
    fn a_layout_is_written_the_way_the_rules_write_one() {
        assert_eq!(Layout::named("de").unwrap().to_string(), "de");
        assert_eq!(
            Layout::variant_of("us", "intl").unwrap().to_string(),
            "us(intl)"
        );
        for written in ["de", "us(intl)", "gr(polytonic)", "latam", "bas_phonetic"] {
            let layout = Layout::try_from(written.to_owned()).unwrap();
            assert_eq!(layout.to_string(), written);
        }
    }

    /// **A name that is not one is refused**, so nothing further down has to
    /// wonder whether the string it is holding could be a shell argument, a
    /// path or a second option.
    #[test]
    fn a_name_that_could_not_be_a_layouts_is_refused() {
        assert_eq!(Layout::named(""), Err(LayoutError::Empty));
        for written in ["de de", "../etc/passwd", "de,fr", "de\n", "de:fr", "de$"] {
            assert!(
                matches!(Layout::named(written), Err(LayoutError::NotAName { .. })),
                "{written}"
            );
        }
        assert!(matches!(
            Layout::named(&"d".repeat(AT_MOST + 1)),
            Err(LayoutError::TooLong { .. })
        ));
        assert!(matches!(
            Layout::try_from("us(intl".to_owned()),
            Err(LayoutError::NotClosed { .. })
        ));
        assert!(matches!(
            Layout::try_from("us(int l)".to_owned()),
            Err(LayoutError::NotAName { .. })
        ));
    }

    /// The mark in the status area is short, upper case, and the same in every
    /// language.
    #[test]
    fn the_badge_is_a_short_mark() {
        assert_eq!(Layout::named("de").unwrap().badge(), "DE");
        assert_eq!(Layout::variant_of("us", "intl").unwrap().badge(), "US");
        assert_eq!(Layout::named("latam").unwrap().badge(), "LAT");
    }
}
