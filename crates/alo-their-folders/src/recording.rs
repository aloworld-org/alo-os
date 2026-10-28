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
//!
//! # A record that is already there is the person's
//!
//! It is read before anything is made, and **what it names is where each
//! folder is** — whatever language that name is in. A home made in English and
//! signed into after a change to German keeps `Documents`: renaming a person's
//! folder, or making a second `Dokumente` beside it, is not this program's to
//! do (the owner's decision of 2026-10-28; ADR 0136). Lines naming folders
//! this crate does not make — `XDG_MUSIC_DIR`, somebody's own — are kept as
//! they were, and a key set to `$HOME` alone, which is how the format says
//! *this person has no such folder*, is honoured rather than refilled.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::folders::Folder;

/// Where the record goes, under the person's home folder.
pub const THE_RECORD: &str = ".config/user-dirs.dirs";

/// **What a record already says**: each key's value, and every other line.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AlreadyRecorded {
    /// Each `KEY="value"`, by key, with the value as written.
    values: BTreeMap<String, String>,
    /// Every line that is not a comment and not one of this crate's keys,
    /// exactly as it was.
    others: Vec<String>,
}

/// Where a recorded folder is, from what the record says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Points {
    /// At this folder.
    At(PathBuf),
    /// Nowhere: the person has no such folder, which the format writes as the
    /// home folder itself.
    Nowhere,
}

impl AlreadyRecorded {
    /// The record whose text this is.
    #[must_use]
    pub fn read(text: &str) -> Self {
        let mut already = Self::default();
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let Some((key, value)) = trimmed.split_once('=') else {
                already.others.push(line.to_owned());
                continue;
            };
            let key = key.trim();
            if Folder::ALL.iter().any(|it| it.recorded_as() == key) {
                let value = value.trim().trim_matches('"');
                already.values.insert(key.to_owned(), value.to_owned());
            } else {
                already.others.push(line.to_owned());
            }
        }
        already
    }

    /// **Where the record says this folder is**, under `home` — or [`None`]
    /// where it says nothing about it, or something that is not a place.
    #[must_use]
    pub fn where_it_points(&self, folder: Folder, home: &Path) -> Option<Points> {
        let value = self.values.get(folder.recorded_as())?;
        if value == "$HOME" || value == "$HOME/" {
            return Some(Points::Nowhere);
        }
        if let Some(under) = value.strip_prefix("$HOME/") {
            return Some(Points::At(home.join(under)));
        }
        value
            .starts_with('/')
            .then(|| Points::At(PathBuf::from(value)))
    }

    /// The value the record gives this folder, as written.
    #[must_use]
    pub fn value_of(&self, folder: Folder) -> Option<&str> {
        self.values.get(folder.recorded_as()).map(String::as_str)
    }
}

/// **The record**: this crate's folders, each with the value it is recorded
/// as (`$HOME/Dokumente`), and every other line `already` had, unchanged.
///
/// Only folders that are on the disk belong in `ours`: a record naming a
/// folder that is not there sends software to a place that does not exist,
/// which is worse than it guessing.
#[must_use]
pub fn the_record(ours: &[(Folder, String)], already: &AlreadyRecorded) -> String {
    let mut text = String::from(
        "# Which folder means what, for software that asks.\n\
         # Written by alo OS. A folder already named here keeps its name.\n",
    );
    for line in &already.others {
        text.push_str(line);
        text.push('\n');
    }
    for (folder, value) in ours {
        text.push_str(folder.recorded_as());
        text.push_str("=\"");
        text.push_str(value);
        text.push_str("\"\n");
    }
    text
}

/// The value a folder made under the home folder is recorded as.
#[must_use]
pub fn under_home(name: &str) -> String {
    format!("$HOME/{name}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn the_three(names: [&str; 3]) -> Vec<(Folder, String)> {
        Folder::ALL
            .iter()
            .zip(names)
            .map(|(folder, name)| (*folder, under_home(name)))
            .collect()
    }

    /// The shape every other desktop writes, so every reader already parses it.
    #[test]
    fn the_record_is_the_format_other_software_already_reads() {
        let text = the_record(
            &the_three(["Dokumente", "Downloads", "Bilder"]),
            &AlreadyRecorded::default(),
        );
        assert!(
            text.contains("XDG_DOCUMENTS_DIR=\"$HOME/Dokumente\"\n"),
            "{text}"
        );
        assert!(
            text.contains("XDG_DOWNLOAD_DIR=\"$HOME/Downloads\"\n"),
            "{text}"
        );
        assert!(
            text.contains("XDG_PICTURES_DIR=\"$HOME/Bilder\"\n"),
            "{text}"
        );
        assert!(!text.contains("/var/home"), "{text}");
    }

    /// **A record read back says where each folder is**, and the lines this
    /// crate does not own are kept as they were.
    #[test]
    fn a_record_read_back_keeps_what_it_does_not_own() {
        let home = Path::new("/var/home/ada");
        let already = AlreadyRecorded::read(
            "# a comment\n\
             XDG_DOCUMENTS_DIR=\"$HOME/Documents\"\n\
             XDG_MUSIC_DIR=\"$HOME/Musik\"\n\
             XDG_PICTURES_DIR=\"$HOME\"\n\
             XDG_DOWNLOAD_DIR=\"/srv/downloads\"\n",
        );
        assert_eq!(
            already.where_it_points(Folder::Documents, home),
            Some(Points::At(home.join("Documents")))
        );
        assert_eq!(
            already.where_it_points(Folder::Pictures, home),
            Some(Points::Nowhere)
        );
        assert_eq!(
            already.where_it_points(Folder::Downloads, home),
            Some(Points::At(PathBuf::from("/srv/downloads")))
        );
        let text = the_record(&[], &already);
        assert!(text.contains("XDG_MUSIC_DIR=\"$HOME/Musik\"\n"), "{text}");
    }

    /// **Nothing said is nothing pointed at**, and a value that is not a
    /// place is not guessed at.
    #[test]
    fn a_key_not_there_or_not_a_place_points_nowhere_known() {
        let home = Path::new("/var/home/ada");
        let already = AlreadyRecorded::read("XDG_DOCUMENTS_DIR=\"Documents\"\n");
        assert_eq!(already.where_it_points(Folder::Documents, home), None);
        assert_eq!(already.where_it_points(Folder::Pictures, home), None);
    }
}
