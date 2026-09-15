//! What a person chose to open each kind of file with.
//!
//! [`Chosen`] is the person's side of *what opens what*, and the only value
//! [`crate::keeping`] writes into `what-opens-what.toml`. It holds only what the
//! person chose — a kind they never chose for is absent, and is answered by what
//! the applications declare — so an untouched machine writes a format line and
//! nothing else.
//!
//! # Only a person changes it
//!
//! Nothing that answers a request holds a `&mut Chosen`.
//! [`crate::WhatOpensWhat`] borrows it to read, an application's declaration
//! goes into [`crate::Declared`] and never here, no agent verb names it
//! ([`crate::application_verbs`] is the four application verbs and nothing
//! else), and the portal backend answers open-with without a door to change it.
//! What changes it is a Settings surface acting on a person's pick, through
//! [`Chosen::choose`] and [`Chosen::forget`].
//!
//! # An application chosen by its identifier
//!
//! As a grant is made over an identifier and never a name ([`crate::application`]),
//! so is a choice: two applications may both call themselves *Documents*, and
//! the file says which one the person picked.

use std::collections::BTreeMap;

use alo_opening::Kind;
use serde::{Deserialize, Serialize};

use crate::application::Application;
use crate::spelled::{kind_spelled, spelled};

/// Every kind a person chose an application for, and the application.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Written", into = "Written")]
pub struct Chosen {
    /// Each kind chosen for, and the identifier of the application chosen.
    kinds: BTreeMap<Kind, String>,
}

impl Chosen {
    /// Nothing chosen yet, which is what a fresh machine has and what a missing
    /// file reads as.
    #[must_use]
    pub fn untouched() -> Self {
        Self::default()
    }

    /// Whether the person has chosen nothing at all.
    #[must_use]
    pub fn is_untouched(&self) -> bool {
        self.kinds.is_empty()
    }

    /// Open files of this kind with this application, replacing whatever was
    /// chosen for the kind before.
    ///
    /// Any installed application may be chosen for any kind, including one
    /// that does not declare it: a person who opens PDFs in a drawing program
    /// has chosen to, and the choice wins over every declaration.
    pub fn choose(&mut self, kind: Kind, application: &Application) {
        self.kinds.insert(kind, application.identifier().to_owned());
    }

    /// Forget the choice for this kind, so what the applications declare
    /// answers for it again. Says whether there was a choice to forget.
    pub fn forget(&mut self, kind: Kind) -> bool {
        self.kinds.remove(&kind).is_some()
    }

    /// The identifier of the application chosen for this kind, if one was.
    #[must_use]
    pub fn for_kind(&self, kind: Kind) -> Option<&str> {
        self.kinds.get(&kind).map(String::as_str)
    }

    /// Every choice, in the order [`Kind::EVERY`] lists the kinds — for a
    /// Settings surface showing what the person picked.
    pub fn all(&self) -> impl Iterator<Item = (Kind, &str)> {
        self.kinds
            .iter()
            .map(|(kind, application)| (*kind, application.as_str()))
    }
}

/// Choices as the file holds them: a `[kinds]` table from a kind's spelling to
/// an application's identifier, absent entirely when nothing was chosen.
#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Written {
    /// Each kind's spelling and the identifier chosen for it.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    kinds: BTreeMap<String, String>,
}

impl TryFrom<Written> for Chosen {
    type Error = String;

    /// Refuses a spelling that is no kind and an identifier no verb could name,
    /// so a file with either is refused whole rather than read in part.
    fn try_from(written: Written) -> Result<Self, Self::Error> {
        let mut kinds = BTreeMap::new();
        for (name, identifier) in written.kinds {
            let kind = kind_spelled(&name)
                .ok_or_else(|| format!("`{name}` is not a kind of file alo OS reads"))?;
            let application = Application::identified(&identifier)
                .map_err(|_| format!("`{identifier}` is not an application's identifier"))?;
            if application.identifier() != identifier {
                return Err(format!(
                    "`{identifier}` has spaces around it, and an identifier has none"
                ));
            }
            kinds.insert(kind, identifier);
        }
        Ok(Self { kinds })
    }
}

impl From<Chosen> for Written {
    fn from(chosen: Chosen) -> Self {
        Self {
            kinds: chosen
                .kinds
                .into_iter()
                .map(|(kind, application)| (spelled(kind).to_owned(), application))
                .collect(),
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

    fn papers() -> Application {
        Application::called("org.gnome.Papers", "Papers").unwrap()
    }

    fn krita() -> Application {
        Application::identified("org.kde.krita").unwrap()
    }

    /// A choice is made, replaced and forgotten, and *was anything chosen* is
    /// answerable.
    #[test]
    fn a_choice_is_made_replaced_and_forgotten() {
        let mut chosen = Chosen::untouched();
        assert!(chosen.is_untouched());
        chosen.choose(Kind::Pdf, &papers());
        chosen.choose(Kind::Pdf, &krita());
        assert_eq!(chosen.for_kind(Kind::Pdf), Some("org.kde.krita"));
        assert_eq!(chosen.for_kind(Kind::PngImage), None);
        assert!(chosen.forget(Kind::Pdf));
        assert!(!chosen.forget(Kind::Pdf));
        assert!(chosen.is_untouched());
    }

    /// **Nothing chosen writes nothing**, and a choice writes its kind by its
    /// spelling and its application by its identifier.
    #[test]
    fn the_file_holds_only_what_was_chosen() {
        assert_eq!(serde_json::to_string(&Chosen::untouched()).unwrap(), "{}");
        let mut chosen = Chosen::untouched();
        chosen.choose(Kind::WordDocument, &papers());
        assert_eq!(
            serde_json::to_string(&chosen).unwrap(),
            r#"{"kinds":{"word-document":"org.gnome.Papers"}}"#
        );
        let back: Chosen =
            serde_json::from_str(r#"{"kinds":{"word-document":"org.gnome.Papers"}}"#).unwrap();
        assert_eq!(back, chosen);
    }

    /// **A kind that is not one, or an identifier that is not one, refuses the
    /// whole value.**
    #[test]
    fn what_no_choice_could_be_is_refused() {
        for written in [
            r#"{"kinds":{"docx":"org.gnome.Papers"}}"#,
            r#"{"kinds":{"pdf":"/usr/bin/evince"}}"#,
            r#"{"kinds":{"pdf":"org gnome"}}"#,
            r#"{"kinds":{"pdf":" org.gnome.Papers"}}"#,
            r#"{"kinds":{"pdf":""}}"#,
            r#"{"kinds":{},"default":"org.gnome.TextEditor"}"#,
        ] {
            assert!(
                serde_json::from_str::<Chosen>(written).is_err(),
                "{written}"
            );
        }
    }
}
