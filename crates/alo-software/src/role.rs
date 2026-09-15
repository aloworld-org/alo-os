//! What each application a fresh machine ships is there to do.
//!
//! `docs/autonomy/v0-5-software-and-the-web-plan.md`, task 2, names the list: a
//! web browser, a file manager with trash and archives that open, a text editor,
//! an image viewer, a document viewer and a terminal. **Seven roles for six
//! things**, because the file manager a fresh machine ships opens archives
//! through a second application from the same developer, and a role with two
//! applications behind it would be a list that cannot say which one is missing.
//!
//! A role is a closed list rather than a string, so [`crate::Shipped`] can
//! refuse a list that lacks one and a caller can ask for *the terminal* without
//! knowing today's identifier.

use serde::Deserialize;

/// What an application on a fresh machine is there to do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Role {
    /// The open web.
    WebBrowser,
    /// Folders and files, with the trash.
    FileManager,
    /// The archives a file manager hands on, opened and made.
    Archives,
    /// Plain text.
    TextEditor,
    /// Pictures.
    ImageViewer,
    /// Documents to read, PDF first.
    DocumentViewer,
    /// A person's own shell — and never an agent's (ADR 0043).
    Terminal,
}

impl Role {
    /// Every role, in the order the list is kept.
    pub const EVERY: [Self; 7] = [
        Self::WebBrowser,
        Self::FileManager,
        Self::Archives,
        Self::TextEditor,
        Self::ImageViewer,
        Self::DocumentViewer,
        Self::Terminal,
    ];

    /// The role as the list names it.
    #[must_use]
    pub const fn named(self) -> &'static str {
        match self {
            Self::WebBrowser => "web-browser",
            Self::FileManager => "file-manager",
            Self::Archives => "archives",
            Self::TextEditor => "text-editor",
            Self::ImageViewer => "image-viewer",
            Self::DocumentViewer => "document-viewer",
            Self::Terminal => "terminal",
        }
    }

    /// Where this role is in [`Role::EVERY`].
    pub(crate) const fn place(self) -> usize {
        match self {
            Self::WebBrowser => 0,
            Self::FileManager => 1,
            Self::Archives => 2,
            Self::TextEditor => 3,
            Self::ImageViewer => 4,
            Self::DocumentViewer => 5,
            Self::Terminal => 6,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_role_is_in_its_own_place() {
        for (at, role) in Role::EVERY.into_iter().enumerate() {
            assert_eq!(role.place(), at, "{}", role.named());
        }
    }
}
