//! The record saying which folder means what, for software that asks.
//!
//! Making the folders is half the job. A person's machine is full of software
//! that wants to know *where do pictures go* — the file chooser a portal
//! opens, a camera application saving a photograph, a browser putting a
//! download somewhere — and none of it can ask the person. Every desktop
//! operating system answers this the same way: it keeps a small record mapping
//! a fixed key to a folder, and software reads the record rather than guessing
//! a name.
//!
//! **Without it, the folders are worse than useless on a translated machine.**
//! Software that guesses, guesses `Pictures`; a Greek machine has a folder
//! whose name is in Greek; so the camera application makes a second, English
//! `Pictures` beside it and the person has two. That is the failure this file
//! exists to prevent, and it is the reason the record is written in the same
//! change as the folders rather than afterwards.
//!
//! # The format is somebody else's and is followed rather than improved
//!
//! It is the `user-dirs.dirs` file every Linux desktop already writes, placed
//! where they all place it. The lines are `KEY="$HOME/name"` — `$HOME` rather
//! than the path spelled out, so that the record stays true if the home folder
//! moves. `crate::naming` is what guarantees a name can sit inside those
//! quotation marks without breaking them.

use crate::naming::Named;

/// Where the record goes, under the person's home folder.
pub const THE_RECORD: &str = ".config/user-dirs.dirs";

/// The record naming each of these folders, as the file's own format.
///
/// Only folders that were actually made belong here: a record naming a folder
/// that is not there sends software to a place that does not exist, which is
/// worse than it guessing.
#[must_use]
pub fn the_record(named: &[Named]) -> String {
    let mut text = String::from(
        "# Which folder means what, for software that asks.\n\
         # Written by alo OS when this account was made.\n\
         # The names are in the language this machine was set up in.\n",
    );
    for one in named {
        text.push_str(one.folder().recorded_as());
        text.push_str("=\"$HOME/");
        text.push_str(one.name());
        text.push_str("\"\n");
    }
    text
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::folders::Folder;
    use crate::testing::{in_english, translated};
    use crate::words::DOCUMENTS;

    fn named(strings: &alo_strings::Strings) -> Vec<Named> {
        Folder::ALL
            .iter()
            .map(|it| Named::of(*it, strings).unwrap())
            .collect()
    }

    /// The shape every other desktop writes, so every reader already parses it.
    #[test]
    fn the_record_is_the_format_other_software_already_reads() {
        let text = the_record(&named(&in_english()));
        assert!(
            text.contains("XDG_DOCUMENTS_DIR=\"$HOME/Documents\"\n"),
            "{text}"
        );
        assert!(
            text.contains("XDG_DOWNLOAD_DIR=\"$HOME/Downloads\"\n"),
            "{text}"
        );
        assert!(
            text.contains("XDG_PICTURES_DIR=\"$HOME/Pictures\"\n"),
            "{text}"
        );
    }

    /// The point of the record: an English key against a translated folder.
    #[test]
    fn a_translated_folder_keeps_the_key_other_software_looks_for() {
        let strings = translated(&[(DOCUMENTS, "Dokumente")]);
        let text = the_record(&named(&strings));
        assert!(
            text.contains("XDG_DOCUMENTS_DIR=\"$HOME/Dokumente\"\n"),
            "{text}"
        );
    }

    /// `$HOME` rather than the path, so the record survives a home that moves.
    #[test]
    fn the_record_names_the_home_folder_rather_than_spelling_it_out() {
        let text = the_record(&named(&in_english()));
        assert!(!text.contains("/var/home"), "{text}");
        assert_eq!(text.matches("$HOME/").count(), Folder::ALL.len());
    }

    /// A folder that was not made is not in the record.
    #[test]
    fn only_the_folders_that_were_made_are_recorded() {
        let strings = in_english();
        let one = vec![Named::of(Folder::Pictures, &strings).unwrap()];
        let text = the_record(&one);
        assert!(text.contains("XDG_PICTURES_DIR"), "{text}");
        assert!(!text.contains("XDG_DOCUMENTS_DIR"), "{text}");
    }

    /// Every line a reader cares about is a key, and the rest are comments.
    #[test]
    fn nothing_but_comments_and_keys_reaches_the_file() {
        let text = the_record(&named(&in_english()));
        for line in text.lines() {
            assert!(
                line.starts_with('#') || line.contains("=\"$HOME/"),
                "{line}"
            );
        }
    }
}
