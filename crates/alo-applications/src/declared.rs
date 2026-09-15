//! What each application on this machine says it opens.
//!
//! An application's own declaration — its desktop entry's media types — is the
//! answer to *what opens this* only where the person has chosen nothing
//! ([`crate::WhatOpensWhat`]). This is the list of those declarations, and
//! nothing about it is a choice.
//!
//! # The first to declare a kind is the one that answers, and that is a security decision
//!
//! It is [`crate::Installed`]'s rule met again. Where two applications declare
//! one kind, taking the later would let whatever was installed most recently
//! make itself what opens every PDF on the machine — an application setting an
//! association on its own behalf, which `docs/features.md`'s *changeable by a
//! person* rules out. So a declaration joins the end of the list and never moves
//! ahead of one already there; what opens a kind changes because a person chose,
//! or because the application that answered is gone.
//!
//! # A declaration is not an installation
//!
//! Declaring is not being here: [`crate::WhatOpensWhat`] asks
//! [`crate::Installed`] about every application this list names, and passes over
//! one that is not installed.

use alo_opening::Kind;

use crate::application::Application;
use crate::media_types::kinds_declared_by;

/// What the applications on this machine say they open, in the order they said
/// it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Declared {
    /// Each kind and the identifier of an application that declared it, in the
    /// order the declarations arrived.
    declarations: Vec<(Kind, String)>,
}

impl Declared {
    /// Nothing declared, which is a machine nothing has looked at yet.
    #[must_use]
    pub fn nothing() -> Self {
        Self::default()
    }

    /// This application says it opens these kinds.
    ///
    /// Answers how many of them were new: a kind this application already
    /// declared is not declared twice, and keeps its place.
    pub fn declares(
        &mut self,
        application: &Application,
        kinds: impl IntoIterator<Item = Kind>,
    ) -> usize {
        let mut added = 0;
        for kind in kinds {
            let already = self
                .declarations
                .iter()
                .any(|(declared, by)| *declared == kind && by == application.identifier());
            if !already {
                self.declarations
                    .push((kind, application.identifier().to_owned()));
                added += 1;
            }
        }
        added
    }

    /// This application says it opens these media types, as its desktop entry
    /// lists them.
    ///
    /// A type this machine does not read declares nothing
    /// ([`crate::media_types`]). Answers how many kinds were new.
    pub fn declares_media_types<'a>(
        &mut self,
        application: &Application,
        media_types: impl IntoIterator<Item = &'a str>,
    ) -> usize {
        let kinds: Vec<Kind> = media_types
            .into_iter()
            .flat_map(|media_type| kinds_declared_by(media_type).iter().copied())
            .collect();
        self.declares(application, kinds)
    }

    /// The identifiers of every application that declared this kind, first
    /// declared first.
    pub fn declaring(&self, kind: Kind) -> impl Iterator<Item = &str> {
        self.declarations
            .iter()
            .filter(move |(declared, _)| *declared == kind)
            .map(|(_, by)| by.as_str())
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

    fn intruder() -> Application {
        Application::identified("com.example.PdfEverything").unwrap()
    }

    /// **A later declaration never moves ahead of an earlier one**, so what was
    /// installed last cannot make itself what opens a kind.
    #[test]
    fn a_later_declaration_joins_the_end() {
        let mut declared = Declared::nothing();
        assert_eq!(declared.declares(&papers(), [Kind::Pdf]), 1);
        assert_eq!(
            declared.declares(&intruder(), [Kind::Pdf, Kind::PngImage]),
            2
        );
        assert_eq!(
            declared.declaring(Kind::Pdf).collect::<Vec<_>>(),
            ["org.gnome.Papers", "com.example.PdfEverything"]
        );

        // Declaring again does not move it forward either.
        assert_eq!(declared.declares(&intruder(), [Kind::Pdf]), 0);
        assert_eq!(
            declared.declaring(Kind::Pdf).next(),
            Some("org.gnome.Papers")
        );
    }

    /// Media types become kinds, and a type this machine does not read
    /// declares nothing.
    #[test]
    fn media_types_are_declared_as_kinds() {
        let mut declared = Declared::nothing();
        let added = declared.declares_media_types(
            &papers(),
            ["application/pdf", "application/x-shellscript", "text/plain"],
        );
        assert_eq!(added, 3);
        assert_eq!(declared.declaring(Kind::Pdf).count(), 1);
        assert_eq!(declared.declaring(Kind::Text).count(), 1);
        assert_eq!(
            declared.declaring(Kind::TextInAnOlderCharacterSet).count(),
            1
        );
        assert_eq!(declared.declaring(Kind::PngImage).count(), 0);
    }
}
