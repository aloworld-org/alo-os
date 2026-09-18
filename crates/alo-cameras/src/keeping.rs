//! What this crate keeps in a person's folder (ADR 0038).
//!
//! Two switches, and nothing else. Which cameras a machine has is the machine's
//! answer and is asked fresh; what a person switched off is theirs and outlives
//! a reboot — **including the reboot they did to try to fix something**, which is
//! the case that matters: a camera that came back on by itself would be a switch
//! nobody could trust.

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::switch::TheSwitches;

/// What the file is called.
pub const THE_FILE: &str = "cameras.toml";

/// The shape of the file.
pub const FORMAT: i64 = 1;

/// **What a person switched off.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Kept {
    /// Which shape this file is in.
    pub format: i64,
    /// The two switches.
    #[serde(default)]
    pub switches: TheSwitches,
}

impl Default for Kept {
    fn default() -> Self {
        Self {
            format: FORMAT,
            switches: TheSwitches::both_on(),
        }
    }
}

/// Why the file could not be read.
#[derive(Debug, thiserror::Error)]
pub enum FileNotRead {
    /// Nothing could be read from that path.
    #[error("the camera switches could not be read: {0}")]
    NotReadable(#[from] io::Error),
    /// It is there and it is not these settings.
    #[error("the camera switches are there and are not settings this alo OS reads: {0}")]
    NotTheseSettings(#[from] toml::de::Error),
}

/// Why the file could not be written.
#[derive(Debug, thiserror::Error)]
pub enum FileNotWritten {
    /// The folder is not writable.
    #[error("the camera switches could not be written: {0}")]
    NotWritable(#[from] io::Error),
    /// They could not be written down at all.
    #[error("the camera switches could not be written down: {0}")]
    NotWritableAsSettings(#[from] toml::ser::Error),
}

/// **Read what a person switched off.** No file is a machine with both on.
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

/// **Keep what a person switched off.**
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
    use crate::switch::{Switch, Which};

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
            loop {
                let at = std::env::temp_dir().join(format!(
                    "alo-cameras-{}-{}",
                    std::process::id(),
                    NEXT.fetch_add(1, Ordering::Relaxed)
                ));
                // Made rather than made-if-needed, and tried again when it is
                // already there. Two fixtures with one counter each and one
                // prefix between them handed the same folder to both on
                // 2026-09-18, and one test's cleanup then removed the other's
                // files; it cost two lanes a publication. `create_dir` refuses a
                // folder that exists, so no two fixtures can hold one folder
                // however their names are spelled.
                if let Err(why) = std::fs::create_dir(&at) {
                    assert!(
                        why.kind() == std::io::ErrorKind::AlreadyExists,
                        "a folder for this test: {why}"
                    );
                    continue;
                }
                return Self(at);
            }
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

    /// **A machine nobody has touched has both on.**
    #[test]
    fn a_machine_nobody_has_touched_has_both_switches_on() {
        let kept = read(AFolder::made().at()).expect("no file is not an error");
        assert_eq!(kept, Kept::default());
        assert_eq!(kept.switches.of(Which::Camera), Switch::On);
    }

    /// **A switch a person turned off is off after a reboot**, which is the
    /// case that matters: a camera that came back on by itself is a switch
    /// nobody can trust.
    #[test]
    fn a_switch_turned_off_is_still_off_tomorrow() {
        let folder = AFolder::made();
        let folder = folder.at();
        let kept = Kept {
            switches: TheSwitches::both_on().switching(Which::Camera),
            ..Kept::default()
        };
        keep(folder, &kept).expect("written");

        let read_back = read(folder).expect("read");
        assert_eq!(read_back, kept);
        assert_eq!(read_back.switches.of(Which::Camera), Switch::Off);
        assert_eq!(read_back.switches.of(Which::Microphone), Switch::On);
    }
}
