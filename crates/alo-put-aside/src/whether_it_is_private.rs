//! Whether a window is private, and if it is, what may be shown instead.
//!
//! Part of task 8 of `docs/autonomy/putting-a-window-aside.md`.
//!
//! # Private carries its safe name, so the pair cannot come apart
//!
//! [`Privacy::Private`] holds the [`SafeName`] rather than sitting beside one. A `bool` plus
//! an `Option<SafeName>` has a fourth state — **private, with no safe name** — and the only
//! thing a surface could do with it is fall back to the title, which is the leak this whole
//! task exists to prevent. Here that state does not exist.
//!
//! # There is no default, and that is deliberate
//!
//! No `Default`, and this is passed to every constructor that needs it rather than assigned
//! afterwards. A default of *ordinary* is the wrong default for privacy: the caller who
//! forgets gets a leaked title, and *forgot* is the commonest thing a caller does.
//!
//! So it is an argument. A caller cannot omit it, because omitting it does not compile —
//! which is the difference between a rule and a guarantee, and this crate has already spent a
//! day on that distinction over where `Zoom` lives.

use crate::a_safe_name::SafeName;

/// Whether a window's contents may be described.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Privacy {
    /// An ordinary window. Its title may be shown.
    Ordinary,
    /// A private window. **Its title may not be shown**, and this is shown instead.
    Private(SafeName),
}

impl Privacy {
    /// Whether the title must be withheld.
    #[must_use]
    pub const fn is_private(&self) -> bool {
        matches!(self, Self::Private(_))
    }

    /// The safe name, for a private window.
    #[must_use]
    pub const fn safe_name(&self) -> Option<&SafeName> {
        match self {
            Self::Ordinary => None,
            Self::Private(name) => Some(name),
        }
    }
}
