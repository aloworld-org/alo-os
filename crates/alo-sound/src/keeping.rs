//! What this crate keeps in a person's folder (ADR 0038).

use std::collections::BTreeMap;
use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::device::{Identity, Volume};
use crate::mute::Mute;
use crate::pinned::Pinned;

/// What the file is called.
pub const THE_FILE: &str = "sound.toml";

/// The shape of the file.
pub const FORMAT: i64 = 1;

/// **What a person set, per device, and what they pinned.**
///
/// Kept per device rather than per machine: a headset somebody muted stays
/// muted when they plug it in tomorrow, and the desk speakers keep their own
/// volume.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Kept {
    /// Which shape this file is in.
    pub format: i64,
    /// What they pinned.
    #[serde(default)]
    pub pinned: Pinned,
    /// How loud each device is, where they set it.
    #[serde(default)]
    pub volume: BTreeMap<Identity, Volume>,
    /// Which devices are muted.
    #[serde(default)]
    pub muted: BTreeMap<Identity, Mute>,
}

impl Default for Kept {
    fn default() -> Self {
        Self {
            format: FORMAT,
            pinned: Pinned::nothing(),
            volume: BTreeMap::new(),
            muted: BTreeMap::new(),
        }
    }
}

/// Why the file could not be read.
#[derive(Debug, thiserror::Error)]
pub enum FileNotRead {
    /// Nothing could be read from that path.
    #[error("the sound settings could not be read: {0}")]
    NotReadable(#[from] io::Error),
    /// It is there and it is not these settings.
    #[error("the sound settings are there and are not settings this alo OS reads: {0}")]
    NotTheseSettings(#[from] toml::de::Error),
}

/// Why the file could not be written.
#[derive(Debug, thiserror::Error)]
pub enum FileNotWritten {
    /// The folder is not writable.
    #[error("the sound settings could not be written: {0}")]
    NotWritable(#[from] io::Error),
    /// They could not be written down at all.
    #[error("the sound settings could not be written down: {0}")]
    NotWritableAsSettings(#[from] toml::ser::Error),
}

/// **Read what a person set.** No file is a person who has set nothing.
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

/// **Keep what a person set.**
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
    use crate::device::{Kind, OneDevice};

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
                "alo-sound-{}-{}",
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

    /// **A machine nobody has touched has set nothing**, and is not muted.
    #[test]
    fn a_machine_nobody_has_touched_has_set_nothing() {
        let folder = AFolder::made();
        let folder = folder.at();
        let kept = read(folder).expect("no file is not an error");
        assert_eq!(kept, Kept::default());
        assert!(kept.muted.is_empty());
        assert_eq!(kept.pinned, crate::Pinned::nothing());
    }

    /// **What is set per device comes back per device**, so a headset stays
    /// muted when it is plugged in tomorrow.
    #[test]
    fn what_is_set_for_one_device_is_kept_for_that_device() {
        let folder = AFolder::made();
        let folder = folder.at();
        let headset = OneDevice::reported(
            Identity::reported("usb-headset").expect("named"),
            "Headset",
            Kind::Input,
        );
        let mut kept = Kept {
            pinned: crate::Pinned::nothing().and(&headset),
            ..Kept::default()
        };
        kept.muted
            .insert(headset.identity().clone(), crate::Mute::On);
        kept.volume.insert(
            Identity::reported("desk-speakers").expect("named"),
            Volume::of(60).expect("a volume"),
        );
        keep(folder, &kept).expect("written");

        let read_back = read(folder).expect("read");
        assert_eq!(read_back, kept);
        assert_eq!(
            read_back.muted.get(headset.identity()),
            Some(&crate::Mute::On),
            "a device that was muted came back unmuted"
        );
    }
}
