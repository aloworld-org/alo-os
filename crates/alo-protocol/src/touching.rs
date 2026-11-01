//! **What a change waiting for a person touches**: each file or folder it
//! names, and each application.
//!
//! Added 2026-11-01, additively, by the owner's decision of that day: the
//! approval card (frame `549:135`) shows the file an action is about, by its
//! name and the folder it is in, and until then a change reached the shell as
//! one sentence and nothing else. `docs/contracts/daemon-protocol.md` has the
//! wire shape.
//!
//! # Taken from what the call asks the grants about, and from nothing else
//!
//! `alo_capability::Call::asks` is every path and application a call needs a
//! grant over — the exact list the grants are asked about at the moment of
//! execution. Reading the card's rows off that list means the card can only
//! name what the grants were asked about: a row cannot show a file the call
//! does not reach, and a file the call reaches cannot be missing from the
//! rows. Something the machine has that is not a path or an application — the
//! screen, a printer — is not a row; the sentence already says it.
//!
//! A path is carried as it was given, which is how the sentence names it too.
//! A path that is not text cannot be one: an argument arrives as text.

use alo_capability::Ask;
use serde::{Deserialize, Serialize};

/// One thing a change waiting for a person touches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum Touched {
    /// A file or a folder, by its full path.
    Path(String),
    /// An installed application, by its identifier.
    Application(String),
}

impl Touched {
    /// What one of a call's asks touches, or [`None`] for something that is
    /// neither a path nor an application.
    #[must_use]
    pub fn of(ask: &Ask) -> Option<Self> {
        match ask {
            Ask::Path(path) => path.to_str().map(|it| Self::Path(it.to_owned())),
            Ask::Application(id) => Some(Self::Application(id.clone())),
            Ask::Facility(_) => None,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **On the wire, a path is `{"path": …}` and an application
    /// `{"application": …}`**, which is what the contract writes down.
    #[test]
    fn each_is_written_as_the_contract_says() {
        assert_eq!(
            serde_json::to_string(&Touched::Path("/home/anna/a.pdf".to_owned())).unwrap(),
            r#"{"path":"/home/anna/a.pdf"}"#
        );
        assert_eq!(
            serde_json::to_string(&Touched::Application("org.gnome.TextEditor".to_owned()))
                .unwrap(),
            r#"{"application":"org.gnome.TextEditor"}"#
        );
    }

    /// **A facility is not a row**: the screen is not a file.
    #[test]
    fn a_facility_touches_no_row() {
        assert_eq!(
            Touched::of(&Ask::facility(alo_capability::Facility::ScreenOnce)),
            None
        );
        assert_eq!(
            Touched::of(&Ask::path("/home/anna/a.pdf")),
            Some(Touched::Path("/home/anna/a.pdf".to_owned()))
        );
    }
}
