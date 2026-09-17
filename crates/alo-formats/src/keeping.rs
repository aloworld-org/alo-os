//! The file this crate keeps (ADR 0038), beside every other crate's own.

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::regionally::Regionally;
use crate::timezone::{FollowingTheNetwork, Timezone};

/// What the file is called.
pub const THE_FILE: &str = "formats.toml";

/// The shape of the file.
pub const FORMAT: i64 = 1;

/// **What a person chose about how things are written, and where they are.**
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Chosen {
    /// Which shape this file is in.
    pub format: i64,
    /// The language and region their formats follow, where they said.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub regionally: Option<Regionally>,
    /// Where they said this machine is.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timezone: Option<Timezone>,
    /// Whether the machine may ask the network where it is.
    #[serde(default)]
    pub following_the_network: FollowingTheNetwork,
}

impl Default for Chosen {
    fn default() -> Self {
        Self {
            format: FORMAT,
            regionally: None,
            timezone: None,
            following_the_network: FollowingTheNetwork::No,
        }
    }
}

/// Why the file could not be read.
#[derive(Debug, thiserror::Error)]
pub enum FileNotRead {
    /// Nothing could be read from that path.
    #[error("the formats could not be read: {0}")]
    NotReadable(#[from] io::Error),
    /// It is there and it is not these settings.
    #[error("the formats are there and are not formats this alo OS reads: {0}")]
    NotTheseFormats(#[from] toml::de::Error),
}

/// Why the file could not be written.
#[derive(Debug, thiserror::Error)]
pub enum FileNotWritten {
    /// The folder is not writable.
    #[error("the formats could not be written: {0}")]
    NotWritable(#[from] io::Error),
    /// They could not be written down at all.
    #[error("the formats could not be written down: {0}")]
    NotWritableAsFormats(#[from] toml::ser::Error),
}

/// **Read what a person chose.** No file is a person who has not chosen.
///
/// # Errors
/// [`FileNotRead`] where a file is there and cannot be read or is not this.
pub fn read(at: &Path) -> Result<Chosen, FileNotRead> {
    match std::fs::read_to_string(at.join(THE_FILE)) {
        Ok(text) => Ok(toml::from_str(&text)?),
        Err(why) if why.kind() == io::ErrorKind::NotFound => Ok(Chosen::default()),
        Err(why) => Err(why.into()),
    }
}

/// **Keep what a person chose.**
///
/// # Errors
/// [`FileNotWritten`] where the folder cannot be written to.
pub fn keep(at: &Path, chosen: &Chosen) -> Result<(), FileNotWritten> {
    std::fs::write(at.join(THE_FILE), toml::to_string_pretty(chosen)?)?;
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn a_folder() -> std::path::PathBuf {
        use std::sync::atomic::{AtomicU32, Ordering};
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let path = std::env::temp_dir().join(format!(
            "alo-formats-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    /// **An untouched machine has chosen nothing, and follows no network.**
    #[test]
    fn a_machine_nobody_has_touched_has_chosen_nothing() {
        let folder = a_folder();
        let read_back = read(&folder).unwrap();
        assert_eq!(read_back, Chosen::default());
        assert_eq!(
            read_back.following_the_network,
            FollowingTheNetwork::No,
            "a machine nobody asked would have looked up where it is"
        );
    }

    /// **What is kept is what comes back**, region and zone apart.
    #[test]
    fn what_a_person_chose_is_what_is_read_back() {
        let folder = a_folder();
        let chosen = Chosen {
            regionally: Some(Regionally::reading("pt").unwrap().in_region("BE").unwrap()),
            timezone: Some(Timezone::named("Europe/Brussels").unwrap()),
            following_the_network: FollowingTheNetwork::Yes,
            ..Chosen::default()
        };
        keep(&folder, &chosen).unwrap();
        assert_eq!(read(&folder).unwrap(), chosen);
    }
}
