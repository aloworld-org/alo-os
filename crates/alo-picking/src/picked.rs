//! What a person's pick is, and what closing the picker without one is.
//!
//! [`Picked`] is the value the rest of the machine acts on, and the one thing
//! in this crate that is **sealed**: it has no public constructor, so the only
//! way anything anywhere can obtain one is [`crate::Picker::pick`] — a folder
//! somebody navigated to and chose. `alo-capability` says a grant is made from
//! a folder somebody picked and that nothing may widen one; this type is where
//! that sentence stops depending on everybody remembering it. The seam is the
//! same one `alo_files::Real` and `alo_overlay::SurfaceRequest` use, for the
//! same reason.
//!
//! [`Chosen`] is the picker's ending, and it has two: a folder, or nothing.
//! *Nothing* is a value rather than an absence because the plan's second
//! acceptance is about it — **picking nothing grants nothing** — and a
//! [`Option::None`] would have left that as a convention in whatever wired the
//! picker up.

use std::path::{Path, PathBuf};

use alo_capability::Reach;
use alo_strings::{Filling, Said, Strings};

use crate::words;

/// A folder a person picked.
///
/// Sealed: see this module's documentation. Holding one is knowing that
/// somebody stood in this folder in a picker and chose it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picked(PathBuf);

impl Picked {
    /// The one place a pick comes from.
    pub(crate) fn folder_chosen(at: PathBuf) -> Self {
        Self(at)
    }

    /// The folder itself.
    #[must_use]
    pub fn folder(&self) -> &Path {
        &self.0
    }

    /// What a grant made from this pick covers: **this folder**, and never the
    /// folder it was picked from.
    ///
    /// The plan's third acceptance lives on this line. A picker navigates
    /// downwards, and the tempting mistake — granting where the picker was
    /// opened, or the parent a person came through — is not available here,
    /// because the only path this type holds is the one that was picked.
    #[must_use]
    pub fn reach(&self) -> Reach {
        Reach::Folder(self.0.clone())
    }

    /// What this pick will let the agent do, in the language the person reads.
    ///
    /// Shown beside the folder before anybody grants it, which is why it says
    /// what the grant does **not** cover as well: a sentence about reach that
    /// only says what is included is a sentence that reads as more than it is.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(
            &words::COVERS.key(),
            &Filling::of(words::THE_FOLDER, self.0.display().to_string()),
        )
    }
}

/// How the picker ended.
///
/// There is no third ending. A picker that is still open has not ended, and
/// [`crate::Picker`] answers a refusal for everything else — so *closed
/// without picking* cannot be confused with *something went wrong*, which is a
/// distinction the person made and a machine must not lose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Chosen {
    /// The person picked this folder.
    Folder(Picked),
    /// The person closed the picker without picking. Nothing is granted, and
    /// [`crate::Granting`] has no path that could grant anything from it.
    Nothing,
}

impl Chosen {
    /// The folder, where one was picked.
    #[must_use]
    pub fn folder(&self) -> Option<&Path> {
        match self {
            Self::Folder(picked) => Some(picked.folder()),
            Self::Nothing => None,
        }
    }

    /// Whether the picker was closed without picking.
    #[must_use]
    pub fn is_nothing(&self) -> bool {
        matches!(self, Self::Nothing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// The folder these tests are about.
    fn invoices() -> Picked {
        Picked::folder_chosen(PathBuf::from("/home/anna/Invoices"))
    }

    /// **What a pick covers is the folder picked**, which is the one thing
    /// `alo-capability` will be handed.
    #[test]
    fn a_pick_covers_the_folder_that_was_picked() {
        let picked = invoices();
        assert_eq!(picked.folder(), Path::new("/home/anna/Invoices"));
        assert_eq!(
            picked.reach(),
            Reach::Folder(PathBuf::from("/home/anna/Invoices"))
        );
    }

    /// **Nothing picked is a folder nobody can read out of it.** The type is
    /// the whole of *picking nothing grants nothing*: there is no folder in
    /// this value for anything to grant.
    #[test]
    fn nothing_picked_holds_no_folder() {
        let nothing = Chosen::Nothing;
        assert!(nothing.is_nothing());
        assert_eq!(nothing.folder(), None);
    }

    /// A folder picked is not nothing, and reads back as the folder it was.
    #[test]
    fn a_folder_picked_is_not_nothing() {
        let chosen = Chosen::Folder(invoices());
        assert!(!chosen.is_nothing());
        assert_eq!(chosen.folder(), Some(Path::new("/home/anna/Invoices")));
    }

    /// **The sentence a person reads names their own folder and says what is
    /// not covered.** The path is not translated — a translated path would
    /// name a different folder — and the rest of the sentence is.
    #[test]
    fn the_sentence_names_the_folder_and_what_is_not_covered() {
        let said = invoices().said(&in_english());
        assert!(!said.is_a_bug());
        assert!(said.text().contains("/home/anna/Invoices"), "{said}");
        assert!(said.text().contains("nothing outside it"), "{said}");
    }

    /// And it arrives in the reader's own language, with their own path still
    /// in it.
    #[test]
    fn the_sentence_is_read_in_the_readers_own_language() {
        let strings = translated(&[(
            words::COVERS,
            "Der Agent kann dann {folder} und alles darin erreichen, und nichts außerhalb davon",
        )]);
        let said = invoices().said(&strings);
        assert!(said.is_translated());
        assert!(said.text().starts_with("Der Agent"), "{said}");
        assert!(said.text().contains("/home/anna/Invoices"), "{said}");
    }
}
