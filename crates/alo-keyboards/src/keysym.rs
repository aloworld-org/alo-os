//! One key, named the way the rented tables name it.
//!
//! `dead_diaeresis`, `Multi_key`, `u`, `U0142`. These names are the rented
//! keyboard data's, not ours: the compositor is handed one by the input library
//! for every key that goes down, and the compose table is written in the same
//! names. A [`Keysym`] is that name, checked once so that nothing further down
//! has to.
//!
//! **No sentence anywhere says one of these.** A person is never shown
//! `dead_diaeresis`; they press a key and `ü` appears. The name exists so two
//! rented components can agree, which is why this type has a `Display` for a
//! log and no [`alo_strings`] word at all.

use std::fmt;

/// The longest name the rented tables use, with room to spare.
const AT_MOST: usize = 64;

/// One key, as the rented keyboard data names it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Keysym(String);

impl Keysym {
    /// This name, if it is one.
    ///
    /// # Errors
    /// [`KeysymError`] when it is empty, too long, or holds a character no
    /// keysym name holds.
    pub fn named(name: &str) -> Result<Self, KeysymError> {
        if name.is_empty() {
            return Err(KeysymError::Empty);
        }
        if name.len() > AT_MOST {
            return Err(KeysymError::TooLong {
                written: name.to_owned(),
            });
        }
        if let Some(what) = name
            .chars()
            .find(|c| !(c.is_ascii_alphanumeric() || *c == '_'))
        {
            return Err(KeysymError::NotAName {
                written: name.to_owned(),
                what,
            });
        }
        Ok(Self(name.to_owned()))
    }

    /// The name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Keysym {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A name that is not a keysym's.
///
/// English, for a log: nothing a person reads names a key this way.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum KeysymError {
    /// An empty name.
    #[error("a key has a name and this one is empty")]
    Empty,
    /// A name longer than any the rented tables use.
    #[error("{written} is too long to be the name of a key")]
    TooLong {
        /// What was written.
        written: String,
    },
    /// A character no keysym name holds.
    #[error("{written} is not the name of a key: {what} is not a letter, a digit or _")]
    NotAName {
        /// What was written.
        written: String,
        /// The first character that is not one.
        what: char,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The names the rented tables actually use are names.
    #[test]
    fn the_names_the_rented_tables_use_are_names() {
        for name in [
            "u",
            "Multi_key",
            "dead_diaeresis",
            "dead_doubleacute",
            "U0142",
            "KP_Divide",
            "space",
        ] {
            assert_eq!(
                Keysym::named(name).map(|key| key.name().to_owned()),
                Ok(name.to_owned())
            );
        }
    }

    /// **A name that is not one is refused**, which is what stops a broken line
    /// of a table from becoming a key nothing will ever press.
    #[test]
    fn a_name_that_is_not_a_keys_is_refused() {
        assert_eq!(Keysym::named(""), Err(KeysymError::Empty));
        for name in ["dead acute", "<u>", "a-b", "u\n", "a.b"] {
            assert!(
                matches!(Keysym::named(name), Err(KeysymError::NotAName { .. })),
                "{name}"
            );
        }
        assert!(matches!(
            Keysym::named(&"u".repeat(AT_MOST + 1)),
            Err(KeysymError::TooLong { .. })
        ));
    }
}
