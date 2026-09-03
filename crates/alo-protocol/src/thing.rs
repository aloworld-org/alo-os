//! One thing in a folder, as a client is told about it.
//!
//! `alo_files::Named` with a wire spelling and nothing else: a name, what it
//! is, and how big it is. It is a shape rather than a line of text for the
//! reason `alo-files` gives — an answer that is prose is an answer a model has
//! to parse, and a model asked to parse an answer will one day parse it wrongly.
//!
//! # Nothing is filtered here
//!
//! A [`alo_files::Named`] cannot hold a name that could rewrite what an answer
//! appears to say: it was refused when the folder was read, and the listing
//! counted it. So this conversion drops nothing and counts nothing — what was
//! left out is already in [`crate::Done::Listed`]'s own count, and a second
//! filter here would be a second answer to a question that was settled at the
//! disk.

use alo_files::{Kind as OnDisk, Named};
use serde::{Deserialize, Serialize};

/// One thing in a folder.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Thing {
    /// What it is called. Shown as it is written, with nothing tidied away.
    name: String,
    /// What it is.
    kind: Kind,
    /// How many bytes it holds — nothing, for anything that is not a file.
    bytes: u64,
}

/// What one thing in a folder is.
///
/// `alo_files::Kind`'s four, and a link is a **link** rather than whatever it
/// points at: a client told otherwise would be told about a place the grants
/// were never asked about.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Kind {
    /// A file.
    File,
    /// A folder.
    Folder,
    /// A link, which may point anywhere, including somewhere nobody granted.
    Link,
    /// A device, a socket, a pipe — something that is none of the three.
    Other,
}

impl Thing {
    /// This thing, as it goes on the wire.
    #[must_use]
    pub fn of(named: &Named) -> Self {
        Self {
            name: named.name().to_owned(),
            kind: Kind::of(named.kind()),
            bytes: named.bytes(),
        }
    }

    /// What it is called.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// What it is.
    #[must_use]
    pub fn kind(&self) -> Kind {
        self.kind
    }

    /// How big it is.
    #[must_use]
    pub fn bytes(&self) -> u64 {
        self.bytes
    }
}

impl Kind {
    /// The same kind, as the disk answered it.
    #[must_use]
    pub fn of(kind: OnDisk) -> Self {
        match kind {
            OnDisk::File => Self::File,
            OnDisk::Folder => Self::Folder,
            OnDisk::Link => Self::Link,
            OnDisk::Other => Self::Other,
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

    /// **A link crosses as a link.** The one thing about a listing that a
    /// client must not be lied to about: everything else in a folder is where
    /// it appears to be, and a link is a name for something that may be
    /// anywhere.
    #[test]
    fn every_kind_has_a_spelling_of_its_own_and_a_link_stays_a_link() {
        for (kind, spelled) in [
            (OnDisk::File, "\"file\""),
            (OnDisk::Folder, "\"folder\""),
            (OnDisk::Link, "\"link\""),
            (OnDisk::Other, "\"other\""),
        ] {
            let written = serde_json::to_string(&Kind::of(kind)).unwrap();
            assert_eq!(written, spelled);
            let back: Kind = serde_json::from_str(&written).unwrap();
            assert_eq!(back, Kind::of(kind));
        }
    }

    /// What is written is what is read back, name and size and all.
    #[test]
    fn a_thing_reads_back_as_what_was_written() {
        let thing = Thing {
            name: "März 2026.pdf".to_owned(),
            kind: Kind::File,
            bytes: 4096,
        };
        let written = serde_json::to_string(&thing).unwrap();
        let back: Thing = serde_json::from_str(&written).unwrap();
        assert_eq!(back, thing);
        assert_eq!(back.name(), "März 2026.pdf");
        assert_eq!(back.kind(), Kind::File);
        assert_eq!(back.bytes(), 4096);
    }

    /// A field nobody declared is refused rather than ignored, on the way back
    /// as on the way in.
    #[test]
    fn a_field_nobody_declared_is_refused() {
        let extra = r#"{"name":"a.pdf","kind":"file","bytes":1,"owner":"root"}"#;
        assert!(serde_json::from_str::<Thing>(extra).is_err());
    }
}
