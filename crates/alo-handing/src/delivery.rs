//! How what was dropped reaches the window it was dropped on.
//!
//! Two ways, and which one is not a preference: it is decided by what was
//! dragged and whether the window is sandboxed.
//!
//! # Text and images go straight over, exactly as a paste does
//!
//! [`Delivery::OnTheSpot`] carries the bytes the source gave. That is the whole
//! of a drop for every form but one, and it is the same journey
//! `alo_clipboard::Clipboard::paste` makes — the application that had the data
//! is asked for the form the target takes, and what it answers is what arrives.
//!
//! # Files to a sandboxed window go through the documents portal
//!
//! A path is meaningless inside a sandbox. `/home/anna/march.pdf` is not
//! visible to a Flatpak that was never granted it, so a drop that handed the
//! path over would arrive as a file that does not exist — and a compositor that
//! made it exist by widening the sandbox would have turned a drag of the wrist
//! into a grant over the person's home folder.
//!
//! So [`Delivery::ThroughTheDocuments`] names the files, and the portal exports
//! each of them to that application: the window is handed the exported
//! locations, which are the same bytes of file content under names the sandbox
//! can open, and **no path of the person's crosses the boundary**. That is the
//! journey `xdg-desktop-portal` makes for every drag onto a sandboxed
//! application, and it is what ADR 0005's applications expect.
//!
//! What this crate decides is *which* files are to be exported and to which
//! application. The export itself is the portal's — the names it gives them are
//! its to choose, which is why nothing here pretends to know the bytes the
//! window will read. A delivery that carried a guess at them would be this
//! crate answering a question the portal has not been asked yet.
//!
//! # And a drop is not a grant either way
//!
//! An exported file is reachable by the application that was dropped on, for as
//! long as the portal keeps it, and by nothing else. It is not a grant to an
//! agent, not a grant to the person's folder, and not a widening of anything:
//! ADR 0001 §3's grants are made in `alo-picking`, and this crate has no
//! `alo-capability` dependency to make one with.

use std::path::{Path, PathBuf};

/// How what was dropped gets to the window it was dropped on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Delivery {
    /// The bytes the source gave, handed straight to the window.
    OnTheSpot(Vec<u8>),
    /// These files, each exported to the application through the documents
    /// portal before the window is told about them.
    ThroughTheDocuments {
        /// The files to export, in the order they were dragged.
        files: Vec<PathBuf>,
    },
}

impl Delivery {
    /// The bytes handed over, for a delivery that hands bytes over.
    ///
    /// [`None`] for files to a sandboxed window: what that window reads is the
    /// exported locations, and the portal is what names them.
    #[must_use]
    pub fn bytes(&self) -> Option<&[u8]> {
        match self {
            Self::OnTheSpot(bytes) => Some(bytes),
            Self::ThroughTheDocuments { .. } => None,
        }
    }

    /// The files to be exported, for a delivery that exports files.
    #[must_use]
    pub fn files(&self) -> Option<&[PathBuf]> {
        match self {
            Self::OnTheSpot(_) => None,
            Self::ThroughTheDocuments { files } => Some(files),
        }
    }

    /// Whether this delivery would export a particular file.
    #[must_use]
    pub fn exports(&self, file: &Path) -> bool {
        self.files()
            .is_some_and(|files| files.iter().any(|named| named == file))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bytes on the spot, and nothing to export.
    #[test]
    fn bytes_handed_over_are_read_back_and_export_nothing() {
        let delivery = Delivery::OnTheSpot(b"the second paragraph".to_vec());
        assert_eq!(delivery.bytes(), Some(b"the second paragraph".as_slice()));
        assert_eq!(delivery.files(), None);
        assert!(!delivery.exports(Path::new("/home/anna/march.pdf")));
    }

    /// Files to export, and **no bytes at all** — which is the guarantee: there
    /// is nothing in this variant for a path to be carried in.
    #[test]
    fn files_through_the_portal_carry_no_bytes_and_no_path() {
        let delivery = Delivery::ThroughTheDocuments {
            files: vec![PathBuf::from("/home/anna/march.pdf")],
        };
        assert_eq!(delivery.bytes(), None);
        assert!(delivery.exports(Path::new("/home/anna/march.pdf")));
        assert!(!delivery.exports(Path::new("/home/anna/april.pdf")));
    }
}
