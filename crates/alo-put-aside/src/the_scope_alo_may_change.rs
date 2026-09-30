//! What alo may change while it works in a window put aside.
//!
//! Part of task 7 of `docs/autonomy/putting-a-window-aside.md`. A person who put a window
//! away and left alo working in it is owed the answer to *what can it touch* without
//! bringing the window back.
//!
//! # Reading and changing are different states, not a list that happens to be empty
//!
//! The repository's standing rule is **reads answer, changes wait**: a read runs inside the
//! turn, any change to the machine is proposed and waits for one approval. So *alo is
//! reading this* and *alo may change these three things* are different answers, and
//! [`Scope::MayChange`] refuses an empty list rather than representing the first.
//!
//! That refusal is the whole reason this is a type. An empty `MayChange` drawn by a
//! surface is the words *may change* followed by nothing — **an empty control**, which is
//! what ADR 0009 forbids in this panel: *the agent's surfaces disappear rather than nag*,
//! and *a greyed-out feature is an advertisement*. An empty list is that advertisement
//! with its label still attached.

use std::path::PathBuf;

/// Why a scope could not be made.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotAScope {
    /// A change scope naming nothing.
    ///
    /// **Use [`Scope::ReadsOnly`] instead**, which is what it means — a surface shown this
    /// would write *may change* above an empty space, and a person reading it would not
    /// know whether that meant *nothing* or *not loaded yet*.
    #[error("a change scope that names nothing is reading, and reading has its own answer")]
    ItChangesNothing,
}

/// What alo may touch in this window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Scope {
    /// It is reading, and will propose before it changes anything.
    ReadsOnly,
    /// It may change these, and nothing else.
    MayChange(Vec<PathBuf>),
}

impl Scope {
    /// A scope that may change these things.
    ///
    /// # Errors
    ///
    /// [`NotAScope::ItChangesNothing`] if the list is empty. See this module's header: an
    /// empty change scope is reading, and a surface drawn from one shows an empty control.
    pub fn may_change(what: Vec<PathBuf>) -> Result<Self, NotAScope> {
        if what.is_empty() {
            return Err(NotAScope::ItChangesNothing);
        }
        Ok(Self::MayChange(what))
    }

    /// Whether anything can be changed under this scope.
    #[must_use]
    pub const fn can_change_anything(&self) -> bool {
        matches!(self, Self::MayChange(_))
    }

    /// What may be changed, or nothing at all.
    ///
    /// Returns an empty slice for [`Scope::ReadsOnly`] **and a caller must not read that as
    /// the same thing** — `can_change_anything` is the question that distinguishes them.
    /// The slice is here so a surface can list the entries it is about to draw, not so that
    /// emptiness can stand in for reading.
    #[must_use]
    pub fn what(&self) -> &[PathBuf] {
        match self {
            Self::ReadsOnly => &[],
            Self::MayChange(what) => what,
        }
    }
}
