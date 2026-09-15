//! Whether the file at a path may be written over.
//!
//! [ADR 0038](../../../docs/decisions/0038-a-persons-settings-are-kept-by-the-crate-that-owns-each.md),
//! clause 3: *Settings does not write over a file that did not read* except when
//! the person puts that section back as shipped, *so a hand-edit with a typo is
//! never lost silently to the next click.* A surface that drew the release's
//! settings because the file was refused at sign-in, and then saved the person's
//! next change, would otherwise replace the very edit they were about to fix.
//!
//! # When it is asked
//!
//! **At the moment of the write, of the file as it is then**, after the new text
//! has been staged and read back and immediately before the rename that would
//! replace the old file. Never of what was read at sign-in: a person who fixed
//! the typo in an editor since has a file that reads, and their change is
//! written.
//!
//! # What may be written over
//!
//! A file that is not there, and a file that reads as its shape. Anything else —
//! text that is not this shape, another format, a key nobody declared, bytes
//! that are not text, a file the disk will not hand over, a folder where the
//! file should be — is a file that did not read, and it stays.
//!
//! # The one door that does
//!
//! [`crate::put_back_as_shipped`] replaces whatever is there with the person
//! who has changed nothing. It takes no value, so it cannot be used to write a
//! change past this check: the only thing it can put in place of a broken file
//! is the release's settings, which is the deliberate act the ADR names.

use std::path::Path;

use crate::kept::Kept;
use crate::reading::as_it_is;
use crate::unwritten::Unwritten;

/// What a write is allowed to replace.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Over {
    /// Only a file that is not there or that reads: every write of a value.
    WhatReads,
    /// Whatever is there: putting the section back as shipped, and only that.
    Anything,
}

impl Over {
    /// Whether the file at `at`, as it is now, may be written over.
    ///
    /// # Errors
    ///
    /// [`Unwritten::OverAFileThatDidNotRead`], carrying why it did not read,
    /// when this write may only replace what reads and the file does not.
    pub(crate) fn allows<K: Kept>(self, at: &Path) -> Result<(), Unwritten> {
        match self {
            Self::Anything => Ok(()),
            Self::WhatReads => as_it_is::<K>(at)
                .map(|_| ())
                .map_err(Unwritten::OverAFileThatDidNotRead),
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
    use crate::testing::{Example, a_folder_of_our_own};
    use crate::unread::Unread;

    /// **A file that is not there may be written**, which is a first change.
    #[test]
    fn a_file_that_is_not_there_may_be_written() {
        let at = a_folder_of_our_own("replacing-missing").join("example.toml");
        assert_eq!(Over::WhatReads.allows::<Example>(&at), Ok(()));
    }

    /// **A file that reads may be written over**, including one that is only a
    /// format line.
    #[test]
    fn a_file_that_reads_may_be_written_over() {
        let at = a_folder_of_our_own("replacing-reads").join("example.toml");
        for text in ["format = 1\n", "format = 1\nedge = \"left\"\n"] {
            std::fs::write(&at, text).unwrap();
            assert_eq!(Over::WhatReads.allows::<Example>(&at), Ok(()), "{text:?}");
        }
    }

    /// **A file that did not read may not be written over by a value**, and the
    /// reason carried is the reason it did not read.
    #[test]
    fn a_file_that_did_not_read_may_not_be_written_over_by_a_value() {
        let folder = a_folder_of_our_own("replacing-unread");
        let at = folder.join("example.toml");
        std::fs::write(&at, "format = 1\nwallpaper = \"navy\"\n").unwrap();
        assert_eq!(
            Over::WhatReads.allows::<Example>(&at),
            Err(Unwritten::OverAFileThatDidNotRead(Unread::UnknownKey {
                key: "wallpaper".to_owned()
            }))
        );

        let a_folder = folder.join("folder.toml");
        std::fs::create_dir(&a_folder).unwrap();
        assert!(matches!(
            Over::WhatReads.allows::<Example>(&a_folder),
            Err(Unwritten::OverAFileThatDidNotRead(Unread::Disk(_)))
        ));
    }

    /// **Putting back as shipped may write over anything.**
    #[test]
    fn putting_back_may_write_over_anything() {
        let at = a_folder_of_our_own("replacing-anything").join("example.toml");
        std::fs::write(&at, "not toml at all [").unwrap();
        assert_eq!(Over::Anything.allows::<Example>(&at), Ok(()));
    }
}
