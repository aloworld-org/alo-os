//! What a grant covers, and what a verb asks to touch.
//!
//! Two types facing each other. [`Reach`] is the person's side: the folder they
//! picked, the document they offered, the application they allowed. [`Ask`] is
//! the agent's side: the one thing a verb wants to act on, right now. The whole
//! question this crate answers is whether some [`Reach`] a person made covers
//! the [`Ask`] a verb arrived with.
//!
//! Keeping them apart is what stops reach being widened by use. There is no
//! method here that turns an [`Ask`] into a [`Reach`]; a grant is made by
//! picking a folder and by nothing else.
//!
//! **Identities are matched exactly.** A folder is compared component by
//! component and an application by its identifier, with no case folding.
//! Matching loosely means matching *more* than the person picked, and on this
//! side of the system every widening is a security bug. Names people type are
//! compared kindly elsewhere in alo OS; names the system enumerates are not.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::path::{is_exactly, is_inside};

/// What a grant covers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Reach {
    /// A folder, and everything inside it. What a person picks in a file
    /// chooser.
    Folder(PathBuf),
    /// Exactly one file, and nothing else — the document offered at
    /// invocation, which is a grant with a very short life rather than a
    /// special case (ADR 0001 §4).
    File(PathBuf),
    /// One installed application, by the identifier the system knows it by.
    Application(String),
}

/// What a verb is asking to touch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Ask {
    /// A path on this machine. Resolved by the caller before it gets here —
    /// see [`crate::path`] on symbolic links.
    Path(PathBuf),
    /// An installed application, by its identifier.
    Application(String),
}

impl Ask {
    /// A question about this path.
    #[must_use]
    pub fn path(path: impl Into<PathBuf>) -> Self {
        Self::Path(path.into())
    }

    /// A question about this application.
    #[must_use]
    pub fn application(id: &str) -> Self {
        Self::Application(id.trim().to_owned())
    }

    /// What was asked for, in words a person can read in a refusal.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::Path(path) => path.display().to_string(),
            Self::Application(id) => format!("the application {id}"),
        }
    }
}

impl Reach {
    /// Whether this reach covers what is being asked for.
    ///
    /// A folder covers itself and everything under it; a file covers itself
    /// alone; an application covers no path at all. A grant to a folder is not
    /// a grant to an application that happens to live in it, which is why the
    /// mismatched pairs answer `false` rather than being clever.
    #[must_use]
    pub fn covers(&self, ask: &Ask) -> bool {
        match (self, ask) {
            (Self::Folder(folder), Ask::Path(path)) => is_inside(folder, path),
            (Self::File(file), Ask::Path(path)) => is_exactly(file, path),
            (Self::Application(granted), Ask::Application(wanted)) => granted == wanted,
            (Self::Folder(_) | Self::File(_), Ask::Application(_))
            | (Self::Application(_), Ask::Path(_)) => false,
        }
    }

    /// The path this reach is over, when it is over one.
    #[must_use]
    pub fn as_path(&self) -> Option<&Path> {
        match self {
            Self::Folder(path) | Self::File(path) => Some(path),
            Self::Application(_) => None,
        }
    }

    /// What is granted, in words — the line a person reads in the list of
    /// grants, with the times left to whoever is displaying it.
    ///
    /// Formatting an expiry here would hardcode English and a calendar, and
    /// this crate is not where that decision belongs.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            Self::Folder(path) => format!("{} and everything in it", path.display()),
            Self::File(path) => format!("the file {}", path.display()),
            Self::Application(id) => format!("the application {id}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder() -> Reach {
        Reach::Folder(PathBuf::from("/home/anna/Invoices"))
    }

    #[test]
    fn a_folder_covers_what_is_in_it_and_stops_at_its_edge() {
        assert!(folder().covers(&Ask::path("/home/anna/Invoices/march.pdf")));
        assert!(folder().covers(&Ask::path("/home/anna/Invoices/2024/march.pdf")));
        assert!(!folder().covers(&Ask::path("/home/anna/Taxes/march.pdf")));
        assert!(!folder().covers(&Ask::path("/home/anna")));
    }

    /// The document offered at invocation is one file, not the folder it
    /// happens to sit in.
    #[test]
    fn a_file_covers_itself_and_nothing_beside_it() {
        let reach = Reach::File(PathBuf::from("/home/anna/Invoices/march.pdf"));
        assert!(reach.covers(&Ask::path("/home/anna/Invoices/march.pdf")));
        assert!(!reach.covers(&Ask::path("/home/anna/Invoices/april.pdf")));
        assert!(!reach.covers(&Ask::path("/home/anna/Invoices")));
    }

    /// An application identifier is matched exactly. Matching loosely would
    /// cover more than the person allowed.
    #[test]
    fn an_application_is_matched_exactly() {
        let reach = Reach::Application("org.blender.Blender".to_owned());
        assert!(reach.covers(&Ask::application("org.blender.Blender")));
        assert!(reach.covers(&Ask::application("  org.blender.Blender  ")));
        assert!(!reach.covers(&Ask::application("org.blender.blender")));
        assert!(!reach.covers(&Ask::application("org.blender.Blender2")));
    }

    /// A grant of one kind is never a grant of another kind.
    #[test]
    fn a_folder_is_not_a_grant_to_an_application_or_the_other_way_round() {
        assert!(!folder().covers(&Ask::application("org.blender.Blender")));
        let app = Reach::Application("org.blender.Blender".to_owned());
        assert!(!app.covers(&Ask::path("/home/anna/Invoices/march.pdf")));
        assert!(app.as_path().is_none());
    }

    #[test]
    fn what_is_granted_reads_as_a_sentence() {
        assert!(folder().describe().contains("and everything in it"));
        assert!(
            Reach::File(PathBuf::from("/home/anna/march.pdf"))
                .describe()
                .starts_with("the file")
        );
    }
}
