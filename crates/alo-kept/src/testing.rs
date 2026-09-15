//! Shapes this crate's own tests keep, including the ones the rule refuses.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a test fixture, a panic on a None or an Err is the failure being reported"
)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::kept::Kept;
use crate::unread::Unread;
use crate::unwritten::Unwritten;

/// Which edge, as a shape with a closed list of values has one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum Edge {
    /// The bottom edge.
    Bottom,
    /// The left edge.
    Left,
}

/// A shape the way a settings crate's `Changes` is one: only what changed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Example {
    /// A value a person may have changed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) background: Option<String>,
    /// A value from a closed list.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) edge: Option<Edge>,
    /// A table of exceptions.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) displays: BTreeMap<String, String>,
    /// A key the shape has and [`Kept::KEYS`] forgot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) forgotten: Option<bool>,
}

impl Kept for Example {
    const FILE: &'static str = "example.toml";
    const FORMAT: i64 = 1;
    const KEYS: &'static [&'static str] = &["background", "edge", "displays"];
    type NotRead = (PathBuf, Unread);
    type NotWritten = (PathBuf, Unwritten);

    fn untouched() -> Self {
        Self::default()
    }

    fn not_read(at: &Path, why: Unread) -> Self::NotRead {
        (at.to_owned(), why)
    }

    fn not_written(at: &Path, why: Unwritten) -> Self::NotWritten {
        (at.to_owned(), why)
    }
}

/// A shape that writes a value for a person who changed nothing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct WritesWhatNobodyChanged {
    /// Written whether or not anybody chose it.
    pub(crate) edge: Edge,
}

impl Kept for WritesWhatNobodyChanged {
    const FILE: &'static str = "nobody.toml";
    const FORMAT: i64 = 1;
    const KEYS: &'static [&'static str] = &["edge"];
    type NotRead = Unread;
    type NotWritten = Unwritten;

    fn untouched() -> Self {
        Self { edge: Edge::Bottom }
    }

    fn not_read(_: &Path, why: Unread) -> Self::NotRead {
        why
    }

    fn not_written(_: &Path, why: Unwritten) -> Self::NotWritten {
        why
    }
}

/// A shape that is a list rather than a table of keys.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct NotATable(pub(crate) Vec<String>);

impl Kept for NotATable {
    const FILE: &'static str = "list.toml";
    const FORMAT: i64 = 1;
    const KEYS: &'static [&'static str] = &[];
    type NotRead = Unread;
    type NotWritten = Unwritten;

    fn untouched() -> Self {
        Self(Vec::new())
    }

    fn not_read(_: &Path, why: Unread) -> Self::NotRead {
        why
    }

    fn not_written(_: &Path, why: Unwritten) -> Self::NotWritten {
        why
    }
}

/// A shape with a `format` key of its own.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TakesTheFormat {
    /// The key the file's own format would overwrite.
    pub(crate) format: i64,
}

impl Kept for TakesTheFormat {
    const FILE: &'static str = "format.toml";
    const FORMAT: i64 = 1;
    const KEYS: &'static [&'static str] = &["format"];
    type NotRead = Unread;
    type NotWritten = Unwritten;

    fn untouched() -> Self {
        Self { format: 0 }
    }

    fn not_read(_: &Path, why: Unread) -> Self::NotRead {
        why
    }

    fn not_written(_: &Path, why: Unwritten) -> Self::NotWritten {
        why
    }
}

/// A folder under the temporary directory that is this test's alone.
pub(crate) fn a_folder_of_our_own(what: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!("alo-kept-{what}"));
    // Whatever a previous run left is gone, so what a test measures is what it
    // wrote.
    if folder.exists() {
        std::fs::remove_dir_all(&folder).unwrap();
    }
    std::fs::create_dir_all(&folder).unwrap();
    folder
}
