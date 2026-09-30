//! A name for a private window that is safe to put on a screen.
//!
//! Part of task 8 of `docs/autonomy/putting-a-window-aside.md`: a private window shows a
//! neutral *Preview hidden* surface **with a safe identifying name**.
//!
//! # Safe means it is not the title
//!
//! A window's title is the first thing that leaks. *Re: settlement figure*, *Dr Ahmed —
//! results*, *Invoice 4471 overdue* — a person putting that window aside is putting it away
//! from the room, and a panel that heads the preview with it has moved the private thing
//! somewhere more visible than it was.
//!
//! So this type is **never derived from a title**. There is no `SafeName::from_title`, no
//! truncation, no redaction pass. Redaction is the wrong shape for the problem: it decides
//! what is sensitive by pattern, and the one it misses is the one on the screen. What is safe
//! is a name chosen to be safe — the application, or a word the person gave.
//!
//! # And it still identifies
//!
//! *Preview hidden* on three rows identifies nothing, and a person with three private windows
//! away has to be able to pick one. So a blank name is refused: this carries the part that
//! distinguishes without describing.

/// Why a name could not be called safe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotSafe {
    /// Blank.
    ///
    /// **Three rows reading *Preview hidden* identify nothing**, and a person with three
    /// private windows away could not pick one.
    #[error("a safe name that says nothing cannot be told apart from another one")]
    ItSaysNothing,
}

/// A name for a private window that may be shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeName(String);

impl SafeName {
    /// A name chosen to be safe.
    ///
    /// The caller is whoever knows what is safe — usually the application's own name, which
    /// says *a Browser window* without saying which page. **Never a title**, for the reason
    /// this module's header gives.
    ///
    /// # Errors
    ///
    /// [`NotSafe::ItSaysNothing`] for a blank name.
    pub fn chosen(name: &str) -> Result<Self, NotSafe> {
        if name.trim().is_empty() {
            return Err(NotSafe::ItSaysNothing);
        }
        Ok(Self(name.to_owned()))
    }

    /// The name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
