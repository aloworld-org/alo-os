//! The one implementation that reads a real disk.
//!
//! # A link is followed here
//!
//! A link to a folder is a folder and a link to a file is a file, as every file
//! chooser shows them. That is the opposite of `alo-picking`'s rule, and for
//! the opposite reason: a pick there makes a grant over a place, so a link
//! could make it cover somewhere nobody chose; a choice here hands the
//! application one file **the person chose by looking at it**, and the
//! application is given what the person saw. A link that leads nowhere is
//! counted as not shown.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use crate::listing::{Folders, Kind, Listing, NotListed};

/// What is really on this machine's disk. It remembers nothing between
/// questions, so a folder is shown as it is when it is asked about.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OnThisDisk;

impl Folders for OnThisDisk {
    fn listed(&self, folder: &Path) -> Result<Listing, NotListed> {
        if !fs::metadata(folder).map_err(|failed| why(&failed))?.is_dir() {
            return Err(NotListed::NotThere);
        }
        let mut folders = Vec::new();
        let mut files = Vec::new();
        let mut could_not_be_named = 0_usize;
        for entry in fs::read_dir(folder).map_err(|failed| why(&failed))? {
            let Ok(entry) = entry else {
                could_not_be_named = could_not_be_named.saturating_add(1);
                continue;
            };
            // **Followed**, for this file's reason.
            let Ok(what) = fs::metadata(entry.path()) else {
                could_not_be_named = could_not_be_named.saturating_add(1);
                continue;
            };
            let Ok(name) = entry.file_name().into_string() else {
                could_not_be_named = could_not_be_named.saturating_add(1);
                continue;
            };
            if what.is_dir() {
                folders.push(name);
            } else {
                files.push(name);
            }
        }
        Ok(Listing::of(folders, files).and_could_not_be_named(could_not_be_named))
    }

    fn holds(&self, folder: &Path, name: &str) -> Option<Kind> {
        let there = folder.join(name);
        // A link that leads nowhere is still a name taken.
        let what = fs::metadata(&there)
            .or_else(|_| fs::symlink_metadata(&there))
            .ok()?;
        Some(if what.is_dir() {
            Kind::Folder
        } else {
            Kind::File
        })
    }
}

/// What the machine said, as one of the two things a person is told.
fn why(failed: &std::io::Error) -> NotListed {
    match failed.kind() {
        ErrorKind::NotFound => NotListed::NotThere,
        _ => NotListed::WouldNotBeRead,
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **A real folder lists its folders and files**, and a file or a folder
    /// that is not there cannot be stood in.
    #[test]
    fn a_real_folder_is_listed() {
        let at = std::env::temp_dir().join(format!("alo-file-chooser-{}", std::process::id()));
        let _gone = fs::remove_dir_all(&at);
        fs::create_dir_all(at.join("Letters")).unwrap();
        fs::write(at.join("march.txt"), b"m").unwrap();
        fs::write(at.join(".hidden"), b"h").unwrap();
        let listing = OnThisDisk.listed(&at).unwrap();
        assert_eq!(listing.folders(), ["Letters".to_owned()]);
        assert_eq!(listing.files(), ["march.txt".to_owned()]);
        assert_eq!(listing.not_shown(), 1);
        assert_eq!(OnThisDisk.holds(&at, ".hidden"), Some(Kind::File));
        assert_eq!(OnThisDisk.holds(&at, "Letters"), Some(Kind::Folder));
        assert_eq!(OnThisDisk.holds(&at, "nowhere"), None);
        assert_eq!(
            OnThisDisk.listed(&at.join("march.txt")),
            Err(NotListed::NotThere)
        );
        assert_eq!(
            OnThisDisk.listed(&at.join("nowhere")),
            Err(NotListed::NotThere)
        );
        fs::remove_dir_all(&at).unwrap();
    }
}
