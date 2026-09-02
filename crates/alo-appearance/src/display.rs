//! Which screen, when there is more than one.
//!
//! **A display is matched exactly**, which is the rule item 1 set for grants and
//! is here for the same reason: matching loosely matches more than the person
//! picked, and the picture on the laptop appearing on the projector in a meeting
//! is the cheap version of that mistake.
//!
//! The name is whatever the compositor calls the display — a connector and the
//! monitor's own description, on the machines this will run on. This crate never
//! interprets it, so a display that is renamed by a driver update is simply a
//! display nobody has chosen for yet, and gets the background the person set for
//! everywhere.

use serde::{Deserialize, Serialize};

/// Why a piece of text does not name a display.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DisplayError {
    /// Nothing at all.
    #[error("name the display — it is the name the shell shows for the screen")]
    Unnamed,
    /// A name with a space at one end, which reads as the same name and is not.
    #[error(
        "{0:?} begins or ends with a space, and a display is matched exactly — give the name without it"
    )]
    Spaced(String),
}

/// What one display is called.
///
/// Reads back through [`DisplayId::named`], so a hand-edited file cannot hold a
/// name that would never match anything.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct DisplayId {
    /// The name, exactly as the compositor gives it.
    name: String,
}

impl DisplayId {
    /// A display by name.
    ///
    /// # Errors
    /// [`DisplayError`] for an empty name, or one padded with spaces that would
    /// stop it ever matching the display it looks like.
    pub fn named(name: &str) -> Result<Self, DisplayError> {
        if name.is_empty() {
            return Err(DisplayError::Unnamed);
        }
        if name.trim() != name {
            return Err(DisplayError::Spaced(name.to_owned()));
        }
        Ok(Self {
            name: name.to_owned(),
        })
    }

    /// The name, exactly as the compositor gives it.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl TryFrom<String> for DisplayId {
    type Error = DisplayError;

    fn try_from(name: String) -> Result<Self, Self::Error> {
        Self::named(&name)
    }
}

impl From<DisplayId> for String {
    fn from(display: DisplayId) -> Self {
        display.name
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The name is kept as it was given, and survives a settings file.
    #[test]
    fn a_display_is_named_and_written_down_as_its_name() {
        let display = DisplayId::named("DP-1 Dell U2720Q").unwrap();
        assert_eq!(display.name(), "DP-1 Dell U2720Q");
        assert_eq!(
            serde_json::to_string(&display).unwrap(),
            r#""DP-1 Dell U2720Q""#
        );
        assert_eq!(
            serde_json::from_str::<DisplayId>(r#""DP-1 Dell U2720Q""#).unwrap(),
            display
        );
    }

    /// **Exactly**, which is the whole rule: a name that differs by a letter or
    /// a case is a different display, not a near miss.
    #[test]
    fn a_display_is_matched_exactly() {
        let display = DisplayId::named("DP-1").unwrap();
        assert_ne!(display, DisplayId::named("dp-1").unwrap());
        assert_ne!(display, DisplayId::named("DP-2").unwrap());
        assert_eq!(display, DisplayId::named("DP-1").unwrap());
    }

    /// A name that could never match anything is refused where it is given,
    /// rather than becoming a row in the settings file that does nothing.
    #[test]
    fn a_name_that_would_never_match_is_refused() {
        assert_eq!(DisplayId::named(""), Err(DisplayError::Unnamed));
        assert_eq!(
            DisplayId::named(" DP-1"),
            Err(DisplayError::Spaced(" DP-1".to_owned()))
        );
        assert_eq!(
            DisplayId::named("DP-1 "),
            Err(DisplayError::Spaced("DP-1 ".to_owned()))
        );
        assert!(serde_json::from_str::<DisplayId>(r#""""#).is_err());
    }
}
