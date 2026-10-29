//! Which folders a person gets, and what each is called in the record.
//!
//! Three, and the reasoning for each absence is as deliberate as the presence:
//!
//! - **Documents, Downloads and Pictures** are made, because every desktop
//!   operating system makes them and a person arriving from one of those looks
//!   for them by name.
//! - **Desktop is not**, because a Desktop folder exists on other systems to
//!   hold the icons they put on the desktop, and alo OS has a canvas instead.
//!   A folder whose only purpose is a feature this machine does not have is a
//!   folder a person would open once, find empty, and never open again.
//! - **Music, Videos, Templates and Public are not**, because nothing this
//!   machine ships today plays, uses or shares from them. They are the ones to
//!   add when something does, and adding one is a line in [`Folder::ALL`] and
//!   a word.
//!
//! # The name in the record is not the name on the disk
//!
//! The folder on the disk is called whatever `xdg-user-dirs` calls it in the
//! person's language (`alo_gettext`).
//! The key in the record is always the same ASCII, because that key is what
//! other software reads — it is a name for programs, not for people, and
//! translating it would hide the folder from everything that looks for it.

/// **The package whose catalogue names these folders**: `xdg-user-dirs`, the
/// one every Linux desktop names them with (ADR 0137). Its keys are the English
/// names in `/etc/xdg/user-dirs.defaults` — [`Folder::xdg_name`].
pub const THE_CATALOGUE: &str = "xdg-user-dirs";

/// One of the folders a person gets in their home.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Folder {
    /// A person's own documents.
    Documents,
    /// What arrives from the network.
    Downloads,
    /// Photographs and images.
    Pictures,
}

impl Folder {
    /// Every folder a person gets, in the order they are made.
    pub const ALL: [Self; 3] = [Self::Documents, Self::Downloads, Self::Pictures];

    /// The word this folder is named by, which a person's language translates.
    #[must_use]
    pub const fn xdg_name(self) -> &'static str {
        match self {
            Self::Documents => "Documents",
            Self::Downloads => "Downloads",
            Self::Pictures => "Pictures",
        }
    }

    /// The key other software looks this folder up by.
    ///
    /// **`XDG_DOWNLOAD_DIR` is singular and the other two are plural.** That
    /// is not a typo here: it is what the specification says and what every
    /// reader of this file already expects, so a tidier spelling would be a
    /// folder no application finds. `the_download_key_is_singular_because_the
    /// _specification_says_so` is the test that stops somebody correcting it.
    #[must_use]
    pub const fn recorded_as(self) -> &'static str {
        match self {
            Self::Documents => "XDG_DOCUMENTS_DIR",
            Self::Downloads => "XDG_DOWNLOAD_DIR",
            Self::Pictures => "XDG_PICTURES_DIR",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The one spelling in this file that looks like a mistake and is not.
    #[test]
    fn the_download_key_is_singular_because_the_specification_says_so() {
        assert_eq!(Folder::Downloads.recorded_as(), "XDG_DOWNLOAD_DIR");
        assert_eq!(Folder::Documents.recorded_as(), "XDG_DOCUMENTS_DIR");
        assert_eq!(Folder::Pictures.recorded_as(), "XDG_PICTURES_DIR");
    }

    /// A key is for programs, so it is ASCII and shouts.
    #[test]
    fn every_key_is_a_name_for_programs_rather_than_for_people() {
        for folder in Folder::ALL {
            let key = folder.recorded_as();
            assert!(key.starts_with("XDG_"), "{key}");
            assert!(key.ends_with("_DIR"), "{key}");
            assert!(
                key.chars()
                    .all(|it| it.is_ascii_uppercase() || it == '_' || it.is_ascii_digit()),
                "{key}"
            );
        }
    }

    /// Three folders, three distinct keys, three distinct words.
    #[test]
    fn no_two_folders_share_a_key_or_a_word() {
        let mut keys: Vec<&str> = Folder::ALL.iter().map(|it| it.recorded_as()).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), Folder::ALL.len());

        let mut words: Vec<&str> = Folder::ALL.iter().map(|it| it.xdg_name()).collect();
        words.sort_unstable();
        words.dedup();
        assert_eq!(words.len(), Folder::ALL.len());
    }
}
