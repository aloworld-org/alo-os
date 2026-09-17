//! What this crate keeps in a person's folder (ADR 0038).
//!
//! The profile a person chose, and nothing else. Not the battery's readings,
//! which are the machine's and are asked fresh; not what they have been told,
//! which belongs to this run down the battery and should not survive it.
//!
//! # Why the profile is kept here rather than left to the daemon
//!
//! The daemon keeps what the machine is in now; it does not keep what **this
//! person** chose. A machine that came back from a reinstall in *balanced*
//! because that is the daemon's default would have quietly undone a choice, and
//! a person who chose *save power* on a laptop they carry chose it for a reason.

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::profiles::Profile;

/// What the file is called.
pub const THE_FILE: &str = "power.toml";

/// The shape of the file.
pub const FORMAT: i64 = 1;

/// **What a person chose.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Kept {
    /// Which shape this file is in.
    pub format: i64,
    /// The profile they chose, or none if they never have.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chosen: Option<Profile>,
}

impl Default for Kept {
    fn default() -> Self {
        Self {
            format: FORMAT,
            chosen: None,
        }
    }
}

/// Why the file could not be read.
#[derive(Debug, thiserror::Error)]
pub enum FileNotRead {
    /// Nothing could be read from that path.
    #[error("the power settings could not be read: {0}")]
    NotReadable(#[from] io::Error),
    /// It is there and it is not these settings.
    #[error("the power settings are there and are not settings this alo OS reads: {0}")]
    NotTheseSettings(#[from] toml::de::Error),
}

/// Why the file could not be written.
#[derive(Debug, thiserror::Error)]
pub enum FileNotWritten {
    /// The folder is not writable.
    #[error("the power settings could not be written: {0}")]
    NotWritable(#[from] io::Error),
    /// They could not be written down at all.
    #[error("the power settings could not be written down: {0}")]
    NotWritableAsSettings(#[from] toml::ser::Error),
}

/// **Read what a person chose.** No file is a person who has chosen nothing.
///
/// # Errors
/// [`FileNotRead`] where a file is there and cannot be read or is not this.
pub fn read(at: &Path) -> Result<Kept, FileNotRead> {
    match std::fs::read_to_string(at.join(THE_FILE)) {
        Ok(text) => Ok(toml::from_str(&text)?),
        Err(why) if why.kind() == io::ErrorKind::NotFound => Ok(Kept::default()),
        Err(why) => Err(why.into()),
    }
}

/// **Keep what a person chose.**
///
/// # Errors
/// [`FileNotWritten`] where the folder cannot be written to.
pub fn keep(at: &Path, kept: &Kept) -> Result<(), FileNotWritten> {
    std::fs::write(at.join(THE_FILE), toml::to_string_pretty(kept)?)?;
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A folder for one test, which takes itself away when the test is done
    /// with it — pass or fail.
    ///
    /// `docs/quirks.md` records what the alternative costs: on 2026-09-16 a
    /// machine's `/tmp` held 19,632 folders left by tests and `alo-measuring`'s
    /// walk took forty minutes; on 2026-09-17 the same machine held 43,290 and
    /// the suite died of open files. Every one of them was left by a test that
    /// made a folder and did not remove it.
    struct AFolder(std::path::PathBuf);

    impl AFolder {
        fn made() -> Self {
            use std::sync::atomic::{AtomicU32, Ordering};
            static NEXT: AtomicU32 = AtomicU32::new(0);
            let at = std::env::temp_dir().join(format!(
                "alo-power-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(&at).expect("a folder for this test");
            Self(at)
        }

        fn at(&self) -> &std::path::Path {
            &self.0
        }
    }

    impl Drop for AFolder {
        fn drop(&mut self) {
            drop(std::fs::remove_dir_all(&self.0));
        }
    }

    /// **A person who has chosen nothing has chosen nothing**, which is not the
    /// same as having chosen balanced.
    #[test]
    fn a_machine_nobody_has_chosen_on_has_no_choice_in_it() {
        let kept = read(AFolder::made().at()).expect("no file is not an error");
        assert_eq!(kept.chosen, None);
    }

    /// **And what they chose comes back.**
    #[test]
    fn what_a_person_chose_is_what_comes_back() {
        let folder = AFolder::made();
        let folder = folder.at();
        let kept = Kept {
            chosen: Some(Profile::Saver),
            ..Kept::default()
        };
        keep(folder, &kept).expect("written");
        assert_eq!(read(folder).expect("read").chosen, Some(Profile::Saver));
    }
}
