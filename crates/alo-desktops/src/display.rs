//! Which display a set of desktops belongs to, and what a caller that named one
//! wrongly is told.
//!
//! Desktops are **per display**: the laptop's own screen and an external one
//! each have their own desktops, their own order and their own current one, so
//! switching on one screen does not change what is on the other. That is
//! `docs/features.md` v0.5 — *splitting works on an external display
//! independently of the laptop's own* — carried up from a division to the
//! desktop the division is on.
//!
//! # A live handle, not a stable identity
//!
//! A [`DisplayId`] is the number the compositor uses for a screen while it is
//! plugged in. It is deliberately **not** the identity a remembered arrangement
//! is kept under: the session plan's `alo-displays` gives a display an identity
//! that survives being unplugged, and task 2 of this crate's plan is what keeps
//! anything under it. Until then this crate holds desktops for the session and
//! hands them back whole when a display goes ([`crate::Desktops::unplug`]), so
//! whoever remembers them has something to remember.

use std::fmt;

/// Which display, as the compositor names it.
///
/// Opaque here: this crate compares two of them and never looks inside.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DisplayId(u64);

impl DisplayId {
    /// The display the compositor calls this.
    #[must_use]
    pub const fn from_compositor(id: u64) -> Self {
        Self(id)
    }

    /// What the compositor called it.
    #[must_use]
    pub const fn to_compositor(self) -> u64 {
        self.0
    }
}

/// Why a display could not be plugged in or unplugged.
///
/// **English, and said to the compositor rather than to a person.** Both of
/// these mean the caller and this crate disagree about which screens exist,
/// which is alo OS's own bug: there is nothing to ask a person and nothing they
/// could do about it. The refusals a person reads are [`crate::Refused`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotADisplay {
    /// No desktops are held for that display.
    #[error("display {0} has no desktops: it was never plugged in, or has already gone")]
    Unknown(DisplayId),
    /// That display already has desktops, and plugging it in again would
    /// discard them.
    #[error("display {0} already has desktops, which plugging it in again would discard")]
    AlreadyThere(DisplayId),
}

impl fmt::Display for DisplayId {
    /// The compositor's number, for the two sentences above.
    ///
    /// Not a person's reading of anything: a display has no name here, and the
    /// one a person would read is `alo-displays`' when it exists.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A display is the number it was handed, and says so in the two sentences
    /// that are the compositor's to read.
    #[test]
    fn a_display_is_the_number_the_compositor_gave_it() {
        let display = DisplayId::from_compositor(3);
        assert_eq!(display.to_compositor(), 3);
        assert_eq!(
            NotADisplay::Unknown(display).to_string(),
            "display 3 has no desktops: it was never plugged in, or has already gone"
        );
        assert_eq!(
            NotADisplay::AlreadyThere(display).to_string(),
            "display 3 already has desktops, which plugging it in again would discard"
        );
    }
}
