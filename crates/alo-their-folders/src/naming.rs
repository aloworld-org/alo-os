//! A folder's name in the person's language — `xdg-user-dirs`' own, from
//! [`crate::catalogue`] — checked before it becomes a directory.
//!
//! This is the one file in this crate that can be wrong in a way a person
//! would have to live with. Everywhere else a translator's mistake produces a
//! sentence that reads oddly; here it produces **a folder in their home**, and
//! three mistakes in particular produce one they cannot see, cannot open, or
//! that is not where it says it is:
//!
//! - a name beginning with a dot is hidden on every Unix machine, so a person
//!   told they have a Documents folder would not find one;
//! - a name holding a slash is not a name at all — it is a path, and it would
//!   put the folder somewhere nobody asked for, including somewhere above the
//!   home folder;
//! - a name that is only punctuation, or empty, gives them a folder they
//!   cannot refer to.
//!
//! A fourth rule is not about the disk at all. `crate::recording` writes each
//! name into a record in quotation marks, and that record is read — and on
//! some systems sourced as a shell script — by other software. So a name
//! holding a quotation mark, a dollar, a backslash or a backtick is refused
//! here rather than escaped later: escaping it correctly in a file format
//! somebody else's shell parses is a thing to get wrong once and never
//! notice, and no language's word for *documents* contains any of them.
//!
//! None of these is hypothetical in the way it sounds. The catalogue is
//! somebody else's file on the machine, translated by people who never saw
//! this rule. **A list that is only checked when somebody translates it badly
//! is a list nothing checks**, so every rule below is a test against a name
//! that breaks it.
//!
//! # A language with no catalogue
//!
//! Is the English key, exactly as `xdg-user-dirs` itself leaves it on every
//! Linux desktop — and an empty translation is the same, as gettext gives it.

use crate::catalogue::Catalogue;
use crate::folders::Folder;
use crate::refusing::NotMade;

/// One folder's name, as this machine will write it on the disk.
///
/// Sealed: the only way to make one is [`Named::of`], so nothing downstream
/// can hand `crate::making` a name that was never checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Named {
    /// Which folder this is.
    folder: Folder,
    /// Its name, trimmed, in the person's language.
    name: String,
}

impl Named {
    /// This folder's name in the language whose catalogue this is — or, for
    /// [`None`], its English key.
    ///
    /// The name is trimmed before it is checked and before it is used: a
    /// trailing space in a translator's file is invisible in their tool and
    /// produces a directory whose name a person cannot type.
    ///
    /// # Errors
    /// [`NotMade::NotANameAFolderCanHave`], carrying the key, when the
    /// translation is one of the kinds named in this file's header.
    pub fn of(folder: Folder, catalogue: Option<&Catalogue>) -> Result<Self, NotMade> {
        let key = folder.xdg_name();
        let refused = || NotMade::NotANameAFolderCanHave {
            key: key.to_owned(),
        };
        let name = catalogue
            .and_then(|it| it.name_of(key))
            .unwrap_or(key)
            .trim();
        if name.is_empty()
            || name.contains('/')
            || name.starts_with('.')
            || name.chars().any(char::is_control)
            || name.contains(['"', '$', '\\', '`'])
            || !name.chars().any(char::is_alphanumeric)
        {
            return Err(refused());
        }
        Ok(Self {
            folder,
            name: name.to_owned(),
        })
    }

    /// Which folder this is.
    #[must_use]
    pub const fn folder(&self) -> Folder {
        self.folder
    }

    /// Its name on the disk, in the person's language.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_german;

    fn documents_in_german(name: &str) -> Result<Named, NotMade> {
        Named::of(
            Folder::Documents,
            Some(&in_german(&[(Folder::Documents, name)])),
        )
    }

    /// **No catalogue is the English key**, as `xdg-user-dirs` leaves it.
    #[test]
    fn a_language_with_no_catalogue_makes_the_english_names() {
        for folder in Folder::ALL {
            assert_eq!(Named::of(folder, None).unwrap().name(), folder.xdg_name());
        }
    }

    /// The whole point of the crate: a German person gets German folders.
    #[test]
    fn a_catalogue_makes_folders_in_its_language() {
        assert_eq!(
            documents_in_german("Dokumente").unwrap().name(),
            "Dokumente"
        );
    }

    /// **A catalogue without this folder, or with it empty, is the English
    /// key**, as gettext gives it.
    #[test]
    fn a_name_the_catalogue_lacks_is_the_english_key() {
        let without = in_german(&[(Folder::Pictures, "Bilder")]);
        assert_eq!(
            Named::of(Folder::Documents, Some(&without)).unwrap().name(),
            "Documents"
        );
        assert_eq!(documents_in_german("").unwrap().name(), "Documents");
    }

    /// A name that is not Latin at all, because 24 languages are not.
    #[test]
    fn a_name_outside_the_latin_alphabet_is_a_name() {
        assert_eq!(documents_in_german("Έγγραφα").unwrap().name(), "Έγγραφα");
    }

    /// Invisible in a translator's tool, and a folder nobody can type.
    #[test]
    fn a_name_with_space_round_it_is_trimmed_rather_than_refused() {
        assert_eq!(
            documents_in_german("  Dokumente  ").unwrap().name(),
            "Dokumente"
        );
    }

    /// Hidden on every Unix machine: the person would be told they have a
    /// folder and would not find one — and `.` and `..` are the sharpest case.
    #[test]
    fn a_name_beginning_with_a_dot_is_refused_because_it_would_be_hidden() {
        for name in [".Dokumente", ".", ".."] {
            assert!(documents_in_german(name).is_err(), "{name}");
        }
    }

    /// Not a name but a path, and it could point above the home folder.
    #[test]
    fn a_name_holding_a_slash_is_refused_because_it_is_a_path() {
        for name in ["Doku/mente", "../elsewhere", "/etc"] {
            assert!(documents_in_german(name).is_err(), "{name}");
        }
    }

    /// A folder a person cannot refer to: only punctuation, or only space.
    #[test]
    fn a_name_that_is_only_punctuation_or_space_is_refused() {
        for name in ["---", "!!!", "   "] {
            assert!(documents_in_german(name).is_err(), "{name:?}");
        }
    }

    /// A control character in a directory name is a name nothing displays.
    #[test]
    fn a_name_with_a_control_character_is_refused() {
        assert!(documents_in_german("Doku\u{7}mente").is_err());
    }

    /// Not a disk rule: these would break the record other software reads.
    #[test]
    fn a_name_that_would_break_the_record_is_refused() {
        for name in ["Doku\"mente", "$HOME", "Doku\\mente", "Doku`mente`"] {
            assert!(documents_in_german(name).is_err(), "{name}");
        }
    }

    /// The refusal names the English key, so whoever fixes it knows which
    /// line of which catalogue.
    #[test]
    fn the_refusal_says_which_name_has_to_be_fixed() {
        let refused = documents_in_german(".hidden").unwrap_err();
        assert_eq!(refused.kept_for_a_maintainer(), "Documents");
    }
}
