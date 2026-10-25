//! A folder's name in the person's language, checked before it becomes a
//! directory.
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
//! None of these is hypothetical in the way it sounds. A translator works in a
//! spreadsheet with no machine in front of them, against 24 languages, and
//! `crate::words` asks them for a noun in a file otherwise full of sentences.
//! **A list that is only checked when somebody translates it badly is a list
//! nothing checks**, so every rule below is a test against a name that breaks
//! it.
//!
//! # An undeclared word is refused rather than made
//!
//! `alo_strings::Strings` answers a key it does not hold with the key itself,
//! marked `Said::is_a_bug`. That is the right answer for a sentence — better a
//! key on screen than nothing — and the wrong one for a directory, because it
//! would leave a person with a folder called `their-folders.documents`. So it
//! is refused here, which turns *the shell forgot to declare this crate's
//! words* into a line in a log rather than three folders nobody can explain.

use alo_strings::{Filling, Strings};

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
    /// This folder's name in the language this person reads.
    ///
    /// The name is trimmed before it is checked and before it is used: a
    /// trailing space in a translator's file is invisible in their tool and
    /// produces a directory whose name a person cannot type.
    ///
    /// # Errors
    /// [`NotMade::NotANameAFolderCanHave`], carrying the key, when the
    /// translation is one of the kinds named in this file's header.
    pub fn of(folder: Folder, strings: &Strings) -> Result<Self, NotMade> {
        let key = folder.word().key();
        let refused = || NotMade::NotANameAFolderCanHave {
            key: key.to_string(),
        };

        let said = strings.say(&key, &Filling::nothing());
        if said.is_a_bug() {
            return Err(refused());
        }
        let name = said.text().trim();
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
    use crate::testing::{in_english, translated};
    use crate::words::DOCUMENTS;

    /// The ordinary case, and the one a machine with no translations gets.
    #[test]
    fn a_machine_with_no_translations_makes_the_english_names() {
        let strings = in_english();
        assert_eq!(
            Named::of(Folder::Documents, &strings).unwrap().name(),
            "Documents"
        );
        assert_eq!(
            Named::of(Folder::Downloads, &strings).unwrap().name(),
            "Downloads"
        );
        assert_eq!(
            Named::of(Folder::Pictures, &strings).unwrap().name(),
            "Pictures"
        );
    }

    /// The whole point of the crate: a German machine gets German folders.
    #[test]
    fn a_translated_machine_makes_folders_in_that_language() {
        let strings = translated(&[(DOCUMENTS, "Dokumente")]);
        assert_eq!(
            Named::of(Folder::Documents, &strings).unwrap().name(),
            "Dokumente"
        );
    }

    /// A name that is not Latin at all, because 24 languages are not.
    #[test]
    fn a_name_outside_the_latin_alphabet_is_a_name() {
        let strings = translated(&[(DOCUMENTS, "Έγγραφα")]);
        assert_eq!(
            Named::of(Folder::Documents, &strings).unwrap().name(),
            "Έγγραφα"
        );
    }

    /// Invisible in a translator's tool, and a folder nobody can type.
    #[test]
    fn a_name_with_space_round_it_is_trimmed_rather_than_refused() {
        let strings = translated(&[(DOCUMENTS, "  Dokumente  ")]);
        assert_eq!(
            Named::of(Folder::Documents, &strings).unwrap().name(),
            "Dokumente"
        );
    }

    /// Hidden on every Unix machine: the person would be told they have a
    /// folder and would not find one.
    #[test]
    fn a_name_beginning_with_a_dot_is_refused_because_it_would_be_hidden() {
        let strings = translated(&[(DOCUMENTS, ".Dokumente")]);
        assert!(Named::of(Folder::Documents, &strings).is_err());
    }

    /// `.` and `..` are the sharpest case of the rule above: both would be a
    /// folder that is not a folder.
    #[test]
    fn the_two_names_every_directory_already_has_are_refused() {
        for name in [".", ".."] {
            let strings = translated(&[(DOCUMENTS, name)]);
            assert!(Named::of(Folder::Documents, &strings).is_err(), "{name}");
        }
    }

    /// Not a name but a path, and it could point above the home folder.
    #[test]
    fn a_name_holding_a_slash_is_refused_because_it_is_a_path() {
        for name in ["Doku/mente", "../elsewhere", "/etc"] {
            let strings = translated(&[(DOCUMENTS, name)]);
            assert!(Named::of(Folder::Documents, &strings).is_err(), "{name}");
        }
    }

    /// A folder a person cannot refer to.
    ///
    /// **Only punctuation reaches this file.** An empty translation never
    /// does: `alo-strings` refuses one when the translator's file is checked
    /// against the vocabulary, long before anything here reads it. That makes
    /// the `is_empty` guard in [`Named::of`] the second of two rather than the
    /// only one, and
    /// `an_empty_translation_is_refused_before_it_can_become_a_folder` below
    /// is where the first is held to its job — written as its own test
    /// because this one cannot reach the case at all.
    #[test]
    fn a_name_that_is_only_punctuation_is_refused() {
        for name in ["---", "!!!"] {
            let strings = translated(&[(DOCUMENTS, name)]);
            assert!(Named::of(Folder::Documents, &strings).is_err(), "{name}");
        }
    }

    /// The first of those two guards, and the reason the test above has no
    /// empty case.
    #[test]
    fn an_empty_translation_is_refused_before_it_can_become_a_folder() {
        for name in ["", "   "] {
            assert!(
                crate::testing::would_not_translate(DOCUMENTS, name),
                "an empty translation was accepted: {name:?}"
            );
        }
    }

    /// A control character in a directory name is a name nothing displays.
    #[test]
    fn a_name_with_a_control_character_is_refused() {
        let strings = translated(&[(DOCUMENTS, "Doku\u{7}mente")]);
        assert!(Named::of(Folder::Documents, &strings).is_err());
    }

    /// Not a disk rule: these would break the record other software reads.
    #[test]
    fn a_name_that_would_break_the_record_is_refused() {
        for name in ["Doku\"mente", "$HOME", "Doku\\mente", "Doku`mente`"] {
            let strings = translated(&[(DOCUMENTS, name)]);
            assert!(Named::of(Folder::Documents, &strings).is_err(), "{name}");
        }
    }

    /// The refusal names the key, so whoever fixes it knows which line.
    #[test]
    fn the_refusal_says_which_word_has_to_be_fixed() {
        let strings = translated(&[(DOCUMENTS, ".hidden")]);
        let refused = Named::of(Folder::Documents, &strings).unwrap_err();
        assert_eq!(refused.kept_for_a_maintainer(), DOCUMENTS.key().to_string());
    }
}
